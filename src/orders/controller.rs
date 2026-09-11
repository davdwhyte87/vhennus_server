use std::{env, str::FromStr};

use actix_web::{
    get, post,
    web::{self, Data, ReqData},
    HttpResponse,
};
use actix_web_validator::Json;
use bigdecimal::{num_bigint::BigInt, BigDecimal};
use futures::FutureExt;
use mongodb::{
    bson::doc,
    error::Error,
    Client, ClientSession,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::orders::models::{
    BuyOrder, CreateBuyOrderReq, CreateOrderMessageReq, CreatePaymentMethodReq,
    CreateSellOrderReq, Currency, OrderMessage, PaymentMethod, PaymentMethodData, SellOrder,
    UpdateSellOrderReq,
};
use crate::orders::repository::{BUY_ORDER_COLLECTION, SELL_ORDER_COLLECTION};
use crate::orders::service::{
    BuyOrderService, OrderMessageService, PaymentMethodService, SellOrderService,
};
use crate::shared::auth::Claims;
use crate::shared::db::MongoService;
use crate::shared::formatter;
use crate::shared::response::{GenericResp, Response};
use crate::shared::tcp::send_to_tcp_server;
use crate::shared::vcrypto::TransferReq;

#[post("/buy")]
pub async fn create_buy_order(
    database: Data<MongoService>,
    new_order: Json<CreateBuyOrderReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    println!("new req");
    let mut respData = GenericResp::<BuyOrder> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();
            respData.data = None;
            respData.server_message = None;
            return HttpResponse::Unauthorized().json(respData);
        }
    };

    // get sell order
    let sell_order_c =
        SellOrderService::get_sell_order_by_id(&database.db, new_order.sell_order_id.to_owned())
            .await;
    let sell_order_c = match sell_order_c {
        Ok(data) => data,
        Err(err) => {
            respData.message = "Error getting  sell order".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());

            return HttpResponse::InternalServerError().json(respData);
        }
    };

    // make sure request amount is good fit for the sell order

    if new_order.amount > sell_order_c.amount {
        respData.message = "Buy order is larger than sell order".to_string();
        respData.data = None;
        respData.server_message = None;
        return HttpResponse::BadRequest().json(respData);
    }
    if new_order.amount > sell_order_c.max_amount || new_order.amount < sell_order_c.min_amount {
        respData.message = "buy order is larger or smaller than sell order".to_string();
        respData.data = None;
        respData.server_message = None;
        return HttpResponse::BadRequest().json(respData);
    }

    // create buy order
    let mut buy_order = BuyOrder {
        id: Uuid::new_v4().to_string(),
        user_name: claim.user_name.clone(),
        is_buyer_confirmed: false,
        amount: new_order.amount.to_owned(),
        sell_order_id: new_order.sell_order_id.to_owned(),
        is_seller_confirmed: false,
        is_canceled: false,
        is_reported: false,
        created_at: chrono::offset::Utc::now().to_string(),
        updated_at: chrono::offset::Utc::now().to_string(),
        wallet_address: new_order.wallet_address.to_owned(),
    };

    let seller_order_id = new_order.sell_order_id.to_owned();

    let mut session = database.client.start_session().await.unwrap();

    match session.start_transaction().await {
        Ok(_) => {}
        Err(err) => {
            respData.message = "error creating buy order".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::InternalServerError().json(respData);
        }
    }

    // start transaction
    let result = async {
        create_buy_order_update_sell_order(
            &mut session,
            buy_order.clone(),
            seller_order_id.clone(),
            sell_order_c.amount - new_order.amount.clone(),
        )
        .await
    }
    .await;

    match result {
        Ok(_) => {
            session.commit_transaction().await;
            println!("Transaction succeeded");
        }
        Err(e) => {
            session.abort_transaction().await;
            respData.message = "error creating buy order".to_string();
            respData.data = None;
            respData.server_message = Some(e.to_string());
            return HttpResponse::InternalServerError().json(respData);
        }
    }

    respData.message = "Ok".to_string();
    respData.data = Some(buy_order.clone());
    respData.server_message = None;

    return HttpResponse::Ok().json(respData);
}

async fn create_buy_order_update_sell_order(
    session: &mut ClientSession,
    mut buy_order: BuyOrder,
    sell_order_id: String,
    new_amount: BigDecimal,
) -> Result<(), Error> {
    let sell_order_collection = session
        .client()
        .database(&MongoService::get_db_name())
        .collection::<SellOrder>(SELL_ORDER_COLLECTION);
    let buy_order_collection = session
        .client()
        .database(&MongoService::get_db_name())
        .collection::<BuyOrder>(BUY_ORDER_COLLECTION);

    let buy_order_id = buy_order.id.to_owned();
    match buy_order_collection.insert_one(buy_order).await {
        Ok(_) => {}
        Err(err) => {
            return Err(err.into());
        }
    };

    // get sell order
    let sell_order_id_2 = sell_order_id.clone();
    let filter = doc! {"id":sell_order_id};
    let order = sell_order_collection.find_one(filter).await;
    let mut order_data = match order {
        Ok(data) => match data {
            Some(data) => data,
            None => {
                return Err(Error::custom("No data found".to_string()));
            }
        },
        Err(err) => {
            return Err(err.into());
        }
    };

    // update sell order
    order_data.buy_orders_id.push(buy_order_id);

    // save sell data
    let update_filter = doc! {"id": sell_order_id_2};
    let update_data = doc! {"$set":doc! {"buy_orders_id":order_data.buy_orders_id, "amount":new_amount.to_string()}};
    match sell_order_collection
        .update_one(update_filter, update_data)
        .await
    {
        Ok(_) => {}
        Err(err) => {
            return Err(err.into());
        }
    };

    Ok(())
}

