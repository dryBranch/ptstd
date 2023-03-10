/// 简单的AES加密与解密接口
/// 加密与解密使用同样的参数
pub trait AESCipher {
    fn encode(&mut self, input: &[u8]) -> Result<Vec<u8>, String>;
    fn decode(&mut self, input: &[u8]) -> Result<Vec<u8>, String>;
}

use crypto::{
    aessafe::{AesSafe256Decryptor, AesSafe256Encryptor},
    blockmodes::{CbcDecryptor, CbcEncryptor, DecPadding, EncPadding, PkcsPadding},
    buffer::{ReadBuffer, RefReadBuffer, RefWriteBuffer, WriteBuffer},
    symmetriccipher::{Decryptor, Encryptor},
};
use rand::RngCore;

/// 使用CBC模式、PkcsPadding、256位密钥
pub struct AESCryptor {
    key: Vec<u8>,
    iv: Vec<u8>,
    encryptor: CbcEncryptor<AesSafe256Encryptor, EncPadding<PkcsPadding>>,
    decryptor: CbcDecryptor<AesSafe256Decryptor, DecPadding<PkcsPadding>>,
}

impl TryFrom<&[u8]> for AESCryptor {
    type Error = String;

    fn try_from(key_iv: &[u8]) -> Result<Self, Self::Error> {
        if key_iv.len() != 48 {
            return Err("the len of key_iv is not 48".to_string());
        }
        Self::try_new_with(&key_iv[0..32], &key_iv[32..48])
    }
}

impl TryFrom<&Vec<u8>> for AESCryptor {
    type Error = String;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        AESCryptor::try_from(value.as_slice())
    }
}

impl Clone for AESCryptor {
    fn clone(&self) -> Self {
        Self::try_new_with(&self.key, &self.iv).unwrap()
    }
}

impl AESCryptor {
    pub fn try_new() -> Result<Self, String> {
        let mut r = rand::thread_rng();
        let mut key = [0u8; 32];
        r.fill_bytes(&mut key);
        let mut iv = [0u8; 16];
        r.fill_bytes(&mut iv);
        Self::try_new_with(&key, &iv)
    }

    pub fn try_new_with(key: &[u8], iv: &[u8]) -> Result<Self, String> {
        if key.len() != 32 {
            return Err("key is not 256 bits".to_string());
        }
        if iv.len() != 16 {
            return Err("iv is not 128 bits".to_string());
        }
        let aes_enc = crypto::aessafe::AesSafe256Encryptor::new(key);
        let aes_dec = crypto::aessafe::AesSafe256Decryptor::new(key);

        let enc = CbcEncryptor::new(aes_enc, PkcsPadding, iv.to_vec());
        let dec = CbcDecryptor::new(aes_dec, PkcsPadding, iv.to_vec());
        Ok(AESCryptor {
            key: key.into(),
            iv: iv.into(),
            encryptor: enc,
            decryptor: dec,
        })
    }

    pub fn to_key_iv_bytes(&self) -> Vec<u8> {
        self.key.iter().chain(self.iv.iter()).cloned().collect()
    }
}

impl AESCipher for AESCryptor {
    /// 由于填充规则16B为一组,但是整数会多填充一组
    /// 16B的数据加密后被填充了到32B
    /// 15B的数据加密后被填充了到16B
    /// 0B的数据还是0B
    fn encode(&mut self, input: &[u8]) -> Result<Vec<u8>, String> {
        self.encryptor.reset(&self.iv);
        let mut read_buf = RefReadBuffer::new(input);
        let mut buff = [0u8; 4096];
        let mut write_buf = RefWriteBuffer::new(&mut buff);
        let mut final_result: Vec<u8> = Vec::new();

        loop {
            let result = self
                .encryptor
                .encrypt(&mut read_buf, &mut write_buf, true)
                .map_err(|_e| "can't encrypt")?;
            final_result.extend(
                write_buf
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .copied(),
            );
            match result {
                crypto::buffer::BufferResult::BufferUnderflow => break,
                crypto::buffer::BufferResult::BufferOverflow => {}
            }
        }
        Ok(final_result)
    }

    fn decode(&mut self, input: &[u8]) -> Result<Vec<u8>, String> {
        self.decryptor.reset(&self.iv);
        let mut read_buf = RefReadBuffer::new(input);
        let mut buff = [0u8; 4096];
        let mut write_buf = RefWriteBuffer::new(&mut buff);
        let mut final_result: Vec<u8> = Vec::new();

        loop {
            let result = self
                .decryptor
                .decrypt(&mut read_buf, &mut write_buf, true)
                .map_err(|_e| "can't decrypt")?;
            final_result.extend(
                write_buf
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .copied(),
            );
            match result {
                crypto::buffer::BufferResult::BufferUnderflow => break,
                crypto::buffer::BufferResult::BufferOverflow => {}
            }
        }
        Ok(final_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::hash::bytes_to_string;

    #[test]
    fn test1() -> Result<(), Box<dyn std::error::Error>> {
        // let key = [0u8; 32];
        // let iv = [0u8; 16];
        let text = "a".to_string().repeat(15);
        println!("text len = {}", text.len());
        // let mut cipher = AESCryptor::try_new_with(&key, &iv).unwrap();
        let mut cipher = AESCryptor::try_new().unwrap();
        // 加密
        let r = cipher.encode(text.as_bytes()).unwrap();
        // 16B的数据加密后被填充了到32B
        // 15B的数据加密后被填充了到16B
        // 0B的数据还是0B
        println!("len = {}", r.len());
        println!("{}", bytes_to_string(&r).unwrap());

        // 解密
        let r = cipher.decode(&r).unwrap();
        println!("{}", String::from_utf8(r).unwrap());

        println!("{:?}", cipher.to_key_iv_bytes());
        Ok(())
    }

    #[test]
    fn test2() {
        let text1 = "123";
        let text2 = "abc";
        let mut cipher = AESCryptor::try_new().unwrap();
        let mut c2 = AESCryptor::try_from(&cipher.to_key_iv_bytes()).unwrap();
        let enc1 = cipher.encode(text1.as_bytes()).unwrap();
        let enc2 = cipher.encode(text2.as_bytes()).unwrap();
        println!("len1 = {}, len2 = {}", enc1.len(), enc2.len());

        let dec2 = c2.decode(&enc2).unwrap();
        let dec1 = c2.decode(&enc1).unwrap();
        let dec2_text = String::from_utf8(dec2).unwrap();
        let dec1_text = String::from_utf8(dec1).unwrap();
        println!("len = {}, c = {}", dec1_text.len(), dec1_text);
        println!("len = {}, c = {}", dec2_text.len(), dec2_text);
    }
}
