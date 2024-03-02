use std::io::{Read, Write};
use std::net::TcpStream;
use anyhow::{anyhow, bail, Result};
use crate::protocol::node_info::NodeInfo;

const PROT_OPCODE_CONTINUATION: u8 = 0b0000; // received frame is an continuation of previous unfinished frame
const PROT_OPCODE_CONN_CLOSED:  u8 = 0b0001; // party disconnected // todo: send in case of graceful shutdown
const PROT_OPCODE_PING:         u8 = 0b0010; // checking if connection is still alive
const PROT_OPCODE_PONG:         u8 = 0b0011; // answer if connection is still alive
const PROT_OPCODE_DATA:         u8 = 0b0100; // frame contains application data
const PROT_OPCODE_NODE_INFO:    u8 = 0b0101; // information about other nodes client chooses to connect/disconnect/etc.

pub enum ProtocolBufferType {
    Data,
    NodeInfo,
    Pong,
}
pub enum ProtocolMessage {
    ConnClosed,
    Ping,
    Pong(Option<NodeInfo>),
    Data(Vec<u8>),
    NodeInfo(NodeInfo),
}

impl ProtocolMessage {
    pub const FRAME_SIZE: usize = 256;
    const PAYLOAD_SIZE: usize = Self::FRAME_SIZE - 1;

    pub fn send_to_stream(
        self,
        stream: &mut TcpStream,
    ) -> Result<()> {
        for chunk in self.into_frames()? {
            stream.write(&chunk).map_err(|e| anyhow!("---Failed to write to stream: {}", e.to_string()))?;
        }

        Ok(())
    }

    fn into_frames(self) -> Result<Vec<Vec<u8>>> {
        let mut buf = vec![];

        let opcode = match self {
            ProtocolMessage::ConnClosed => {
                PROT_OPCODE_CONN_CLOSED
            }
            ProtocolMessage::Ping => {
                PROT_OPCODE_PING
            }
            ProtocolMessage::Pong(node_info) => {
                match node_info {
                    Some(node_info) => {
                        buf.extend(
                            node_info.into_bytes()?
                        );
                    }
                    None => {},
                }
                PROT_OPCODE_PONG
            }
            ProtocolMessage::NodeInfo(node_info) => {
                buf.extend(
                    node_info.into_bytes()?
                );
                PROT_OPCODE_NODE_INFO
            }
            ProtocolMessage::Data(bytes) => {
                buf.extend(bytes);
                PROT_OPCODE_DATA
            }
        };

        let len = buf.len();
        let mut start = 0;

        let mut result = Vec::with_capacity(len / Self::FRAME_SIZE + 1);

        for payload_chunk in buf.chunks(Self::PAYLOAD_SIZE) {
            start += Self::PAYLOAD_SIZE;

            let fin = if start < len {
                0
            } else {
                1
            };
            let opcode = if result.len() > 0 {
                PROT_OPCODE_CONTINUATION
            } else {
                opcode
            };

            let mut result_chunk = Vec::with_capacity(Self::FRAME_SIZE);
            result_chunk.push(fin << 7 | opcode);
            result_chunk.extend_from_slice(payload_chunk);

            result.push(result_chunk);

            if fin == 1 {
                break;
            }
        }

        Ok(result)
    }

    pub fn from_stream(
        stream: &mut TcpStream,
    ) -> Result<Option<Self>> {
        let mut buf = Vec::new();
        let mut buf_type = ProtocolBufferType::Data;

        let mut frame = [0; Self::FRAME_SIZE];

        loop {
            let n = stream.read(&mut frame)?;
            if n == 0 {
                return Ok(None); // stream has ended = host disconnected
            }

            let fin = frame[0] >> 7; // bit
            let rsv = (frame[0] << 1) >> 5; // 3 bits
            let opcode = (frame[0] << 1) >> 1; // 4 bits

            if rsv != 0 {
                bail!("Unknown usage of reserved bits")
            }

            match opcode {
                PROT_OPCODE_CONTINUATION => {
                    buf.extend_from_slice(&frame[1..Self::FRAME_SIZE]);

                    if fin == 1 {
                        let msg = match buf_type {
                            ProtocolBufferType::Data => Self::Data(buf),
                            ProtocolBufferType::NodeInfo => Self::NodeInfo(NodeInfo::from_bytes(buf)?),
                            ProtocolBufferType::Pong => {
                                let info = if buf.len() > 0 {
                                    Some(NodeInfo::from_bytes(buf)?)
                                } else {
                                    None
                                };
                                Self::Pong(info)
                            },
                        };
                        return Ok(Some(msg));
                    }
                }
                PROT_OPCODE_DATA => {
                    buf.extend_from_slice(&frame[1..Self::FRAME_SIZE]);

                    if fin == 1 {
                        let msg = Self::Data(buf);
                        return Ok(Some(msg));
                    } else {
                        buf_type = ProtocolBufferType::Data;
                    }
                }
                PROT_OPCODE_NODE_INFO => {
                    buf.extend_from_slice(&frame[1..Self::FRAME_SIZE]);

                    if fin == 1 {
                        let msg = Self::NodeInfo(NodeInfo::from_bytes(buf)?);
                        return Ok(Some(msg));
                    } else {
                        buf_type = ProtocolBufferType::NodeInfo;
                    }
                }
                PROT_OPCODE_PONG => {
                    buf.extend_from_slice(&frame[1..Self::FRAME_SIZE]);

                    if fin == 1 {
                        let info = if buf.len() > 0 {
                            Some(NodeInfo::from_bytes(buf)?)
                        } else {
                            None
                        };

                        let msg = Self::Pong(info);
                        return Ok(Some(msg));
                    } else {
                        buf_type = ProtocolBufferType::Pong;
                    }
                }
                PROT_OPCODE_CONN_CLOSED => {
                    if fin == 0 {
                        bail!("Received single-frame message but fin bit is not 1")
                    }

                    let msg = Self::ConnClosed;
                    return Ok(Some(msg));
                },
                PROT_OPCODE_PING => {
                    if fin == 0 {
                        bail!("Received single-frame message but fin bit is not 1")
                    }

                    let msg = Self::Ping;
                    return Ok(Some(msg));
                },
                _ => {
                    bail!("Unknown opcode")
                },
            }
        }
    }
}
