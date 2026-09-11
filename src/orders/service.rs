use std::error::Error;

use mongodb::bson::Document;
use mongodb::results::InsertOneResult;
use mongodb::Database;

use crate::orders::models::{BuyOrder, OrderMessage, PaymentMethodData, SellOrder};
use crate::orders::repository::{
    BuyOrderRepo, OrderMessageRepo, PaymentMethodRepo, SellOrderRepo,
};

pub struct SellOrderService;

impl SellOrderService {
    pub async fn create_sell_order(
        db: &Database,
        order: &SellOrder,
    ) -> Result<InsertOneResult, Box<dyn Error>> {
        SellOrderRepo::create_sell_order(db, order).await
    }

    pub async fn get_all_sell_order_by_username(
        db: &Database,
        userName: String,
    ) -> Result<Vec<SellOrder>, Box<dyn Error>> {
        SellOrderRepo::get_all_sell_order_by_username(db, userName).await
    }

    pub async fn get_sell_order_by_filter(
        db: &Database,
        filter: Document,
    ) -> Result<Vec<SellOrder>, Box<dyn Error>> {
        SellOrderRepo::get_sell_order_by_filter(db, filter).await
    }

    pub async fn get_sell_order_by_id(
        db: &Database,
        id: String,
    ) -> Result<SellOrder, Box<dyn Error>> {
        SellOrderRepo::get_sell_order_by_id(db, id).await
    }

    pub async fn update(db: &Database, sell_order: &SellOrder) -> Result<(), Box<dyn Error>> {
        SellOrderRepo::update(db, sell_order).await
    }
}

pub struct BuyOrderService;

impl BuyOrderService {
    pub async fn create_buy_order(
        db: &Database,
        order: &BuyOrder,
    ) -> Result<InsertOneResult, Box<dyn Error>> {
        BuyOrderRepo::create_buy_order(db, order).await
    }

    pub async fn get_all_buy_order_by_username(
        db: &Database,
        userName: String,
    ) -> Result<Vec<BuyOrder>, Box<dyn Error>> {
        BuyOrderRepo::get_all_buy_order_by_username(db, userName).await
    }

    pub async fn get_single_order_by_id(
        db: &Database,
        id: String,
    ) -> Result<Option<BuyOrder>, Box<dyn Error>> {
        BuyOrderRepo::get_single_order_by_id(db, id).await
    }

    pub async fn update(db: &Database, buy_order: &BuyOrder) -> Result<(), Box<dyn Error>> {
        BuyOrderRepo::update(db, buy_order).await
    }
}

pub struct PaymentMethodService;

impl PaymentMethodService {
    pub async fn create_payment_method(
        db: &Database,
        bank_payment: &PaymentMethodData,
    ) -> Result<InsertOneResult, Box<dyn Error>> {
        PaymentMethodRepo::create_payment_method(db, bank_payment).await
    }

    pub async fn get_user_payment_method_by_id(
        db: &Database,
        id: String,
    ) -> Result<Option<PaymentMethodData>, Box<dyn Error>> {
        PaymentMethodRepo::get_user_payment_method_by_id(db, id).await
    }

    pub async fn get_all_user_payment_method_data(
        db: &Database,
        user_name: String,
    ) -> Result<Vec<PaymentMethodData>, Box<dyn Error>> {
        PaymentMethodRepo::get_all_user_payment_method_data(db, user_name).await
    }

    pub async fn update_user_payment_method(
        db: &Database,
        user_name: String,
        payment_method: PaymentMethodData,
    ) -> Result<(), Box<dyn Error>> {
        PaymentMethodRepo::update_user_payment_method(db, user_name, payment_method).await
    }

    pub async fn delete_user_payment_method(
        db: &Database,
        id: String,
    ) -> Result<(), Box<dyn Error>> {
        PaymentMethodRepo::delete_user_payment_method(db, id).await
    }
}

pub struct OrderMessageService;

impl OrderMessageService {
    pub async fn create_message(
        db: &Database,
        message: &OrderMessage,
    ) -> Result<InsertOneResult, Box<dyn Error>> {
        OrderMessageRepo::create_message(db, message).await
    }

    pub async fn get_message(
        db: &Database,
        filter: Document,
    ) -> Result<Vec<OrderMessage>, Box<dyn Error>> {
        OrderMessageRepo::get_message(db, filter).await
    }
}