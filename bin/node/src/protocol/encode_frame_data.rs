use super::vars::{PROT_OPCODE_CONTINUATION, PROTOCOL_BUF_SIZE};

pub type ProtocolFrame = Vec<Vec<u8>>;

pub fn protocol_encode_frame_data(
    opcode: u8,
    data: &[u8],
) -> ProtocolFrame {
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