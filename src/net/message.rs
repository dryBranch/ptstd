/// # 消息机制
/// 
/// - 设计目标是小量信息传输
/// - 丢失超时重传交由传输层协议完成
/// - 停等协议
/// - 协议保证
///     - 数据相对完善（hash等简易校验）
///     - 数据定界（TCP流式协议模糊了定界）
/// 
/// ## 协议头
/// 1. 版本         1B
/// 2. 是否分片     1B
/// 3. 保留标志     2B
/// 4. 块起始       2B
/// 5. 块长度       2B
/// 6. 校验         4B
/// 7. 数据
/// 
/// 连接建立后，发送
use std::{net::{TcpStream, ToSocketAddrs}, io::{self, Write, Read}};

/// 消息头
#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct MessageHeader {
    /// 版本
    pub version: u8,
    /// 第一位为0为发送包，1为应答包
    /// 第二位为是否分片标志
    /// 第五位为应答确认位
    pub flag: u8,
    /// 保留
    pub reserved: u16,
    /// 该包开始偏移
    pub begin: usize,
    /// 该包长度
    pub length: usize,
    /// 消息总长度
    pub whole_length: usize,
    /// 该包数据部分校验码
    pub check: u32,
}

/// 管理消息连接
#[derive(Debug)]
pub struct MessageCenter {
    pub recv_hd     : MessageHeader,
    pub send_hd     : MessageHeader,
    pub recv_buf    : Vec<u8>,
    pub send_buf    : Vec<u8>,
    pub tcpstream   : Option<TcpStream>,
}

/// 消息接口
pub trait Message {
    /// 转化为字节数组
    fn as_bytes(&self) -> &[u8];
    /// 序列化
    fn encode(&self) -> &[u8];
    /// 反序列化
    fn decode() -> Self;
}

impl Default for MessageHeader {
    fn default() -> Self {
        Self {
            version: 1,
            flag: 0,
            reserved: 0,
            begin: 0,
            length: 0,
            whole_length: 0,
            check: 0,
        }
    }
}

impl MessageHeader {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let p: *const u8 = std::mem::transmute(self);
            std::slice::from_raw_parts(p, std::mem::size_of::<MessageHeader>())
        }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            let p: *mut u8 = std::mem::transmute(self);
            std::slice::from_raw_parts_mut(p, std::mem::size_of::<MessageHeader>())
        }
    }

    /// 是否是确认应答包
    pub fn is_response(&self) -> bool {
        (self.flag & 0b0001) == 1
    }

    /// 是否分片
    pub fn is_sliced(&self) -> bool {
        (self.flag & 0b0010) == 1
    }

    /// 接收到数据是否正确
    pub fn is_correct(&self) -> bool {
        (self.flag & 0x10) != 0
    }

    pub fn set_response(&mut self) {
        self.flag |= 0b0001
    }

    pub fn set_sliced(&mut self) {
        self.flag |= 0b0010
    }

    pub fn set_correct(&mut self) {
        self.flag |= 0x10
    }
}

impl MessageCenter {
    /// 通过地址创建
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<MessageCenter> {
        let tcpstream = TcpStream::connect(addr)?;
        Ok(Self::new(tcpstream))
    }

    /// 通过一个已打开的TCP连接创建
    pub fn new(tcpstream: TcpStream) -> MessageCenter {
        MessageCenter {
            recv_hd: Default::default(),
            send_hd: Default::default(),
            recv_buf: Vec::new(),
            send_buf: Vec::new(),
            tcpstream: Some(tcpstream),
        }
    }

    /// 默认的校验和
    pub fn default_checksum(_: &[u8]) -> u32 {
        0
    }

