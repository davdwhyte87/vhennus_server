use std::error::Error;

use futures::StreamExt;
use mongodb::bson::{doc, from_document, Document};
use mongodb::results::InsertOneResult;
use mongodb::Database;

use crate::orders::models::{BuyOrder, OrderMessage, PaymentMethodData, SellOrder};

pub const SELL_ORDER_COLLECTION: &str = "SellOrder";
pub const BUY_ORDER_COLLECTION: &str = "BuyOrder";
pub const PAYMENT_METHOD_COLLECTION: &str = "PaymentMethodData";
pub const ORDER_MESSAGE_COLLECTION: &str = "OrderMessage";

pub struct SellOrderRepo;

impl SellOrderRepo {
    pub async fn create_sell_order(
        db: &Database,
        order: &SellOrder,
    ) -> Result<InsertOneResult, Box<dyn Error>> {
        let collection = db.collection::<SellOrder>(SELL_ORDER_COLLECTION);

        let res_sell_order = collection.insert_one(order).await;

        let res_order = match res_sell_order {
            Ok(data) => data,
            Err(err) => {
                log::error!(" error inserting data  {}", err.to_string());
                return Err(err.into());
            }
        };
        Ok(res_order)
    }

    pub async fn get_all_sell_order_by_username(
        db: &Database,
        userName: String,
    ) -> Result<Vec<SellOrder>, Box<dyn Error>> {
        let collection = db.collection::<SellOrder>(SELL_ORDER_COLLECTION);
        let lookup_2 = doc! {
            "$lookup":
               {
                  "from": "BuyOrder",
                  "localField": "buy_orders_id",
                  "foreignField": "id",
                  "as": "buy_orders"
               }
            };

        let lookup_1 = doc! {
            "$match":doc! {"user_name": userName}
        };

        let mut results = collection.aggregate(vec![lookup_1, lookup_2]).await?;
        let mut sell_orders: Vec<SellOrder> = Vec::new();
        while let Some(result) = results.next().await {
            let data: SellOrder = from_document(result?)?;
            sell_orders.push(data);
        }
        return Ok(sell_orders);
    }

    pub async fn get_sell_order_by_filter(
        db: &Database,
        filter: Document,
    ) -> Result<Vec<SellOrder>, Box<dyn Error>> {
        let collection = db.collection::<SellOrder>(SELL_ORDER_COLLECTION);
        let lookup_2 = doc! {
            "$lookup":
               {
                  "from": "BuyOrder",
                  "localField": "buy_orders_id",
                  "foreignField": "id",
                  "as": "buy_orders"
               }
            };
        let match_1 = doc! {
            "$match":filter
        };

        let mut results = match collection.aggregate(vec![match_1]).await {
            Ok(dd) => dd,
            Err(err) => {
                log::error!(" error with aggregation  {}", err.to_string());
                return Err(err.into());
            }
        };
        let mut sell_orders: Vec<SellOrder> = Vec::new();
        while let Some(result) = results.next().await {
            let data: SellOrder = match from_document(result?) {
                Ok(d) => d,
                Err(err) => {
                    log::error!(" error converting from document  {}", err.to_string());
                    return Err(err.into());
                }
            };
            sell_orders.push(data);
        }
        return Ok(sell_orders);
    }

    pub async fn get_sell_order_by_id(db: &Database, id: String) -> Result<SellOrder, Box<dyn Error>> {
        let collection = db.collection::<SellOrder>(SELL_ORDER_COLLECTION);
        let filter = doc! {"id":id};
        let lookup_2 = doc! {
            "$lookup":
               {
                  "from": "BuyOrder",
                  "localField": "buy_orders_id",
                  "foreignField": "id",
                  "as": "buy_orders"
               }
        };
        let lookup_3 = doc! {
            "$lookup":
               {
                  "from": "PaymentMethodData",
                  "localField": "payment_method_id",
                  "foreignField": "id",
                  "as": "payment_method_data"
               }
        };
        let pipeline = vec![
            doc! { "$match": filter },
            lookup_2,
            lookup_3,
            doc! {
                 "$unwind": "$payment_method_data"
            },
        ];

        let mut sell_orders: Vec<SellOrder> = Vec::new();
        let mut cursor = collection.aggregate(pipeline).await;
        match cursor {
            Ok(mut cursor) => {
                if let Some(result) = cursor.next().await {
                    match result {
                        Ok(res) => {
                            let data: SellOrder = from_document(res)?;
                            return Ok(data);
                        }
                        Err(err) => {
                            log::error!(" error with cursor  {}", err.to_string());
                            return Err(err.into());
                        }
                    };
                } else {
                    return Err(Box::from("Error getting data"));
                }
            }
            Err(err) => {
                log::error!(" error with aggregation {}", err.to_string());
                return Err(err.into());
            }
        };
    }

