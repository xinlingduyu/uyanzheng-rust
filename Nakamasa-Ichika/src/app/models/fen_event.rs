use sqlx::{FromRow, Type};

#[derive(Debug, Type, PartialEq, Clone)]
#[sqlx(type_name = "enum", rename_all = "lowercase")]
pub enum VipFreeStatus {
    Y,
    N,
}

#[derive(Debug, Type, PartialEq, Clone)]
#[sqlx(type_name = "enum", rename_all = "lowercase")]
pub enum EventStatus {
    On,
    Off,
}

#[derive(Debug, FromRow)]
pub struct FenEvent {
    pub id: i32,
    pub name: String,
    pub fen: Option<i32>,
    pub vip: Option<i64>,
    #[sqlx(rename = "vip_free")]
    pub vip_free: VipFreeStatus,
    pub appid: i32,
    pub state: EventStatus,
}