#[get("/my_orders")]
pub async fn get_my_buy_orders(
    database: Data<MongoService>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let mut respData = GenericResp::<Vec<BuyOrder>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();
            respData.data = None;
            respData.server_message = None;
            return HttpResponse::Unauthorized().json(respData);
        }
    };

    let orders =
        match BuyOrderService::get_all_buy_order_by_username(&database.db, claim.user_name.clone())
            .await
        {
            Ok(data) => data,
            Err(err) => {
                respData.message = "error getting buy order".to_string();
                respData.data = None;
                respData.server_message = Some(err.to_string());
                return HttpResponse::InternalServerError().json(respData);
            }
        };

    respData.message = "ok".to_string();
    respData.data = Some(orders);
    respData.server_message = None;
    return HttpResponse::Ok().json(respData);
}

#[derive(Deserialize)]
struct GetBuyOrderPath {
    id: String,
}

#[get("/single/{id}")]
pub async fn get_single_buy_order(
    database: Data<MongoService>,
    claim: Option<ReqData<Claims>>,
    info: web::Path<GetBuyOrderPath>,
) -> HttpResponse {
    let mut respData = GenericResp::<BuyOrder> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };
    // get claim
    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();
            respData.data = None;
            respData.server_message = None;
            return HttpResponse::Unauthorized().json(respData);
        }
    };

    // get buy otder by id
    let order = match BuyOrderService::get_single_order_by_id(&database.db, info.id.to_owned()).await
    {
        Ok(data) => match data {
            Some(data) => data,
            None => {
                respData.message = "No data found".to_string();
                respData.data = None;
                respData.server_message = None;
                return HttpResponse::BadRequest().json(respData);
            }
        },
        Err(err) => {
            respData.message = "Error getting buy order".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData);
        }
    };

    respData.message = "ok".to_string();
    respData.data = Some(order);
    respData.server_message = None;
    return HttpResponse::Ok().json(respData);
}

pub async fn escrow_to_user(address: String, amount: BigDecimal) -> Result<(), Box<dyn std::error::Error>> {
    // send message to the kuracoin blockchain to create new user
    let kura_coin_server_ip = match env::var("KURACOIN_SERVER_ID") {
        Ok(data) => data.to_owned(),
        Err(err) => {
            println!("{}", err.to_string());
            return Err(Box::from("Error connecting to blockchain"));
        }
    };

    let escrow_wallet = match env::var("ESCROW_WALLET") {
        Ok(data) => data.to_owned(),
        Err(err) => {
            println!("{}", err.to_string());
            return Err(Box::from("Error connecting to blockchain"));
        }
    };

    let escrow_wallet_password = match env::var("ESCROW_WALLET_PASSWORD") {
        Ok(data) => data.to_owned(),
        Err(err) => {
            println!("{}", err.to_string());
            return Err(Box::from("Error connecting to blockchain"));
        }
    };

    let message_data = match serde_json::to_string(&TransferReq {
        sender: escrow_wallet,
        receiver: address,
        amount: amount,
        transaction_id: Uuid::new_v4().to_string(),
        sender_password: escrow_wallet_password,
    }) {
        Ok(data) => data,
        Err(err) => {
            println!("{}", err.to_string());
            return Err(Box::from("Error parsing data"));
        }
    };
    let message = formatter::Formatter::request_formatter(
        "Transfer".to_string(),
        message_data,
        "".to_string(),
        "".to_string(),
        "0".to_string(),
    );

    let m = message.clone();
    let ip = kura_coin_server_ip.clone();
    let result = web::block(move || send_to_tcp_server(m, ip)).await;
    let response_string = match result {
        Ok(data) => match data {
            Ok(data) => data,
            Err(err) => {
                println!("{}", err.to_string());
                return Err(Box::from("Error parsing data"));
            }
        },
        Err(err) => {
            println!("{}", err.to_string());
            return Err(Box::from("Error parsing data"));
        }
    };

    let resp_data: Vec<&str> = response_string.split('\n').collect();
    let code = match resp_data.get(0) {
        Some(data) => data,
        None => {
            return Err(Box::from("Error decoding data from blockchain server"));
        }
    };

    if (*code != "1") {
        // blockchain request failed
        return Err(Box::from("Failed trnasfer on blockchain"));
    }

    return Ok(());
}

