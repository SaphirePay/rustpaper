// Written by tools/gen_bundle.py.
fn paper_hook_paper_first_packets(value: Result<(i32, Vec<u8>)>) -> Result<(i32, Vec<u8>)> {
    use crate::packets::{handshake, login};
    use horizon_core::paper_wire::connection;
    let expecting = connection::expecting();
    if expecting == 0 {
        return value;
    }
    if let Ok((id, payload)) = &value {
        if expecting == 1 && let Ok(handshake::Serverbound::Intention(intention)) = handshake::Serverbound::decode(*id, payload) {
            connection::note_handshake(intention.protocol, intention.host, intention.port, intention.intent);
        } else if expecting == 2 && let Ok(login::Serverbound::Hello(hello)) = login::Serverbound::decode(*id, payload) {
            connection::note_hello(hello.name, hello.uuid);
        }
    }
    value
}
