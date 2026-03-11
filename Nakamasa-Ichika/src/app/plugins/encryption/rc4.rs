//! RC4 加解密实现
//! 一比一还原 PHP Ue/tools/encryption/rc4/rc4.php
//! 纯 Rust 实现, 完全跨平台

use super::Encryption;
use encoding_rs::GBK;

/// RC4 加密器
pub struct Rc4Encryption {
    password: Vec<u8>,
}

impl Rc4Encryption {
    /// 创建 RC4 加密实例
    /// password: 密码字符串
    pub fn new(password: &str) -> Self {
        // PHP 使用 UTF-8 转 GBK 编码
        let (password_gbk, _, _) = GBK.encode(password);
        Self {
            password: password_gbk.to_vec(),
        }
    }
    
    /// RC4 核心算法
    /// PHP: mi 方法
    fn rc4_crypt(&self, data: &[u8], _is_decrypt: bool) -> Vec<u8> {
        let pwd = &self.password;
        let pwd_len = pwd.len();
        let data_len = data.len();
        
        // 初始化 S-box 和 key
        let mut sbox: [u8; 256] = std::array::from_fn(|i| i as u8);
        let mut key: [u8; 256] = [0; 256];
        
        for i in 0..256 {
            key[i] = pwd[i % pwd_len];
        }
        
        // KSA (Key-Scheduling Algorithm)
        let mut j: u8 = 0;
        for i in 0..256 {
            j = j.wrapping_add(sbox[i]).wrapping_add(key[i]);
            sbox.swap(i, j as usize);
        }
        
        // PRGA (Pseudo-Random Generation Algorithm)
        let mut result = Vec::with_capacity(data_len);
        let mut i: u8 = 0;
        let mut j: u8 = 0;
        
        for &byte in data {
            i = i.wrapping_add(1);
            j = j.wrapping_add(sbox[i as usize]);
            sbox.swap(i as usize, j as usize);
            let k = sbox[(sbox[i as usize].wrapping_add(sbox[j as usize])) as usize];
            result.push(byte ^ k);
        }
        
        result
    }
}

impl Encryption for Rc4Encryption {
    /// 加密
    /// PHP: encode = mi(data, password, 0)
    /// 返回: hex 编码的密文
    fn encode(&self, data: &str) -> Result<String, String> {
        // UTF-8 转 GBK (与 PHP 一致)
        let (data_gbk, _, _) = GBK.encode(data);
        
        let encrypted = self.rc4_crypt(data_gbk.as_ref(), false);
        
        // PHP 返回 bin2hex
        Ok(hex::encode(encrypted))
    }
    
    /// 解密
    /// PHP: decode = mi(data, password, 1)
    /// 输入: hex 编码的密文
    fn decode(&self, data: &str) -> Result<String, String> {
        // hex 解码
        let encrypted = hex::decode(data)
            .map_err(|e| format!("Hex 解码失败: {:?}", e))?;
        
        let decrypted = self.rc4_crypt(&encrypted, true);
        
        // GBK 转 UTF-8
        let (decrypted_utf8, _, _) = GBK.decode(&decrypted);
        
        Ok(decrypted_utf8.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rc4_encode_decode() {
        let rc4 = Rc4Encryption::new("mypassword123456");
        
        let plaintext = "Hello, 世界!";
        let encrypted = rc4.encode(plaintext).unwrap();
        let decrypted = rc4.decode(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
}