#[get("/buyer_confirmed/{id}")]
pub async fn buyer_confirmed(
    database: Data<MongoService>,
    claim: Option<ReqData<Claims>>,
    info: web::Path<GetBuyOrderPath>,
) -> HttpResponse {
    let mut respData = GenericResp::<BuyOrder> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };
    // get claim
    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();
            respData.data = None;
            respData.server_message = None;
            return HttpResponse::Unauthorized().json(respData);
        }
    };

    // get buy order

    let mut buy_order =
        match BuyOrderService::get_single_order_by_id(&database.db, info.id.to_owned()).await {
            Ok(data) => match data {
                Some(data) => data,
                None => {
                    respData.message = "Could not find order".to_string();
                    respData.data = None;
                    respData.server_message = None;
                    return HttpResponse::NotFound().json(respData);
                }
            },
            Err(err) => {
                respData.message = "Error getting buy order".to_string();
                respData.data = None;
                respData.server_message = Some(err.to_string());
                return HttpResponse::BadRequest().json(respData);
            }
        };

    // make sure user owns order
    if buy_order.user_name != claim.user_name {
        respData.message = "Unauthorized".to_string();
        respData.data = None;
        respData.server_message = None;
        return HttpResponse::Unauthorized().json(respData);
    }
    // modify order
    buy_order.is_buyer_confirmed = true;

    // update database
    match BuyOrderService::update(&database.db, &buy_order).await {
        Ok(_) => {}
        Err(err) => {
            respData.message = "Error updating buy order".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData);
        }
    }

    // check if it is time to release coins
    if buy_order.is_buyer_confirmed && buy_order.is_seller_confirmed {
        match escrow_to_user(buy_order.wallet_address, buy_order.amount).await {
            Ok(_) => {}
            Err(err) => {
                println!("unable to release coins {}", err.to_string());
                respData.message = "Unable to release coins".to_string();
                respData.data = None;
                respData.server_message = Some(err.to_string());
                return HttpResponse::InternalServerError().json(respData);
            }
        }
    }

    respData.message = "Buy order confirmed".to_string();
    respData.data = None;
    respData.server_message = None;
    return HttpResponse::Ok().json(respData);
}

#[get("/seller_confirmed/{id}")]
pub async fn seller_confirmed(
    database: Data<MongoService>,
    claim: Option<ReqData<Claims>>,
    info: web::Path<GetBuyOrderPath>,
) -> HttpResponse {
    let mut respData = GenericResp::<BuyOrder> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };

    // get claim
    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();
            respData.data = None;
            respData.server_message = None;
            return HttpResponse::Unauthorized().json(respData);
        }
    };

    // get buy order

    let mut buy_order =
        match BuyOrderService::get_single_order_by_id(&database.db, info.id.to_owned()).await {
            Ok(data) => match data {
                Some(data) => data,
                None => {
                    respData.message = "Could not find order".to_string();
                    respData.data = None;
                    respData.server_message = None;
                    return HttpResponse::NotFound().json(respData);
                }
            },
            Err(err) => {
                respData.message = "Error getting data".to_string();
                respData.data = None;
                respData.server_message = Some(err.to_string());
                return HttpResponse::BadRequest().json(respData);
            }
        };

    // get sell order
    let sell_order = match SellOrderService::get_sell_order_by_id(
        &database.db,
        buy_order.sell_order_id.to_owned(),
    )
    .await
    {
        Ok(data) => data,
        Err(err) => {
            respData.message = "Error getting data".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData);
        }
    };

    // make sure user owns sell order
    if sell_order.user_name != claim.user_name {
        respData.message = "Unauthorized".to_string();
        respData.data = None;
        respData.server_message = None;
        return HttpResponse::Unauthorized().json(respData);
    }
    // modify order
    buy_order.is_seller_confirmed = true;

    // update database
    match BuyOrderService::update(&database.db, &buy_order).await {
        Ok(_) => {}
        Err(err) => {
            respData.message = "Error saving data".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData);
        }
    }

    // check if it is time to release coins
    if buy_order.is_buyer_confirmed && buy_order.is_seller_confirmed {
        match escrow_to_user(buy_order.wallet_address, buy_order.amount).await {
            Ok(_) => {}
            Err(err) => {
                println!("unable to release coins {}", err.to_string());
                respData.message = "Unable to release coins".to_string();
                respData.data = None;
                respData.server_message = Some(err.to_string());
                return HttpResponse::InternalServerError().json(respData);
            }
        }
    }

    respData.message = "Ok".to_string();
    respData.data = None;
    respData.server_message = None;

    return HttpResponse::Ok().json(respData);
}

