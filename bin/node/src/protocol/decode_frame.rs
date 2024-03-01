use anyhow::bail;
use super::vars::{
    PROTOCOL_BUF_SIZE,
    ProtocolAction,
    PROT_OPCODE_CONTINUATION,
    PROT_OPCODE_CONN_CLOSED,
    PROT_OPCODE_PING,
    PROT_OPCODE_PONG,
    PROT_OPCODE_DATA,
};
use super::encode_frame_data::protocol_encode_frame_data;

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
        PROT_OPCODE_CONTINUATION | PROT_OPCODE_DATA => {
            buf.extend_from_slice(&frame[1..PROTOCOL_BUF_SIZE]);
            ProtocolAction::None
        },
        PROT_OPCODE_CONN_CLOSED => {
            ProtocolAction::CloseConnection
        },
        PROT_OPCODE_PING => {
            let mut chunks = protocol_encode_frame_data(PROT_OPCODE_PONG, &[]).into_iter();
            let chunk = chunks.next().expect("Should be at least one chunk");

            assert_eq!(chunks.next(), None, "Should not be more then one chunk");

            ProtocolAction::Send(chunk)
        },
        PROT_OPCODE_PONG => {
            ProtocolAction::ReceivedPong
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
