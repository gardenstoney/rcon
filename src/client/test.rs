use std::io::{self, Read, Write};
use crate::{test::{DebugStream, BufferedStream}, RconError, RconPacket, RconPacketType};

use super::RconClient;

#[test]
fn client_connect_and_auth() {
    struct MockStream {
        base: BufferedStream
    }

    impl MockStream {
        fn new() -> Self { Self { base: BufferedStream::new(Vec::new()) } }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let id = RconPacket::from_bytes(buf).expect("Couldn't parse packet").p_id;
            let packet = RconPacket::new(id, RconPacketType::AuthResponse.into(), "");
            self.base = BufferedStream::new(packet.as_bytes());

            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }

    impl Read for MockStream {
        fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
            self.base.read(&mut buf)
        }
    }

    let stream = DebugStream::new(MockStream::new());
    
    
    let mut client = RconClient { stream };
    client.auth("asdf").unwrap();
}


#[test]
fn client_auth_fail() {
    let packet = RconPacket::new(-1, RconPacketType::AuthResponse.into(), "");
    let stream = DebugStream::new(BufferedStream::new(packet.as_bytes()));

    let mut client = RconClient { stream };
    assert!(matches!(client.auth("asdf"), Err(RconError::AuthFailed)))
}