#[get("/cancel/{id}")]
pub async fn cancel_buy_order(
    database: Data<MongoService>,
    claim: Option<ReqData<Claims>>,
    info: web::Path<GetBuyOrderPath>,
) -> HttpResponse {
    let mut respData = GenericResp::<BuyOrder> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };
    // get claim
    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Not authorized".to_string();
            respData.data = None;
            respData.server_message = None;
            return HttpResponse::Unauthorized().json(respData);
        }
    };

    // get buy order

    let mut buy_order =
        match BuyOrderService::get_single_order_by_id(&database.db, info.id.to_owned()).await {
            Ok(data) => match data {
                Some(data) => data,
                None => {
                    respData.message = "Could not find data".to_string();
                    respData.data = None;
                    respData.server_message = None;
                    return HttpResponse::NotFound().json(respData);
                }
            },
            Err(err) => {
                log::error!("error getting buy order {}", err);
                respData.message = "Error getting data".to_string();
                respData.data = None;
                respData.server_message = Some(err.to_string());
                return HttpResponse::BadRequest().json(respData);
            }
        };

    // make sure request user is the order owner
    if buy_order.user_name != claim.user_name {
        respData.message = "Not authorized".to_string();
        respData.data = None;
        respData.server_message = None;
        return HttpResponse::Unauthorized().json(respData);
    }

    // check if order is completed
    if buy_order.is_buyer_confirmed || buy_order.is_seller_confirmed {
        respData.message = "Buy Order has been completed buy buyer/seller".to_string();
        respData.data = None;
        respData.server_message = None;
        return HttpResponse::BadRequest().json(respData);
    }

    // send money back to the sell order
    // get sell order
    let mut sell_order =
        match SellOrderService::get_sell_order_by_id(&database.db, buy_order.sell_order_id.to_owned())
            .await
        {
            Ok(data) => data,
            Err(err) => {
                log::error!("error getting sell order {}", err);
                respData.message = "Error cancelling order".to_string();
                respData.data = None;
                respData.server_message = Some(err.to_string());
                return HttpResponse::BadRequest().json(respData);
            }
        };

    // return funds to sell order
    sell_order.amount = sell_order.amount + buy_order.amount.to_owned();

    // update sell order
    match SellOrderService::update(&database.db, &sell_order).await {
        Ok(_) => {}
        Err(err) => {
            log::error!("error saving sell order {}", err);
            respData.message = "Error saving order".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData);
        }
    };

    // update order
    buy_order.is_canceled = true;

    // update dataase
    match BuyOrderService::update(&database.db, &buy_order).await {
        Ok(_) => {}
        Err(err) => {
            log::error!("error updating buy order {}", err);
            respData.message = "Error updating order".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData);
        }
    }

    respData.message = "Ok".to_string();
    respData.data = None;
    respData.server_message = None;
    return HttpResponse::Ok().json(respData);
}

#[post("/create")]
pub async fn create_payment_method(
    database: Data<MongoService>,
    req: Json<CreatePaymentMethodReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let mut respData = GenericResp::<PaymentMethodData> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };

    let claim = match claim {
        Some(claim) => claim,
        None => {
            return HttpResponse::Unauthorized().json(Response {
                message: "Not authorized".to_string(),
            });
        }
    };
    let payment_method = PaymentMethodData {
        id: Uuid::new_v4().to_string(),
        user_name: claim.user_name.to_owned(),
        payment_method: req.payment_method.to_owned(),
        account_name: req.account_name.to_owned(),
        account_number: req.account_number.to_owned(),
        bank_name: req.bank_name.to_owned(),
        other: req.other.to_owned(),
        paypal_email: req.paypal_email.to_owned(),
        venmo_username: req.venmo_username.to_owned(),
        skrill_email: req.skrill_email.to_owned(),
        name: req.name.to_owned(),
    };

    match PaymentMethodService::create_payment_method(&database.db, &payment_method).await {
        Ok(_) => {}
        Err(err) => {
            log::error!(" error creating payment method {}", err.to_string());
            respData.message = "Error creating payment method".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());

            return HttpResponse::BadRequest().json(respData);
        }
    }

    respData.message = "Ok".to_string();
    respData.data = Some(payment_method);
    respData.server_message = None;

    return HttpResponse::Ok().json(respData);
}

#[derive(Deserialize)]
struct GetSingleSellOrderPath {
    id: String,
}

#[get("/delete/{id}")]
pub async fn delete_payment_method(
    database: Data<MongoService>,
    claim: Option<ReqData<Claims>>,
    info: web::Path<GetSingleSellOrderPath>,
) -> HttpResponse {
    let mut respData = GenericResp::<Vec<SellOrder>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();
            return HttpResponse::Unauthorized().json(respData);
        }
    };

    match PaymentMethodService::delete_user_payment_method(&database.db, info.id.to_owned()).await {
        Ok(_) => {}
        Err(err) => {
            log::error!("error deleting payment method  {}", err.to_string());
            respData.message = "Error deleting payment method".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData);
        }
    }
    respData.message = "Ok".to_string();
    respData.data = None;
    respData.server_message = None;

    return HttpResponse::Ok().json(respData);
}