    pub async fn update(db: &Database, sell_order: &SellOrder) -> Result<(), Box<dyn Error>> {
        let filter = doc! {"id":sell_order.id.clone()};
        let collection = db.collection::<SellOrder>(SELL_ORDER_COLLECTION);
        let update_data = doc! {"$set":doc! {
            "buy_orders_id":sell_order.buy_orders_id.to_owned(),
            "amount":sell_order.amount.to_string(),
            "min_amount": sell_order.min_amount.to_string(),
            "max_amount": sell_order.max_amount.to_string(),
            "is_closed": sell_order.is_closed,
            "currency": sell_order.currency.to_string(),
            "updated_at": sell_order.updated_at.to_owned(),
            "payment_method": sell_order.payment_method.to_owned().to_string(),
            "payment_method_id": sell_order.payment_method_id.to_owned()
            }};

        let update_res = collection.update_one(filter, update_data).await;
        match update_res {
            Ok(_) => {}
            Err(err) => {
                log::error!(" error updating db {}", err.to_string());
                return Err(err.into());
            }
        }
        Ok(())
    }
}

pub struct BuyOrderRepo;

impl BuyOrderRepo {
    pub async fn create_buy_order(
        db: &Database,
        order: &BuyOrder,
    ) -> Result<InsertOneResult, Box<dyn Error>> {
        let collection = db.collection::<BuyOrder>(BUY_ORDER_COLLECTION);

        let res_sell_order = collection.insert_one(order).await;

        let res_order = match res_sell_order {
            Ok(data) => data,
            Err(err) => {
                log::error!(" error inserting into db  {}", err.to_string());
                return Err(err.into());
            }
        };
        Ok(res_order)
    }

    pub async fn get_all_buy_order_by_username(
        db: &Database,
        userName: String,
    ) -> Result<Vec<BuyOrder>, Box<dyn Error>> {
        let collection = db.collection::<BuyOrder>(BUY_ORDER_COLLECTION);
        let mut results = collection.find(doc! {"user_name":userName}).await?;
        let mut buy_orders: Vec<BuyOrder> = Vec::new();
        while let Some(result) = results.next().await {
            let data = result.unwrap();
            buy_orders.push(data);
        }
        return Ok(buy_orders);
    }

    pub async fn get_single_order_by_id(
        db: &Database,
        id: String,
    ) -> Result<Option<BuyOrder>, Box<dyn Error>> {
        let collection = db.collection::<BuyOrder>(BUY_ORDER_COLLECTION);
        let results = collection.find_one(doc! {"id":id}).await;

        match results {
            Ok(data) => return Ok(data),
            Err(err) => {
                log::error!(" error getting data from db  {}", err.to_string());
                return Err(err.into());
            }
        }
    }

    pub async fn update(db: &Database, buy_order: &BuyOrder) -> Result<(), Box<dyn Error>> {
        let filter = doc! {"id":buy_order.id.clone()};
        let collection = db.collection::<BuyOrder>(BUY_ORDER_COLLECTION);
        let update_data = doc! {"$set":doc! {
            "amount":buy_order.amount.to_owned().to_string(),
            "is_seller_confirmed":buy_order.is_seller_confirmed,
            "is_buyer_confirmed": buy_order.is_buyer_confirmed,
            "is_canceled": buy_order.is_canceled,
            "is_reported": buy_order.is_reported,
            "updated_at": chrono::offset::Utc::now().to_string(),
            }};

        let update_res = collection.update_one(filter, update_data).await;
        match update_res {
            Ok(_) => {}
            Err(err) => {
                log::error!(" error updating db {}", err.to_string());
                return Err(err.into());
            }
        }
        Ok(())
    }
}

pub struct PaymentMethodRepo;

impl PaymentMethodRepo {
    pub async fn create_payment_method(
        db: &Database,
        bank_payment: &PaymentMethodData,
    ) -> Result<InsertOneResult, Box<dyn Error>> {
        let collection = db.collection::<PaymentMethodData>(PAYMENT_METHOD_COLLECTION);

        let res_sell_order = collection.insert_one(bank_payment).await;

        let res_order = match res_sell_order {
            Ok(data) => data,
            Err(err) => {
                log::error!(" error inserting into db {}", err.to_string());
                return Err(err.into());
            }
        };
        Ok(res_order)
    }

