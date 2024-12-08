use std::io::{self, Read, Write};
use crate::{RconError, RconPacket, RconPacketType};

use super::RconClient;

struct DebugStream<T: Read + Write> {
    base: T,
    recv: Vec<Box<[u8]>>,
    sent: Vec<Box<[u8]>>
}

impl<T: Read + Write> DebugStream<T> {
    fn new(stream: T) -> Self {
        Self { base: stream, recv: Vec::new(), sent: Vec::new() }
    }
}

impl<T: Read + Write> Write for DebugStream<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {   
        let size = self.base.write(buf)?;     
        self.sent.push(Box::from(buf));

        println!("OUTBOUND----------");
        println!("{:02X?}", buf);
    
        return Ok(size)
    }

    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

impl<T: Read + Write> Read for DebugStream<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = self.base.read(buf)?;
        self.recv.push(Box::from(&*buf));
        
        println!("INBOUND----------");
        println!("{:02X?}", &buf[..size]);

        return Ok(size);
    }
}

#[test]
fn client_connect_and_auth() {
    struct MockStream {
        id: i32
    }

    impl MockStream {
        fn new() -> Self { Self { id: 0 } }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.id = RconPacket::from_bytes(buf).expect("Couldn't parse packet").p_id;

            Ok(buf.len()) 
        }

        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }

    impl Read for MockStream {
        fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
            let packet = RconPacket::new(self.id, RconPacketType::AuthResponse.into(), "");
            buf.write(&packet.as_bytes())
        }
    }

    let stream = DebugStream::new(MockStream::new());
    
    
    let mut client = RconClient { stream };
    client.auth("asdf").unwrap();
}


#[test]
fn client_auth_fail() {
    struct MockStream;

    impl MockStream {
        fn new() -> Self { Self {} }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> { Ok(buf.len()) }

        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }

    impl Read for MockStream {
        fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
            let packet = RconPacket::new(-1, RconPacketType::AuthResponse.into(), "");
            buf.write(&packet.as_bytes())
        }
    }

    let stream = DebugStream::new(MockStream::new());

    let mut client = RconClient { stream };
    assert!(matches!(client.auth("asdf"), Err(RconError::AuthFailed)))
}