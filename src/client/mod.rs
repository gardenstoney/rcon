use std::{io::{Read, Write}, net::{SocketAddr, TcpStream}};

use crate::{RconError, RconPacket, RconPacketType};


fn request<T: Read + Write + Unpin>(mut stream: &mut T, payload: RconPacket) -> Result<RconPacket, RconError> {
    stream.write_all(&payload.as_bytes())?;

    let resp = RconPacket::from_stream(&mut stream)?;
    
    Ok(resp)
}

fn request_multi_packet<T: Read + Write + Unpin>(mut stream: &mut T, payload: RconPacket) -> Result<Vec<RconPacket>, RconError> {
    stream.write_all(&payload.as_bytes())?;

    let end_packet = RconPacket::new(2, RconPacketType::ResponseValue.into(), "");
    stream.write_all(&end_packet.as_bytes())?;

    let mut responses: Vec<RconPacket> = Vec::new();

    loop {
        let resp = RconPacket::from_stream(&mut stream)?;
        
        if resp.p_id == 2 && resp.body.len() == 0 {
            break;
        } else {
            responses.push(resp);
        }
    }

    Ok(responses)
}

pub struct RconClient<T: Read + Write + Unpin> {
    stream: T,
}

impl RconClient<TcpStream> {
    pub fn connect(addr: SocketAddr) -> Result<Self, RconError> {
        let stream = TcpStream::connect(addr)?;
        let client = Self { stream };

        Ok(client)
    }
}

impl<T: Read + Write + Unpin> RconClient<T> {
    pub fn auth<U: Into<String>>(&mut self, password: U) -> Result<(), RconError>{
        let packet =
            RconPacket::new(1, RconPacketType::Auth.into(), password);
        
        let resp = request(&mut self.stream, packet)?;

        if resp.p_type != RconPacketType::AuthResponse.into() {
            return Err(
                RconError::InvalidResponseType { expected: RconPacketType::AuthResponse.into(), resp }
            );
        }

        if resp.p_id == -1 {
            return Err(RconError::AuthFailed);
        }

        if resp.p_id != 1 {
            return Err(RconError::InvalidResponse { resp });  // response with different id
        }

        return Ok(());
    }

    pub fn exec<U: Into<String>>(&mut self, message: U) -> Result<String, RconError> {
        let payload = RconPacket::new(1, RconPacketType::ExecCommand.into(), message.into());
        let responses = request_multi_packet(&mut self.stream, payload)?;

        let mut body = String::new();

        for resp in responses { 
            if resp.p_type != RconPacketType::ResponseValue.into() {
                return Err(
                    RconError::InvalidResponseType { expected: RconPacketType::ResponseValue.into(), resp }
                );
            }
            
            if resp.p_id != 1 {
                return Err(RconError::InvalidResponse { resp });  // response with different id
            }

            body += &resp.body;
        }

        return Ok(body);
    }
}

#[cfg(test)]
mod test;