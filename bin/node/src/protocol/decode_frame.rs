use anyhow::bail;
use super::vars::{PROTOCOL_BUF_SIZE, ProtocolAction, PROT_OPCODE_CONTINUATION, PROT_OPCODE_CONN_CLOSED, PROT_OPCODE_PING, PROT_OPCODE_PONG, PROT_OPCODE_DATA, PROT_OPCODE_NODE_INFO, ProtocolBufferType};

pub fn protocol_decode_frame(
    buf: &mut Vec<u8>,
    frame: [u8; PROTOCOL_BUF_SIZE],
) -> anyhow::Result<ProtocolAction> {
    let fin = frame[0] >> 7; // bit
    let rsv = (frame[0] << 1) >> 5; // 3 bits
    let opcode = (frame[0] << 1) >> 1; // 4 bits

    if rsv != 0 {
        bail!("Unknown usage of reserved bits")
    }

    let action = match opcode {
        PROT_OPCODE_CONTINUATION | PROT_OPCODE_DATA | PROT_OPCODE_NODE_INFO | PROT_OPCODE_PONG => {
            buf.extend_from_slice(&frame[1..PROTOCOL_BUF_SIZE]);

            match opcode {
                PROT_OPCODE_CONTINUATION => ProtocolAction::None,
                PROT_OPCODE_DATA => ProtocolAction::UpdateBufferType(ProtocolBufferType::Data),
                PROT_OPCODE_NODE_INFO => ProtocolAction::UpdateBufferType(ProtocolBufferType::NodeInfo),
                PROT_OPCODE_PONG => ProtocolAction::UpdateBufferType(ProtocolBufferType::Pong),
                _ => {
                    unreachable!()
                },
            }
        },
        PROT_OPCODE_CONN_CLOSED => {
            ProtocolAction::CloseConnection
        },
        PROT_OPCODE_PING => {
            ProtocolAction::ReceivedPing
        },
        _ => {
            bail!("Unknown opcode")
        },
    };

    Ok(if fin == 1 {
        ProtocolAction::UseBuffer
    } else {
        action
    })
}
