use core::fmt;
use std::io;

pub mod client;

#[derive(Debug)]
pub enum RconError {
    IoError(io::Error),
    InvalidPacket{buffer: Box<[u8]>, message: &'static str},
    InvalidResponse{resp: RconPacket},
    InvalidResponseType{resp: RconPacket, expected: i32},
    InvalidRequest{req: RconPacket},
    AuthFailed,
}

impl From<io::Error> for RconError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl fmt::Display for RconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RconError::IoError(e) => write!(f, "IO error: {}", e),
            RconError::InvalidPacket { buffer, message } => write!(f, "Invalid packet: {}, {:02X?}", message, buffer),
            RconError::InvalidResponse { resp } => write!(f, "Invalid response:\n{:#?}", resp),
            RconError::InvalidResponseType { resp, expected } => write!(f, "Invalid response type: expected {}, got {}", expected, resp.p_type),
            RconError::InvalidRequest { req} => write!(f, "Invalid request:\n{:#?}", req),
            RconError::AuthFailed => write!(f, "Auth failed")
        }
    }
}

impl std::error::Error for RconError {}

pub enum RconPacketType {
    Auth,
    ExecCommand,
    AuthResponse,
    ResponseValue,
    Unknown(i32)
}

impl RconPacketType {
    pub fn to_i32(&self) -> i32 {
        match self {
            RconPacketType::Auth => 3,
            RconPacketType::ExecCommand => 2,
            RconPacketType::AuthResponse => 2,
            RconPacketType::ResponseValue => 0,
            RconPacketType::Unknown(n) => *n
        }
    }
}

impl From<RconPacketType> for i32 {
    fn from(value: RconPacketType) -> Self {
        value.to_i32()
    }
}

#[derive(Debug)]
pub struct RconPacket {
    pub p_id: i32,
    pub p_type: i32,
    pub body: String
}

impl RconPacket {
    pub fn new<T: Into<String>>(p_id: i32, p_type: i32, body: T) -> Self {
        let body: String = body.into();
        if body.len() > 4086 {
            panic!("Packet longer than 4100 bytes");
        }

        Self { p_id, p_type, body: body.into() }
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, RconError> {
        let packet_size = i32::from_le_bytes(buf[0..4].try_into().unwrap());
        let len = buf.len();

        if len != (packet_size + 4).try_into().unwrap() {
            return Err(RconError::InvalidPacket{ buffer: buf.into(), message: "length different from expected" });
        }

        let p_id = i32::from_le_bytes(buf[4..8].try_into().unwrap());
        let p_type = i32::from_le_bytes(buf[8..12].try_into().unwrap());
        let body = String::from_utf8_lossy(&buf[12..len - 2]).to_string();

        Ok(Self::new(p_id, p_type, body))
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let len = self.body.len() + 14;
        let packet_size: i32 = (len - 4).try_into().unwrap();
    
        let mut buffer = vec![0; len];
    
        buffer[..4].copy_from_slice(&packet_size.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.p_id.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.p_type.to_le_bytes());
        buffer[12..len - 2].copy_from_slice(self.body.as_bytes());
        buffer[len - 2..].fill(0);
    
        return buffer;
    }
}

#[cfg(test)]
mod test;