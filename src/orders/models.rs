use std::collections::HashMap;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use strum_macros;
use validator::Validate;

// ---------- sell_order ----------

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SellOrder {
    pub id: String,
    pub user_name: String,
    pub buy_orders_id: Vec<String>,
    pub buy_orders: Option<Vec<BuyOrder>>,
    pub amount: BigDecimal,
    pub price: BigDecimal,
    pub min_amount: BigDecimal,
    pub max_amount: BigDecimal,
    pub is_closed: bool,
    pub currency: Currency,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub payment_method: PaymentMethod,
    pub payment_method_id: String,
    pub payment_method_data: Option<PaymentMethodData>,
    pub wallet_address: String,
    pub phone_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, strum_macros::Display, Default)]
pub enum Currency {
    #[default]
    NGN,
    USD,
    BTC,
    XRP,
}

// ---------- buy_order ----------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuyOrder {
    pub id: String,
    pub user_name: String,
    pub amount: BigDecimal,
    pub sell_order_id: String,
    pub is_seller_confirmed: bool,
    pub is_buyer_confirmed: bool,
    pub is_canceled: bool,
    pub is_reported: bool,
    pub created_at: String,
    pub updated_at: String,
    pub wallet_address: String,
}

pub type GetBuyOrderRes = BuyOrder;

// ---------- payment_method ----------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentMethodData {
    pub id: String,
    pub name: String,
    pub user_name: String,
    pub payment_method: PaymentMethod,
    pub account_name: String,
    pub account_number: String,
    pub bank_name: String,
    pub other: String,
    pub paypal_email: String,
    pub venmo_username: String,
    pub skrill_email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaypalPaymentMethod {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, strum_macros::Display, Default)]
pub enum PaymentMethod {
    #[default]
    Bank,
    Paypal,
    Skrill,
    Cash,
}

// ---------- message ----------

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OrderMessage {
    pub id: String,
    pub text: String,
    pub image: String,
    pub created_at: String,
    pub sender_user_name: String,
    pub receiver_user_name: String,
    pub buy_order_id: String,
}

// ---------- trade_contact ----------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeContact {
    pub id: String,
    pub phone_number: String,
}

// ---------- request DTOs ----------

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateSellOrderReq {
    pub amount: BigDecimal,
    pub min_amount: BigDecimal,
    pub currency: Currency,
    pub payment_method: PaymentMethod,
    pub payment_method_id: String,
    pub wallet_address: String,
    pub password: String,
    pub phone_number: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateSellOrderReq {
    pub amount: Option<BigDecimal>,
    pub min_amount: Option<BigDecimal>,
    pub max_amount: Option<BigDecimal>,
    pub currency: Option<Currency>,
    pub payment_method: Option<PaymentMethod>,
    pub payment_method_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateBuyOrderReq {
    pub amount: BigDecimal,
    pub sell_order_id: String,
    pub wallet_address: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePaymentMethodReq {
    pub payment_method: PaymentMethod,
    pub account_name: String,
    pub account_number: String,
    pub bank_name: String,
    pub other: String,
    pub paypal_email: String,
    pub venmo_username: String,
    pub skrill_email: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateOrderMessageReq {
    pub receiver_user_name: String,
    pub text: String,
    pub image: String,
    pub buy_order_id: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct GetAllOrderMessageReq {
    pub buy_order_id: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAccountDetailsReq {
    pub account_name: String,
    pub account_number: String,
    pub bank_name: String,
}