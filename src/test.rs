use std::io::{self, Read, Write};
use crate::RconError;

use super::{RconPacket, RconPacketType};

pub struct DebugStream<T: Read + Write> {
    base: T,
    recv: Vec<Box<[u8]>>,
    sent: Vec<Box<[u8]>>
}

impl<T: Read + Write> DebugStream<T> {
    pub fn new(stream: T) -> Self {
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
fn smallest_packet() {
    let packet = RconPacket::new(1, RconPacketType::ResponseValue.into(), "");
    assert_eq!(packet.as_bytes(), [10,0,0,0,1,0,0,0,0,0,0,0,0,0]);
}

#[test]
fn packet_with_body_auth() {
    let packet = RconPacket::new(0, RconPacketType::Auth.into(), "passwrd");
    assert_eq!(packet.as_bytes(), [0x11,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x03,0x00,0x00,0x00,0x70,0x61,0x73,0x73,0x77,0x72,0x64,0x00,0x00]);
}

#[test]
fn parse_packet_auth() {
    let buffer: [u8; 14] = [0x0a,0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x02,0x00,0x00,0x00,0x00,0x00];
    let packet = RconPacket::from_bytes(&buffer).unwrap();

    assert_eq!(packet.p_id, 1);
    assert_eq!(packet.p_type, 2);
    assert_eq!(packet.body, "");
}

#[test]
fn parse_packet_resp() {
    let buffer: [u8; 29] = [0x19,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x65,0x63,0x68,0x6f,0x20,0x48,0x4c,0x53,0x57,0x3a,0x20,0x54,0x65,0x73,0x74,0x00,0x00];
    let packet = RconPacket::from_bytes(&buffer).unwrap();

    assert_eq!(packet.p_id, 0);
    assert_eq!(packet.p_type, 0);
    assert_eq!(packet.body, "echo HLSW: Test");
}

#[test]
#[should_panic]
fn parse_invalid_size() {
    let buffer: [u8; 29] = [0x17,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x65,0x63,0x68,0x6f,0x20,0x48,0x4c,0x53,0x57,0x3a,0x20,0x54,0x65,0x73,0x74,0x00,0x00];
    RconPacket::from_bytes(&buffer).unwrap();
}

struct StaticPayloadStream {
    bytes: Vec<u8>,
    marker: usize
}

impl StaticPayloadStream {
    fn new(bytes: Vec<u8>) -> Self {
        Self { bytes, marker: 0 }
    }
}

impl Read for StaticPayloadStream {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let size = buf.write(&self.bytes[self.marker..])?;

        self.marker += size;

        Ok(size)
    }
}

#[test]
fn from_stream() {
    let stream = StaticPayloadStream::new(vec![0x0a,0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x02,0x00,0x00,0x00,0x00,0x00]);
    let packet = RconPacket::from_stream(stream).unwrap();

    assert_eq!(packet.p_id, 1);
    assert_eq!(packet.p_type, 2);
    assert_eq!(packet.body, "");
}

#[test]
fn from_stream_invalid_end() {
    let stream = StaticPayloadStream::new(vec![0x19,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x65,0x63,0x68,0x6f,0x20,0x48,0x4c,0x53,0x57,0x3a,0x20,0x54,0x65,0x73,0x74,0x74,0x00]);
    let err = RconPacket::from_stream(stream).unwrap_err();

    if let RconError::InvalidPacket { buffer: _, message } = err {
        if message == "packet didn't end with 0x0000" {
            return;
        }            
    }    
    panic!("{}", err);
}

#[test]
fn from_stream_packet_getting_cut() {
    struct MockStream {
        base: StaticPayloadStream,
        read_counter: u8,
    }
    
    impl MockStream {
        fn new(bytes: Vec<u8>) -> Self {
            Self { base: StaticPayloadStream{ bytes, marker: 0 }, read_counter: 0 }
        }
    }
    
    impl Read for MockStream {
        fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
            let size = match self.read_counter {
                0 => self.base.read(&mut buf)?,
                1 => self.base.read(&mut buf[..6])?,
                2 => self.base.read(&mut buf)?,
                _ => panic!("")
            };
            
            self.read_counter += 1;

            println!("read {} bytes / {} bytes", size, buf.len());
            Ok(size)
        }

    }
    let stream = MockStream::new(vec![0x19,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x65,0x63,0x68,0x6f,0x20,0x48,0x4c,0x53,0x57,0x3a,0x20,0x54,0x65,0x73,0x74,0x00,0x00]);
    RconPacket::from_stream(stream).unwrap();
}