use crate::core::frames::ProtocolMessage;

pub enum StreamAction {
    Send(ProtocolMessage),
    Disconnect,
    None,
}
