use crate::core::frames::ProtocolMessage;

pub enum StreamAction {
    Send(ProtocolMessage),
    InitiateDisconnect,
    AcceptDisconnect,
    None,
}