    /// 发送字节
    pub fn send_bytes(&mut self, msg: &[u8]) -> Result<(), &'static str> {
        const SLICE_SIZE: usize = 1024;
        // 协议头填充
        let whole_len = msg.len();
        let mut header = &mut self.recv_hd;
        *header = MessageHeader::default();
        header.whole_length = whole_len;
        let mut already_send_size: usize = 0;
        if msg.len() > SLICE_SIZE {
            header.set_sliced()
        }
        // 发送
        if let Some(tcpstream) = &mut self.tcpstream {
            while already_send_size < whole_len {
                // 填写偏移和长度
                header.begin = already_send_size;
                if already_send_size + SLICE_SIZE < whole_len {
                    header.length = SLICE_SIZE;
                } else {
                    header.length = whole_len - already_send_size;
                }
                // 要发送的数据
                let data = &msg[already_send_size..already_send_size+header.length];
                // 填写该片数据的校验码
                header.check = Self::default_checksum(data);
                // 发送头
                tcpstream.write_all(header.as_bytes()).map_err(|_| "send header error")?;
                // 发送数据
                tcpstream.write_all(data).map_err(|_| "write data error")?;
                // 等待接收结果
                let mut rhd = MessageHeader::default();
                tcpstream.read_exact(rhd.as_bytes_mut()).unwrap();
                if rhd.is_response() && rhd.is_correct() {
                    // 计数后移
                    already_send_size += header.length;
                }
            }
        }
        Ok(())
    }

    /// 发送一个Message
    pub fn send_message(&mut self, msg: &impl Message) -> Result<(), &'static str> {
        self.send_bytes(msg.as_bytes())
    }
    
    pub fn receive_bytes_buf<'a>(&mut self, buf: &'a mut Vec<u8>) -> Result<&'a mut Vec<u8>, Box<dyn std::error::Error>> {
        let checked_data = buf;
        checked_data.clear();
        if let Some(tcpstream) = &mut self.tcpstream {
            // 是否有后续分片
            let mut left_data = true;
            while left_data {
                // 读取该片协议头
                tcpstream.read_exact(self.recv_hd.as_bytes_mut()).unwrap();
                let header = &self.recv_hd;
                // 读取数据
                let mut buff = vec![0; header.length];
                tcpstream.read_exact(&mut buff).map_err(|e| e.kind().to_string())?;
                // 校验数据
                let mut h = MessageHeader::default();
                h.set_response();
                h.begin = header.begin;
                h.length = header.length;
                if Self::default_checksum(&buff) == header.check {
                    // 合并数据
                    checked_data.append(&mut buff);
                    // 发送确认包
                    h.set_correct();
                    tcpstream.write_all(h.as_bytes()).map_err(|_| "send response error")?;
                } else {
                    // 发送重传包
                    tcpstream.write_all(h.as_bytes()).map_err(|_| "send response error")?;
                    continue;
                }
                // 计数后移
                left_data = if header.begin + header.length == header.whole_length {
                    false
                } else {
                    true
                }
            }
        }
        Ok(checked_data)
    }

    /// 接收字节
    pub fn receive_bytes(&mut self) -> Result<&mut Vec<u8>, Box<dyn std::error::Error>> {
        let checked_data = &mut self.recv_buf;
        checked_data.clear();
        if let Some(tcpstream) = &mut self.tcpstream {
            // 是否有后续分片
            let mut left_data = true;
            while left_data {
                // 读取该片协议头
                tcpstream.read_exact(self.recv_hd.as_bytes_mut()).unwrap();
                let header = &self.recv_hd;
                // 读取数据
                let mut buff = vec![0; header.length];
                tcpstream.read_exact(&mut buff).map_err(|e| e.kind().to_string())?;
                // 校验数据
                let mut h = MessageHeader::default();
                h.set_response();
                h.begin = header.begin;
                h.length = header.length;
                if Self::default_checksum(&buff) == header.check {
                    // 合并数据
                    checked_data.append(&mut buff);
                    // 发送确认包
                    h.set_correct();
                    tcpstream.write_all(h.as_bytes()).map_err(|_| "send response error")?;
                } else {
                    // 发送重传包
                    tcpstream.write_all(h.as_bytes()).map_err(|_| "send response error")?;
                    continue;
                }
                // 计数后移
                left_data = if header.begin + header.length == header.whole_length {
                    false
                } else {
                    true
                }
            }
        }
        Ok(checked_data)
    }

}

/// 将一个Sized的引用转为字节引用
pub fn sized_as_bytes<T>(t: &T) -> &[u8] {
    unsafe {
        let p: *const u8 = std::mem::transmute(t);
        std::slice::from_raw_parts(p, std::mem::size_of::<T>())
    }
}

/// 将一个Sized的可变引用转为字节可变引用
pub fn sized_as_bytes_mut<T>(t: &mut T) -> &mut [u8] {
    unsafe {
        let p: *mut u8 = std::mem::transmute(t);
        std::slice::from_raw_parts_mut(p, std::mem::size_of::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, net::TcpListener};

    #[test]
    fn test_basic() {
        const ADDR: &str = "127.0.0.1:31000";
        let t1 = thread::spawn(|| {
            let listen = TcpListener::bind(ADDR).unwrap();
            let (stream, _) = listen.accept().unwrap();
            let mut client = MessageCenter::new(stream);
            let s = String::from("hello world").repeat(1024);
            client.send_bytes(s.as_bytes()).unwrap();
            
            let data = client.receive_bytes().unwrap();
            let data = String::from_utf8(data.to_vec()).unwrap();
            println!("server recv: {}", data);
        });  

        let mut server = MessageCenter::connect(ADDR).unwrap();
        let data = server.receive_bytes().unwrap();
        let data = String::from_utf8(data.to_vec()).unwrap();
        println!("client recv: {}", data);

        server.send_bytes(b"hello from client").unwrap();

        t1.join().unwrap();
    }

    #[test]
    fn test_twice() {
        const ADDR: &str = "127.0.0.1:31000";
        let t1 = thread::spawn(|| {
            let listen = TcpListener::bind(ADDR).unwrap();
            let (stream, _) = listen.accept().unwrap();
            let mut client = MessageCenter::new(stream);
            let s = String::from("hello world").repeat(1024);
            client.send_bytes(s.as_bytes()).unwrap();
            client.send_bytes(b"another").unwrap();
        });  

        let mut buf = Vec::new();
        let mut server = MessageCenter::connect(ADDR).unwrap();
        server.receive_bytes_buf(&mut buf).unwrap();
        let data = String::from_utf8(buf.to_vec()).unwrap();
        println!("client recv: {}", data);
        server.receive_bytes_buf(&mut buf).unwrap();
        let data = String::from_utf8(buf.to_vec()).unwrap();
        println!("client recv: {}", data);

        t1.join().unwrap();
    }
}
