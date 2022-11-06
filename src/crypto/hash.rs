use crypto::{digest::Digest, sha2::Sha256};


/// 将单个16进制数字转为ascii字符数字
/// 若高四位不为0则会出错
#[inline]
fn u8_to_ascii_char(n: u8) -> u8 {
    if n < 0xA {
        n + 48
    } else {
        // n + 55 // 大写
        n + 87 // 小写
    }
}

/// 将序列转为16进制字符串表示
/// 一个字节用两个十六进制字符表示
pub fn bytes_to_string(bs: &[u8]) -> Result<String, std::string::FromUtf8Error> {
    let mut buf = Vec::new();
    for b in bs {
        let (h, l) = ((*b & 0xf0) >> 4, *b & 0x0f);
        buf.push(u8_to_ascii_char(h));
        buf.push(u8_to_ascii_char(l));
    }
    String::from_utf8(buf)
}

/// 将一个对象转为Sha256序列
pub trait ToSha256 {
    /// 得到32字节的Sha256序列
    fn to_sha256(&self) -> Vec<u8>;
    /// 得到32字节Sha256序列的字符串表示
    fn to_sha256_str(&self) -> String {
        bytes_to_string(&self.to_sha256()).unwrap()
    }
}

impl ToSha256 for &[u8] {
    fn to_sha256(&self) -> Vec<u8> {
        let mut r = vec![0u8; 32];
        let mut sha = Sha256::new();
        sha.input(self);
        sha.result(&mut r);
        r
    }
}

impl ToSha256 for Vec<u8> {
    fn to_sha256(&self) -> Vec<u8> {
        self.as_slice().to_sha256()
    }
}

impl ToSha256 for &str {
    fn to_sha256(&self) -> Vec<u8> {
        self.as_bytes().to_sha256()
    }
}

impl ToSha256 for String {
    fn to_sha256(&self) -> Vec<u8> {
        self.as_bytes().to_sha256()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_to_ascii() {
        let a = 0x0A;
        println!("{}", String::from_utf8(vec![u8_to_ascii_char(a)]).unwrap());
        let a = 0x2;
        println!("{}", String::from_utf8(vec![u8_to_ascii_char(a)]).unwrap());
        let a = 0x0f;
        println!("{}", String::from_utf8(vec![u8_to_ascii_char(a)]).unwrap());
    }

    #[test]
    fn test1() {
        let s = "hello";
        let sha = s.to_sha256_str();
        println!("b'{}'", sha);
        println!("len = {} bytes", sha.len()/2)
    }

    #[test]
    fn gen_key_iv() {
        let s = "eryu";
        let key = s.to_sha256();
        let s = "yxy";
        let iv = s.to_sha256();
        println!("key = {:?}", key);
        println!("iv = {:?}", &iv[0..16]);
    }
}
