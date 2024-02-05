use anyhow::{bail, Result};

pub mod read_stream;

pub const PROT_OPCODE_CONTINUATION: u8 = 0b0000;
pub const PROT_OPCODE_CONN_CLOSED:  u8 = 0b0001;
pub const PROT_OPCODE_PING:         u8 = 0b0010;
pub const PROT_OPCODE_PONG:         u8 = 0b0011;
pub const PROT_OPCODE_DATA:         u8 = 0b0100;

pub const PROTOCOL_BUF_SIZE: usize = 256;

pub enum ProtocolAction {
    None,
    UseBuffer,
    CloseConnection,
    Send(Vec<u8>),
}

pub fn protocol_encode_frames(
    opcode: u8,
    data: &[u8],
) -> Vec<Vec<u8>> {
    const PAYLOAD_SIZE: usize = PROTOCOL_BUF_SIZE - 1;

    let len = data.len();
    let mut start = 0;

    let mut result = Vec::with_capacity(len / PROTOCOL_BUF_SIZE + 1);

    for payload_chunk in data.chunks(PAYLOAD_SIZE) {
        start += PAYLOAD_SIZE;

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

        let mut result_chunk = Vec::with_capacity(PROTOCOL_BUF_SIZE);
        result_chunk.push(fin << 7 | opcode);
        result_chunk.extend_from_slice(payload_chunk);

        result.push(result_chunk);

        if fin == 1 {
            break;
        }
    }

    result
}

pub fn protocol_parse_frame(
    buf: &mut Vec<u8>,
    mut frame: [u8; PROTOCOL_BUF_SIZE],
) -> Result<ProtocolAction> {
    let fin = frame[0] >> 7;
    let rsv = (frame[0] << 1) >> 5;
    let opcode = (frame[0] << 1) >> 1;

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
            let mut chunks = protocol_encode_frames(PROT_OPCODE_PONG, &[]).into_iter();
            let chunk = chunks.next().expect("Should be at least one chunk");

            assert_eq!(chunks.next(), None, "Should not be more then one chunk");

            ProtocolAction::Send(chunk)
        },
        PROT_OPCODE_PONG => {
            ProtocolAction::None
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
