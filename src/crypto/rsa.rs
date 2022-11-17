/// RSA算法
///
/// 生成一对公钥和私钥
/// 将公钥发送给对方
/// 对方用受到的公钥加密后发送
/// 用私钥解密内容
///
///
use rsa::{
    errors::Error as CryptError,
    pkcs8::{spki::Error as ParseError, DecodePublicKey, EncodePublicKey, LineEnding},
    PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RSAError {
    /// 编解码相关错误
    #[error(transparent)]
    CryptError(#[from] CryptError),
    // 密钥转换错误
    #[error(transparent)]
    ParseError(#[from] ParseError),
}

/// 使用随机数生成密钥对
/// 使用PKCS#1 v1.5填充
pub struct RSAKeyPair {
    pri_key: Option<RsaPrivateKey>,
    pub_key: RsaPublicKey,
}

impl RSAKeyPair {
    pub fn new() -> Result<RSAKeyPair, RSAError> {
        let mut rng = rand::thread_rng();
        let bits = 2048; // 256B
        let pri_key = RsaPrivateKey::new(&mut rng, bits)?;
        let pub_key = RsaPublicKey::from(&pri_key);
        Ok(RSAKeyPair {
            pri_key: Some(pri_key),
            pub_key,
        })
    }

    /// 使用公钥加密
    /// 数据不能超过256-11=245B(对于2048bit)
    pub fn encrypt(&self, input: &[u8]) -> Result<Vec<u8>, RSAError> {
        let mut rng = rand::thread_rng();
        Ok(self
            .pub_key
            .encrypt(&mut rng, PaddingScheme::new_pkcs1v15_encrypt(), input)?)
    }

    /// 使用私钥解密
    pub fn decrypt(&self, input: &[u8]) -> Result<Vec<u8>, RSAError> {
        if let Some(ref pri_key) = self.pri_key {
            Ok(pri_key.decrypt(PaddingScheme::new_pkcs1v15_encrypt(), input)?)
        } else {
            // 暂时不知道该返回什么错误
            // 没有私钥也算一个验证错误吧
            Err(CryptError::Verification.into())
        }
    }

    pub fn public_key_bytes(&self) -> String {
        self.pub_key.to_public_key_pem(LineEnding::LF).unwrap()
    }
}

impl TryFrom<&str> for RSAKeyPair {
    type Error = RSAError;

    fn try_from(key: &str) -> Result<Self, Self::Error> {
        let pub_key = RsaPublicKey::from_public_key_pem(key)?;
        Ok(Self {
            pri_key: None,
            pub_key,
        })
    }
}

impl TryFrom<&String> for RSAKeyPair {
    type Error = RSAError;

    fn try_from(key: &String) -> Result<Self, Self::Error> {
        let pub_key = RsaPublicKey::from_public_key_pem(&key)?;
        Ok(Self {
            pri_key: None,
            pub_key,
        })
    }
}

impl TryFrom<String> for RSAKeyPair {
    type Error = RSAError;

    fn try_from(key: String) -> Result<Self, Self::Error> {
        let pub_key = RsaPublicKey::from_public_key_pem(&key)?;
        Ok(Self {
            pri_key: None,
            pub_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let keys = RSAKeyPair::new().unwrap();

        let pk = keys.public_key_bytes();
        println!("pub_key: {}", pk);

        // Encrypt
        // 256B(2048b)的RSA只能加密 256-11=245B的数据
        let data = b"a".repeat(245);
        println!("data len = {}", data.len());
        let pk = RSAKeyPair::try_from(&pk).unwrap();
        let enc_data = pk.encrypt(&data).unwrap();
        println!("len = {}", enc_data.len());
        println!("{:?}", enc_data);

        // Decrypt
        let dec_data = keys.decrypt(&enc_data).unwrap();
        println!("{}", String::from_utf8(dec_data).unwrap());
    }
}
