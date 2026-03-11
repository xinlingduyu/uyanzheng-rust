use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
  host: Option<String>,
  port: Option<u16>,
  password: Option<String>,
  db: Option<u8>,
  prefix: Option<String>,
  timeout_ms: Option<u64>,
  max_connections: Option<u32>,
}

impl RedisConfig {
  pub fn host(&self) -> &str {
      self.host.as_deref().unwrap_or("127.0.0.1")
  }

  pub fn port(&self) -> u16 {
      self.port.unwrap_or(6379)
  }

  pub fn password(&self) -> Option<&str> {
      self.password.as_deref()
  }

  pub fn db(&self) -> u8 {
      self.db.unwrap_or(0)
  }

  pub fn prefix(&self) -> &str {
      self.prefix.as_deref().unwrap_or("app_")
  }

  pub fn timeout_ms(&self) -> u64 {
      self.timeout_ms.unwrap_or(1000)
  }

  pub fn max_connections(&self) -> u32 {
      self.max_connections.unwrap_or(10)
  }

  // 构建Redis连接URL
  pub fn connection_url(&self) -> String {
      let mut url = "redis://".to_string();

      // 添加认证信息
      if let Some(password) = self.password() {
          url.push_str(":");
          url.push_str(password);
          url.push('@');
      }

      // 添加主机和端口
      url.push_str(self.host());
      url.push(':');
      url.push_str(&self.port().to_string());
      url.push('/');
      url.push_str(&self.db().to_string());

      url
  }
}