#[get("/my_payment_methods")]
pub async fn get_my_payment_methods(
    database: Data<MongoService>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let mut respData = GenericResp::<Vec<PaymentMethodData>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };
    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Not authorized".to_string();
            respData.data = None;
            respData.server_message = None;
            return HttpResponse::Unauthorized().json(respData);
        }
    };

    let methods = match PaymentMethodService::get_all_user_payment_method_data(
        &database.db,
        claim.user_name.to_owned(),
    )
    .await
    {
        Ok(data) => data,
        Err(err) => {
            log::error!(" error  gettting user payment mthods {}", err.to_string());
            respData.message = "Error getting  payment method".to_string();
            respData.data = None;
            respData.server_message = Some(err.to_string());

            return HttpResponse::BadRequest().json(respData);
        }
    };

    respData.message = "Ok".to_string();
    respData.data = Some(methods);
    respData.server_message = None;
    return HttpResponse::Ok().json(respData);
}

#[post("/post")]
pub async fn create_order_message(
    database: Data<MongoService>,
    new_message: Json<CreateOrderMessageReq>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let mut respData = GenericResp::<OrderMessage> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: Some(OrderMessage::default()),
    };
    println!("new req");

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized().json(respData);
        }
    };

    let order_message = OrderMessage {
        id: Uuid::new_v4().to_string(),
        text: new_message.text.to_owned(),
        image: new_message.image.to_owned(),
        created_at: chrono::offset::Utc::now().to_string(),
        sender_user_name: claim.user_name.to_owned(),
        receiver_user_name: new_message.receiver_user_name.to_owned(),
        buy_order_id: new_message.buy_order_id.to_owned(),
    };

    let response = match OrderMessageService::create_message(&database.db, &order_message).await {
        Ok(data) => data,
        Err(err) => {
            log::error!(" error getting order messages  {}", err.to_string());
            respData.data = None;
            respData.message = "Error creating message".to_string();
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData);
        }
    };

    respData.data = Some(order_message);
    respData.message = "Created".to_string();
    respData.server_message = None;
    return HttpResponse::Ok().json(respData);
}

#[derive(Deserialize)]
pub struct IDPath {
    pub id: String,
}

#[get("/get_all/{id}")]
pub async fn get_order_message(
    database: Data<MongoService>,
    info: web::Path<IDPath>,
    claim: Option<ReqData<Claims>>,
) -> HttpResponse {
    let mut respData = GenericResp::<Vec<OrderMessage>> {
        message: "".to_string(),
        server_message: Some("".to_string()),
        data: None,
    };
    println!("new req");

    let claim = match claim {
        Some(claim) => claim,
        None => {
            respData.message = "Unauthorized".to_string();

            return HttpResponse::Unauthorized().json(respData);
        }
    };

    let filter = doc! {"buy_order_id": info.id.to_owned()};
    let response = match OrderMessageService::get_message(&database.db, filter).await {
        Ok(data) => data,
        Err(err) => {
            log::error!(" error getting messages  {}", err.to_string());
            respData.data = None;
            respData.message = "Error creating message".to_string();
            respData.server_message = Some(err.to_string());
            return HttpResponse::BadRequest().json(respData);
        }
    };

    respData.data = Some(response);
    respData.message = "Ok".to_string();
    respData.server_message = None;
    return HttpResponse::Ok().json(respData);
}

// /////////////////////////////////////////////////////////////////////////////
//  sell_order_controller.rs (kept as comments, as in the original codebase)   //
// /////////////////////////////////////////////////////////////////////////////

