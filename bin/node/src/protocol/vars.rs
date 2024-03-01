pub const PROT_OPCODE_CONTINUATION: u8 = 0b0000; // received frame is an continuation of previous unfinished frame
pub const PROT_OPCODE_CONN_CLOSED:  u8 = 0b0001; // party disconnected // todo: send in case of graceful shutdown
pub const PROT_OPCODE_PING:         u8 = 0b0010; // checking if connection is still alive // todo: send for fun
pub const PROT_OPCODE_PONG:         u8 = 0b0011; // answer if connection is still alive
pub const PROT_OPCODE_DATA:         u8 = 0b0100; // frame contains application data

pub const PROTOCOL_BUF_SIZE: usize = 256;

pub enum ProtocolAction {
    None,
    UseBuffer,
    CloseConnection,
    Send(Vec<u8>),
    MeasurePing,
}
