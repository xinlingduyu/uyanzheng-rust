use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct AppExtend {
    pub id: i32,
    pub name: String,
    pub var_key: String,
    pub var_val: String, // TEXT 类型映射到 String
    pub appid: Option<i32>,
}