// use std::{env, str::FromStr};
//
// use actix_web::{ get, post, web::{self, Data, ReqData}, HttpResponse};
// use actix_web_validator::Json;
// use bigdecimal::{num_bigint::BigInt, BigDecimal};
// use mongodb::bson::doc;
// use serde::Deserialize;
// use uuid::Uuid;
//
// use crate::orders::controller::escrow_to_user;
// use crate::orders::models::{CreateSellOrderReq, Currency, PaymentMethod, SellOrder, UpdateSellOrderReq};
// use crate::orders::service::{PaymentMethodService, SellOrderService};
// use crate::shared::auth::Claims;
// use crate::shared::db::MongoService;
// use crate::shared::formatter;
// use crate::shared::response::{GenericResp, Response};
// use crate::shared::tcp::send_to_tcp_server;
// use crate::shared::vcrypto::TransferReq;
// use crate::system::service::SystemService;
//
//
//
//
// #[post("/sell")]
// pub async fn create_sell_order(
//
//     database:Data<MongoService>,
//     new_order:Json<CreateSellOrderReq>,
//     claim:Option<ReqData<Claims>>
//     )->HttpResponse{
//         let mut respData = GenericResp::<SellOrder>{
//             message:"".to_string(),
//             server_message: Some("".to_string()),
//             data: Some(SellOrder::default())
//         };
//     println!("new req");
//
//     let claim = match claim {
//         Some(claim)=>{claim},
//         None=>{
//             respData.message = "Unauthorized".to_string();
//
//             return HttpResponse::Unauthorized()
//                 .json(
//                     respData
//                 )
//         }
//     };
//
//     // take coins from users wallet
//
//
//
//
//     // send message to the kuracoin blockchain to create new user
//     let kura_coin_server_ip = match  env::var("KURACOIN_SERVER_ID"){
//         Ok(data)=>{data.to_owned()},
//         Err(err)=>{
//             println!("{}", err.to_string());
//             respData.message = "Error connecting to blockchain".to_string();
//             respData.server_message =Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);
//         }
//     };
//
//     let escrow_wallet = match  env::var("ESCROW_WALLET"){
//         Ok(data)=>{data.to_owned()},
//         Err(err)=>{
//             println!("{}", err.to_string());
//             respData.message = "Error connecting to blockchain".to_string();
//             respData.server_message =Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);
//         }
//     };
//
//     let message_data = match serde_json::to_string(&TransferReq{
//         sender: new_order.wallet_address.to_owned(),
//         receiver: escrow_wallet,
//         amount: new_order.amount.to_owned(),
//         transaction_id: Uuid::new_v4().to_string(),
//         sender_password: new_order.password.to_owned()
//     }){
//         Ok(data)=>{data},
//         Err(err)=>{
//             println!("{}", err.to_string());
//             respData.message = "Error persing data".to_string();
//             respData.server_message =Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);
//         }
//     };
//     let message = formatter::Formatter::request_formatter(
//         "Transfer".to_string(),
//         message_data,
//         "".to_string(),
//         "".to_string(),
//         "0".to_string());
//
//     let m = message.clone();
//     let ip = kura_coin_server_ip.clone();
//     let result = web::block(move || send_to_tcp_server(m,ip  )).await;
//     let response_string =match result {
//         Ok(data)=>{
//             match data {
//                 Ok(data)=>{data},
//                 Err(err)=>{
//                     println!("{}", err.to_string());
//                     respData.message = "Error persing data".to_string();
//                     respData.server_message =Some(err.to_string());
//                     respData.data = None;
//                     return HttpResponse::BadRequest().json(respData);
//                 }
//             }
//         },
//         Err(err)=>{
//             println!("{}", err.to_string());
//             respData.message = "Error persing data".to_string();
//             respData.server_message =Some(err.to_string());
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);
//         }
//     };
//
//     let resp_data: Vec<&str>= response_string.split('\n').collect();
//     let code = match resp_data.get(0){
//         Some(data)=>{data},
//         None=>{
//             respData.message = "Error with blockchain response data".to_string();
//             respData.server_message =None;
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);
//         }
//     };
//
//     if(*code != "1"){
//         // blockchain request failed
//         respData.message = "Failed transfer on the blockchain".to_string();
//             respData.server_message =match resp_data.get(1){
//                 Some(d)=>{Some(d.to_string())},
//                 None=>{None}
//             };
//             respData.data = None;
//             return HttpResponse::BadRequest().json(respData);
//     }
//
//     // get system info
//     let system_data = match SystemService::get_system_data(&database.db).await{
//         Ok(data)=>{
//             match data {
//                 Some(data)=>{data},
//                 None=>{
//                     respData.data = None;
//                     respData.message = "No Price data set".to_string();
//                     respData.server_message = None;
//                     return HttpResponse::InternalServerError().json(respData);
//                 }
//             }
//         },
//         Err(err)=>{
//             respData.data = None;
//             respData.message = "Error getting system data".to_string();
//             respData.server_message = Some(err.to_string());
//             return HttpResponse::InternalServerError().json(respData);
//         }
//     };
//
//     // amount bigdeci
//     // let amount = BigDecimal::from_str(new_order.amount);
//
//     let sell_order = SellOrder{
//         id: Uuid::new_v4().to_string(),
//         user_name: claim.user_name.clone(),
//         buy_orders_id : vec![],
//         buy_orders : None,
//         amount : new_order.amount.to_owned(),
//         min_amount: new_order.min_amount.to_owned(),
//         max_amount: new_order.amount.to_owned(),
//         is_closed: false,
//         created_at: chrono::offset::Utc::now().to_string(),
//         currency: new_order.currency.to_owned(),
//         updated_at: Some(chrono::offset::Utc::now().to_string()),
//         payment_method: new_order.payment_method.to_owned(),
//         payment_method_id: new_order.payment_method_id.to_owned(),
//         payment_method_data: None,
//         wallet_address: new_order.wallet_address.to_owned(),
//         phone_number: Some(new_order.phone_number.to_owned()),
//         price:system_data.price
//     };
//
//     // validaate the payment method id
//     match PaymentMethodService::get_user_payment_method_by_id(&database.db,
//          new_order.payment_method_id.to_owned()).await{
//             Ok(_)=>{
//
//             },
//             Err(err)=>{
//                 log::error!("Error getting payment method data {}", err);
//                 respData.message = "Error getting payment method data ".to_string();
//                 respData.server_message = Some(err.to_string());
//                 respData.data =None;
//                 return HttpResponse::BadRequest().json(
//                   respData
//
//                 )
//             }
//          }
//
//     // save order
//     match SellOrderService::create_sell_order(&database.db, &sell_order).await{
//
//         Ok(_)=>{},
//         Err(err)=>{
//             log::error!("Error creating sell order {}", err);
//             respData.message = "Error creating ".to_string();
//             respData.server_message = Some(err.to_string());
//             respData.data =None;
//             return HttpResponse::InternalServerError().json(
//               respData
//
//             )
//         }
//     };
//
//
//
//
//     respData.data = Some(sell_order);
//     respData.message = "Created".to_string();
//     respData.server_message = None;
//     return HttpResponse::Ok().json(
//         respData
//     )
//
// }
//
//
//
//
// #[get("/my_orders")]
// pub async fn get_my_sell_orders(
//
//     database:Data<MongoService>,
//     claim:Option<ReqData<Claims>>
//     )->HttpResponse
//     {
//         let mut respData = GenericResp::<Vec<SellOrder>>{
//             message:"".to_string(),
//             server_message: Some("".to_string()),
//             data: None
//         };
//
//         let claim = match claim {
//             Some(claim)=>{claim},
//             None=>{
//                 respData.data = None;
//                 respData.message = "Unauthorized".to_string();
//                 respData.server_message = None;
//                 return HttpResponse::Unauthorized()
//                     .json(
//                         respData
//                     )
//             }
//         };
//
//
//         let orders = match SellOrderService::get_all_sell_order_by_username(&database.db, claim.user_name.clone()).await{
//             Ok(data)=>{data},
//             Err(err)=>{
//                 respData.data = None;
//                 respData.server_message = Some(err.to_string());
//                 respData.message = "Error getting sell order".to_string();
//
//                 return HttpResponse::InternalServerError().json(
//                   respData
//                 )
//             }
//         };
//
//
//         respData.data = Some(orders);
//         respData.message = "ok".to_string();
//         respData.server_message = None;
//
//         return HttpResponse::Ok().json(
//             respData
//         )
//
// }
//
//
//
// #[derive(Deserialize)]
// struct GetSingleSellOrderPath {
//     id: String,
// }
//
// #[get("/single/{id}")]
// pub async fn get_single_sell_order(
//
//     database:Data<MongoService>,
//     claim:Option<ReqData<Claims>>,
//     inf: web::Path<GetSingleSellOrderPath>
// )->HttpResponse
// {
//     let mut respData = GenericResp::<SellOrder>{
//         message:"".to_string(),
//         server_message: Some("".to_string()),
//         data: None
//     };
//     // get claims
//     let claim = match claim {
//         Some(claim)=>{claim},
//         None=>{
//             return HttpResponse::Unauthorized()
//                 .json(Response{message:"Not authorized".to_string()})
//         }
//     };
//
//     // get sell order
//     let order = match SellOrderService::get_sell_order_by_id(&database.db, inf.id.to_owned()).await{
//         Ok(data)=>{data},
//         Err(err)=>{
//             respData.message = "Error getting sell order".to_string();
//             respData.server_message = Some(err.to_string());
//             return HttpResponse::Ok().json(respData)
//         }
//     };
//
//
//     let mut is_done = true;
//     // check if sell order is completed
//     match order.buy_orders.clone() {
//         Some(data)=>{
//             for buy_order in data {
//                 if !(buy_order.is_buyer_confirmed && buy_order.is_seller_confirmed) {
//                   is_done = false;
//                 }
//             }
//         },
//         None=>{
//
//         }
//
//     }
//     if is_done && order.amount == BigDecimal::from(BigInt::from(0)) {
//         // close the order
//         let mut n_order = order.clone();
//         n_order.is_closed = true;
//         SellOrderService::update(&database.db, &n_order).await;
//     }
//
//     respData.data = Some(order);
//
//     return HttpResponse::Ok().json(respData)
//
// }
//
//
//
// #[get("/cancel/{id}")]
// pub async fn cancel_sell_order(
//
//     database:Data<MongoService>,
//     claim:Option<ReqData<Claims>>,
//     inf: web::Path<GetSingleSellOrderPath>
// )->HttpResponse
// {
//     let mut respData = GenericResp::<SellOrder>{
//         message:"".to_string(),
//         server_message: Some("".to_string()),
//         data: None
//     };
//    // get claims
//     let claim = match claim {
//         Some(claim)=>{claim},
//         None=>{
//             return HttpResponse::Unauthorized()
//                 .json(Response{message:"Not authorized".to_string()})
//         }
//     };
//
//     // get sell order
//     println!("{}", inf.id);
//     let mut order = match SellOrderService::get_sell_order_by_id(&database.db, inf.id.to_owned()).await{
//         Ok(data)=>{data},
//         Err(err)=>{
//             respData.message = "Error getting sell order".to_string();
//             respData.data = None;
//             respData.server_message = Some(err.to_string());
//
//             return HttpResponse::BadRequest().json(
//                 respData
//             )
//         }
//     };
//
//     // check of any of the buy orders is still active
//     match order.buy_orders.clone() {
//         Some(data)=>{
//             for buy_order in data {
//                 if !(buy_order.is_buyer_confirmed && buy_order.is_seller_confirmed) {
//                      // check if the buy order is has been cancelled
//                      if (buy_order.is_canceled){
//                         continue;
//                     }
//                     respData.message = "There is still an open buy order".to_string();
//                     respData.data = None;
//                     respData.server_message = None;
//
//                     return HttpResponse::BadRequest().json(respData)
//                 }
//
//
//             }
//         },
//         None=>{
//
//         }
//
//     }
//
//     order.is_closed = true;
//
//     match SellOrderService::update(&database.db, &order).await{
//         Ok(_)=>{},
//         Err(err)=>{
//             println!("{}", err.to_string());
//             respData.message = "error updating sell order".to_string();
//             respData.data = None;
//             respData.server_message = Some(err.to_string());
//
//             return HttpResponse::InternalServerError().json(respData)  ;
//         }
//     }
//
//     match escrow_to_user(order.wallet_address.to_owned(), order.amount.to_owned()).await{
//         Ok(_)=>{
//             // update price data
//         },
//         Err(err)=>{
//             println!("{}", err.to_string());
//             respData.message = "error moving coins to wallet".to_string();
//             respData.data = None;
//             respData.server_message = Some(err.to_string());
//
//             return HttpResponse::InternalServerError().json(respData);
//         }
//     };
//
//     // move the remaining funds back to the user wallet
//
//
//     respData.data = Some(order);
//     respData.message = "Ok".to_string();
//     respData.server_message = None;
//
//     return HttpResponse::Ok().json(respData)
// }
//
//
// #[post("/update/{id}")]
// pub async fn update_sell_order(
//
//     database:Data<MongoService>,
//     claim:Option<ReqData<Claims>>,
//     new_order:Json<UpdateSellOrderReq>,
//     inf: web::Path<GetSingleSellOrderPath>
// )->HttpResponse
// {
//     let mut respData = GenericResp::<SellOrder>{
//         message:"".to_string(),
//         server_message: None,
//         data: None
//     };
//
//     // get claims
//     // get claims
//     let claim = match claim {
//         Some(claim)=>{claim},
//         None=>{
//             respData.message = "Unauthorized".to_string();
//             return HttpResponse::Unauthorized()
//                 .json(
//                     respData
//                 )
//         }
//     };
//
//     // get sell order
//     let mut sell_order = match SellOrderService::get_sell_order_by_id(&database.db, inf.id.to_owned()).await{
//         Ok(data)=>{data},
//         Err(err)=>{
//             respData.message = "error getting data".to_string();
//             respData.data = None;
//             respData.server_message = Some(err.to_string());
//             return HttpResponse::BadRequest().json(respData)
//         }
//     };
//
//     // check if user owns the order
//     if sell_order.user_name != claim.user_name{
//         respData.message = "Unauthorized".to_string();
//         respData.data = None;
//         respData.server_message = None;
//         return HttpResponse::Unauthorized().json(respData)
//     }
//
//     // update data
//     if new_order.currency.is_some(){
//         sell_order.currency = new_order.currency.to_owned().unwrap()
//     }
//     if new_order.max_amount.is_some(){
//         sell_order.max_amount = new_order.max_amount.to_owned().unwrap()
//     }
//     if new_order.min_amount.is_some(){
//         sell_order.min_amount = new_order.min_amount.to_owned().unwrap()
//     }
//
//     // update on database
//     match SellOrderService::update(&database.db, &sell_order).await {
//         Ok(_)=>{},
//         Err(err)=>{
//             respData.data = None;
//             respData.server_message = Some(err.to_string());
//             respData.message = "Error updating data".to_string();
//
//             return HttpResponse::BadRequest().json(respData)
//         }
//     }
//
//     respData.message = "Ok".to_string();
//     respData.data = None;
//     respData.server_message = None;
//
//     return HttpResponse::Ok().json(respData)
// }
//
//
//
//
// #[get("/open_orders")]
// pub async fn get_all_open_sell_orders(
//
//     database:Data<MongoService>,
//     claim:Option<ReqData<Claims>>
//     )->HttpResponse
//     {
//         let mut respData = GenericResp::<Vec<SellOrder>>{
//             message:"".to_string(),
//             server_message: Some("".to_string()),
//             data: None
//         };
//
//         let claim = match claim {
//             Some(claim)=>{claim},
//             None=>{
//                 respData.data = None;
//                 respData.message = "Unauthorized".to_string();
//                 respData.server_message = None;
//                 return HttpResponse::Unauthorized()
//                     .json(
//                         respData
//                     )
//             }
//         };
//
//
//         let orders = match SellOrderService::get_sell_order_by_filter(&database.db, doc! {"is_closed":false}).await{
//             Ok(data)=>{data},
//             Err(err)=>{
//                 respData.data = None;
//                 respData.server_message = Some(err.to_string());
//                 respData.message = "Error ".to_string();
//
//                 return HttpResponse::InternalServerError().json(
//                   respData
//                 )
//             }
//         };
//
//
//         respData.data = Some(orders);
//         respData.message = "ok".to_string();
//         respData.server_message = None;
//
//         return HttpResponse::Ok().json(
//             respData
//         )
//
// }