    pub async fn get_user_payment_method_by_id(
        db: &Database,
        id: String,
    ) -> Result<Option<PaymentMethodData>, Box<dyn Error>> {
        let filter = doc! {"id":id};
        let collection = db.collection::<PaymentMethodData>(PAYMENT_METHOD_COLLECTION);
        let res = collection.find_one(filter).await;

        match res {
            Ok(data) => {
                return Ok(data);
            }
            Err(err) => {
                log::error!(" error getting data from db {}", err.to_string());
                return Err(err.into());
            }
        }
    }

    pub async fn get_all_user_payment_method_data(
        db: &Database,
        user_name: String,
    ) -> Result<Vec<PaymentMethodData>, Box<dyn Error>> {
        let filter = doc! {"user_name":user_name};
        let collection = db.collection::<PaymentMethodData>(PAYMENT_METHOD_COLLECTION);
        let res = collection.find(filter).await;
        let mut res_data: Vec<PaymentMethodData> = vec![];
        match res {
            Ok(mut data) => {
                while let Some(result) = data.next().await {
                    let payment_method: PaymentMethodData = match result {
                        Ok(d) => d,
                        Err(err) => {
                            return Err(err.into());
                        }
                    };
                    res_data.push(payment_method);
                }
                return Ok(res_data);
            }
            Err(err) => {
                log::error!(" error getting data from db {}", err.to_string());
                return Err(err.into());
            }
        }
    }

    pub async fn update_user_payment_method(
        db: &Database,
        user_name: String,
        payment_method: PaymentMethodData,
    ) -> Result<(), Box<dyn Error>> {
        let filter = doc! {"user_name":user_name};
        let update_data = doc! {
            "$set": doc! {
                "account_name": payment_method.account_name.to_owned(),
                "account_number": payment_method.account_number.to_owned(),
                "bank_name": payment_method.bank_name.to_owned(),
                "other": payment_method.other.to_owned(),
                "paypal_email": payment_method.paypal_email.to_owned(),
                "venmo_username": payment_method.venmo_username.to_owned(),
                "skril_email":payment_method.skrill_email.to_owned()
            }
        };
        let collection = db.collection::<PaymentMethodData>(PAYMENT_METHOD_COLLECTION);
        let res = collection.update_one(filter, update_data).await;

        match res {
            Ok(data) => {
                return Ok(());
            }
            Err(err) => {
                log::error!(" error updating db  {}", err.to_string());
                return Err(err.into());
            }
        }
    }

    pub async fn delete_user_payment_method(db: &Database, id: String) -> Result<(), Box<dyn Error>> {
        let filter = doc! {"id":id};
        let collection = db.collection::<PaymentMethodData>(PAYMENT_METHOD_COLLECTION);
        let res = collection.delete_one(filter).await;

        match res {
            Ok(data) => {
                return Ok(());
            }
            Err(err) => {
                log::error!(" error deleting from db {}", err.to_string());
                return Err(err.into());
            }
        }
    }
}

pub struct OrderMessageRepo;

impl OrderMessageRepo {
    pub async fn create_message(
        db: &Database,
        message: &OrderMessage,
    ) -> Result<InsertOneResult, Box<dyn Error>> {
        let collection = db.collection::<OrderMessage>(ORDER_MESSAGE_COLLECTION);

        let result = collection.insert_one(message).await;

        let data = match result {
            Ok(data) => data,
            Err(err) => {
                log::error!(" error inserting into db  {}", err.to_string());
                return Err(err.into());
            }
        };
        Ok(data)
    }

    pub async fn get_message(db: &Database, filter: Document) -> Result<Vec<OrderMessage>, Box<dyn Error>> {
        let collection = db.collection::<OrderMessage>(ORDER_MESSAGE_COLLECTION);

        let results = collection.find(filter).await;
        let mut messages: Vec<OrderMessage> = Vec::new();
        let mut results = match results {
            Ok(dd) => dd,
            Err(err) => {
                log::error!(" error getting data from db {}", err.to_string());
                return Err(err.into());
            }
        };

        while let Some(result) = results.next().await {
            let data = match result {
                Ok(data) => data,
                Err(err) => {
                    log::error!(" error with cursor  {}", err.to_string());
                    return Err(err.into());
                }
            };
            messages.push(data);
        }

        Ok(messages)
    }
}