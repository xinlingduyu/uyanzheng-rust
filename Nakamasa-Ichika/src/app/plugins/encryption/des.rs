//! DES 加解密实现
//! 一比一还原 PHP Ue/tools/encryption/des/des.php
//! 支持 ECB 和 CBC 模式

use super::Encryption;
use des::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit, generic_array::GenericArray};

type DesEcbEnc = cbc::Encryptor<des::Des>;
type DesEcbDec = cbc::Decryptor<des::Des>;

/// DES 加密器
pub struct DesEncryption {
    key: [u8; 8],
    iv: Option<[u8; 8]>,
}

impl DesEncryption {
    /// 创建 DES 加密实例
    /// key: 8字节密钥
    /// iv: 8字节初始化向量 (可选, 为 None 时使用 ECB 模式)
    pub fn new(key: &str, iv: Option<&str>) -> Self {
        let mut key_arr = [0u8; 8];
        let key_bytes = key.as_bytes();
        for i in 0..8.min(key_bytes.len()) {
            key_arr[i] = key_bytes[i];
        }
        
        let iv_arr = iv.map(|iv_str| {
            let mut arr = [0u8; 8];
            let iv_bytes = iv_str.as_bytes();
            for i in 0..8.min(iv_bytes.len()) {
                arr[i] = iv_bytes[i];
            }
            arr
        });
        
        Self {
            key: key_arr,
            iv: iv_arr,
        }
    }
    
    /// 创建全零 IV (用于 ECB 模式)
    fn zero_iv() -> [u8; 8] {
        [0u8; 8]
    }
}

impl Encryption for DesEncryption {
    /// 加密
    /// PHP: openssl_encrypt($str, 'des-ecb', $key) 或 openssl_encrypt($str, 'des-cbc', $key, OPENSSL_RAW_DATA, $iv)
    /// 返回: hex 编码的密文
    fn encode(&self, data: &str) -> Result<String, String> {
        let key = GenericArray::from(self.key);
        let iv_bytes = self.iv.unwrap_or_else(Self::zero_iv);
        let iv = GenericArray::from(iv_bytes);
        
        let cipher = DesEcbEnc::new(&key, &iv);
        
        let mut buf = vec![0u8; data.len() + 8];
        let ct_len = cipher
            .encrypt_padded_b2b_mut::<Pkcs7>(data.as_bytes(), &mut buf)
            .map_err(|e| format!("DES 加密失败: {:?}", e))?
            .len();
        buf.truncate(ct_len);
        
        // PHP 使用 bin2hex
        Ok(hex::encode(&buf))
    }
    
    /// 解密
    /// PHP: openssl_decrypt($str, 'des-ecb', $key) 或 openssl_decrypt(hex2bin($str), 'des-cbc', $key, OPENSSL_RAW_DATA, $iv)
    /// 输入: hex 编码的密文
    fn decode(&self, data: &str) -> Result<String, String> {
        // hex 解码
        let encrypted = hex::decode(data)
            .map_err(|e| format!("Hex 解码失败: {:?}", e))?;
        
        let key = GenericArray::from(self.key);
        let iv_bytes = self.iv.unwrap_or_else(Self::zero_iv);
        let iv = GenericArray::from(iv_bytes);
        
        let cipher = DesEcbDec::new(&key, &iv);
        
        let mut buf = encrypted.clone();
        let decrypted = cipher
            .decrypt_padded_mut::<Pkcs7>(&mut buf)
            .map_err(|e| format!("DES 解密失败: {:?}", e))?;
        
        String::from_utf8(decrypted.to_vec())
            .map_err(|e| format!("UTF-8 转换失败: {:?}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_des_ecb() {
        let des = DesEncryption::new("12345678", None);
        
        let plaintext = "Hello!";
        let encrypted = des.encode(plaintext).unwrap();
        let decrypted = des.decode(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_des_cbc() {
        let des = DesEncryption::new("12345678", Some("87654321"));
        
        let plaintext = "Hello, World!";
        let encrypted = des.encode(plaintext).unwrap();
        let decrypted = des.decode(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
}