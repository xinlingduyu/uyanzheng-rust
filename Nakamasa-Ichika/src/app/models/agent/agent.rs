use sqlx::FromRow;
use serde_json::Value;
use super::super::common::enums::{CashWay, AgentStatus};

#[derive(Debug, FromRow)]
pub struct Agent {
    pub id: i32,
    pub aggid: i32,
    pub uid: i32,
    pub note: Option<String>,
    pub pay_divide: Option<i32>,
    pub km_discount: Option<i32>,
    pub money: Option<f64>,
    pub cash_name: Option<String>,
    pub cash_account: Option<String>,
    pub cash_way: Option<CashWay>,
    pub authority: Option<Value>,
    pub time: i32,
    pub state: AgentStatus,
    pub appid: i32,
}

// 为UAgent结构体添加业务方法
impl Agent {
    pub fn is_active(&self) -> bool {
        self.state == AgentStatus::On
    }
    
    pub fn has_withdrawal_info(&self) -> bool {
        self.cash_name.is_some() && self.cash_account.is_some() && self.cash_way.is_some()
    }
}
