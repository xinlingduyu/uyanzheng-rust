//! 配置敏感信息加密模块
//! 
//! 使用 AES-256-CBC 变种算法加密配置中的敏感信息
//! - 密钥派生：使用 SHA3-256 从 app.code 派生加密密钥
//! - 加密模式：AES-256-CBC with PKCS7 padding
//! - 输出格式：Base64 编码，前缀 "enc:"

use aes::cipher::{block_padding::Pkcs7, KeyIvInit, BlockEncryptMut, BlockDecryptMut};
use aes::Aes256;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use cbc::{Encryptor, Decryptor};
use sha3::{Digest, Sha3_256};
use thiserror::Error;

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

/// 加密前缀，用于标识加密字段
pub const ENCRYPTED_PREFIX: &str = "enc:";

/// 加密错误类型
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("无效的密钥长度")]
    InvalidKeyLength,
    #[error("加密失败: {0}")]
    EncryptionFailed(String),
    #[error("解密失败: {0}")]
    DecryptionFailed(String),
    #[error("无效的加密数据格式")]
    InvalidFormat,
    #[error("Base64解码失败: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

/// 从密钥字符串派生 AES-256 密钥和 IV
/// 
/// 使用 SHA3-256 进行密钥派生：
/// - 前32字节作为 AES 密钥
/// - 后16字节作为 CBC IV（取前16字节）
fn derive_key_and_iv(secret: &str) -> ([u8; 32], [u8; 16]) {
    let mut hasher = Sha3_256::new();
    
    // 派生密钥
    hasher.update(secret.as_bytes());
    hasher.update(b":key:v1");
    let key_hash = hasher.finalize_reset();
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_hash);
    
    // 派生 IV
    hasher.update(secret.as_bytes());
    hasher.update(b":iv:v1");
    let iv_hash = hasher.finalize();
    let mut iv = [0u8; 16];
    iv.copy_from_slice(&iv_hash[..16]);
    
    (key, iv)
}

/// 加密敏感数据
/// 
/// # 参数
/// - `plaintext`: 明文数据
/// - `secret`: 加密密钥（app.code）
/// 
/// # 返回
/// 加密后的数据，格式为 "enc:{base64_encoded_ciphertext}"
pub fn encrypt(plaintext: &str, secret: &str) -> Result<String, CryptoError> {
    if secret.len() < 16 {
        return Err(CryptoError::InvalidKeyLength);
    }
    
    let (key, iv) = derive_key_and_iv(secret);
    let plaintext_bytes = plaintext.as_bytes();
    
    // AES-256-CBC 加密
    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
    let pt_len = plaintext_bytes.len();
    let ct_len = pt_len + 16 - (pt_len % 16);
    let mut buffer = vec![0u8; ct_len];
    buffer[..pt_len].copy_from_slice(plaintext_bytes);
    
    cipher.encrypt_padded_mut::<Pkcs7>(&mut buffer, pt_len)
        .map_err(|_e| CryptoError::EncryptionFailed("padding error".to_string()))?;
    
    // Base64 编码并添加前缀
    let encoded = BASE64.encode(&buffer);
    Ok(format!("{}{}", ENCRYPTED_PREFIX, encoded))
}

/// 解密敏感数据
/// 
/// # 参数
/// - `ciphertext`: 加密数据，格式为 "enc:{base64_encoded_ciphertext}"
/// - `secret`: 解密密钥（app.code）
/// 
/// # 返回
/// 解密后的明文数据
pub fn decrypt(ciphertext: &str, secret: &str) -> Result<String, CryptoError> {
    if secret.len() < 16 {
        return Err(CryptoError::InvalidKeyLength);
    }
    
    // 检查并移除前缀
    let encoded = if ciphertext.starts_with(ENCRYPTED_PREFIX) {
        &ciphertext[ENCRYPTED_PREFIX.len()..]
    } else {
        return Err(CryptoError::InvalidFormat);
    };
    
    // Base64 解码
    let mut buffer = BASE64.decode(encoded)?;
    
    let (key, iv) = derive_key_and_iv(secret);
    
    // AES-256-CBC 解密
    let cipher = Aes256CbcDec::new(&key.into(), &iv.into());
    let decrypted = cipher.decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|_e| CryptoError::DecryptionFailed("unpad error".to_string()))?;
    
    // 转換為字符串
    String::from_utf8(decrypted.to_vec())
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
}

/// 检查值是否已加密
#[inline]
pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(ENCRYPTED_PREFIX)
}

/// 条件解密：如果值已加密则解密，否则返回原值
/// 
/// # 参数
/// - `value`: 可能加密的值
/// - `secret`: 解密密钥
/// 
/// # 返回
/// 解密后的值或原值
pub fn decrypt_if_needed(value: &str, secret: &str) -> Result<String, CryptoError> {
    if is_encrypted(value) {
        decrypt(value, secret)
    } else {
        Ok(value.to_string())
    }
}

/// 安全清零内存
/// 
/// 用于在解密后清除内存中的敏感数据
#[inline]
pub fn secure_zero(data: &mut [u8]) {
    use std::ptr::write_volatile;
    for byte in data.iter_mut() {
        unsafe { write_volatile(byte, 0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt() {
        let secret = "test_secret_key_12345678";
        let plaintext = "my_database_password";
        
        let encrypted = encrypt(plaintext, secret).unwrap();
        assert!(is_encrypted(&encrypted));
        
        let decrypted = decrypt(&encrypted, secret).unwrap();
        assert_eq!(plaintext, decrypted);
    }
    
    #[test]
    fn test_decrypt_if_needed() {
        let secret = "test_secret_key_12345678";
        
        // 加密值
        let encrypted = encrypt("password", secret).unwrap();
        let result = decrypt_if_needed(&encrypted, secret).unwrap();
        assert_eq!(result, "password");
        
        // 非加密值
        let result = decrypt_if_needed("plain_password", secret).unwrap();
        assert_eq!(result, "plain_password");
    }
    
    #[test]
    fn test_different_secrets() {
        let secret1 = "secret_one_12345678";
        let secret2 = "secret_two_12345678";
        let plaintext = "sensitive_data";
        
        let encrypted = encrypt(plaintext, secret1).unwrap();
        
        // 使用相同密钥可以解密
        let decrypted = decrypt(&encrypted, secret1).unwrap();
        assert_eq!(plaintext, decrypted);
        
        // 使用不同密钥无法正确解密
        let result = decrypt(&encrypted, secret2);
        assert!(result.is_err() || result.unwrap() != plaintext);
    }
}