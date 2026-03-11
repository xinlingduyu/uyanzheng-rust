use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MysqlConfig {
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    dbname: Option<String>,
    prefix: Option<String>,
    schema: Option<String>,
}

impl MysqlConfig {
    pub fn host(&self) -> &str {
        self.host.as_deref().unwrap_or("127.0.0.1")
    }
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(5432)
    }
    pub fn user(&self) -> &str {
        self.user.as_deref().unwrap_or("mysql")
    }
    pub fn password(&self) -> &str {
        self.password.as_deref().unwrap_or("mysql")
    }
    pub fn dbname(&self) -> &str {
        self.dbname.as_deref().unwrap_or("mysql")
    }
    pub fn schema(&self) -> &str {
        self.schema.as_deref().unwrap_or("public")
    }
}