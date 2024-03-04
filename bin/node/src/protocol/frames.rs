use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use anyhow::{anyhow, bail, Result};
use crate::protocol::node_info::NodeInfo;
use crate::utils::socket_addr_to_bytes::{socket_addr_from_bytes, socket_addr_to_bytes};

const PROT_OPCODE_CONTINUATION: u8 = 0b0000; // received frame is a continuation of previous unfinished frame
const PROT_OPCODE_CONN_INIT:    u8 = 0b0001; // init connection with some data
const PROT_OPCODE_CONN_CLOSED:  u8 = 0b0010; // party disconnected // todo: send in case of graceful shutdown
const PROT_OPCODE_PING:         u8 = 0b0011; // checking if connection is still alive
const PROT_OPCODE_PONG:         u8 = 0b0100; // answer if connection is still alive
const PROT_OPCODE_DATA:         u8 = 0b0101; // frame contains application data
const PROT_OPCODE_NODE_INFO:    u8 = 0b0110; // information about other nodes client chooses to connect/disconnect/etc.

pub enum ProtocolBufferType {
    ConnInit,
    Data,
    NodeInfo,
    Pong,
}
pub enum ProtocolMessage {
    ConnInit { // maybe there will be more info
        server_addr: SocketAddr,
    },
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
            ProtocolMessage::ConnInit { server_addr } => {
                buf.extend(
                    socket_addr_to_bytes(server_addr)?
                );
                PROT_OPCODE_CONN_INIT
            }
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
                    None => {}
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

        if len == 0 {
            result.push(vec![1 << 7 | opcode])
        } else {
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

            let frame = &frame[1..frame.len()];

            match opcode {
                PROT_OPCODE_CONN_INIT => {
                    buf.extend_from_slice(frame);

                    if fin == 1 {
                        let mut iter = buf.into_iter();
                        let server_addr = socket_addr_from_bytes(&mut iter)
                            .expect("---Failed to parse buffer")
                            .expect("---Address is required for this opcode");
                        let msg = Self::ConnInit { server_addr };
                        return Ok(Some(msg));
                    } else {
                        buf_type = ProtocolBufferType::ConnInit;
                    }
                }
                PROT_OPCODE_CONTINUATION => {
                    buf.extend_from_slice(frame);

                    if fin == 1 {
                        let msg = match buf_type {
                            ProtocolBufferType::ConnInit => {
                                let mut iter = buf.into_iter();
                                let server_addr = socket_addr_from_bytes(&mut iter)
                                    .expect("---Failed to parse buffer")
                                    .expect("---Address is required for this opcode");
                                Self::ConnInit { server_addr }
                            }
                            ProtocolBufferType::Data => Self::Data(buf),
                            ProtocolBufferType::NodeInfo => Self::NodeInfo(NodeInfo::from_bytes(buf)?.expect("opcode has to provide at least one")),
                            ProtocolBufferType::Pong => Self::Pong(NodeInfo::from_bytes(buf)?),
                        };
                        return Ok(Some(msg));
                    }
                }
                PROT_OPCODE_DATA => {
                    buf.extend_from_slice(frame);

                    if fin == 1 {
                        let msg = Self::Data(buf);
                        return Ok(Some(msg));
                    } else {
                        buf_type = ProtocolBufferType::Data;
                    }
                }
                PROT_OPCODE_NODE_INFO => {
                    buf.extend_from_slice(frame);

                    if fin == 1 {
                        let msg = Self::NodeInfo(NodeInfo::from_bytes(buf)?.expect("opcode has to provide at least one"));
                        return Ok(Some(msg));
                    } else {
                        buf_type = ProtocolBufferType::NodeInfo;
                    }
                }
                PROT_OPCODE_PONG => {
                    buf.extend_from_slice(frame);

                    if fin == 1 {
                        let msg = Self::Pong(NodeInfo::from_bytes(buf)?);
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
