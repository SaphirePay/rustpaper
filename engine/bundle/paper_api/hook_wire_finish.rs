// Written by tools/gen_bundle.py.
fn paper_hook_wire_finish(conn: &mut Connection, body: &[u8]) -> ::core::ops::ControlFlow<Result<()>> {
    use crate::packets::{Empty, configuration::Clientbound};
    use horizon_core::paper_wire::wire;
    if !wire::tasks_pending() || body != Clientbound::FinishConfiguration(Empty).encode() {
        return ::core::ops::ControlFlow::Continue(());
    }
    while wire::tasks_pending() {
        match conn.recv() {
            Ok((id, payload)) => wire::config_packet(id, &payload),
            Err(e) => return ::core::ops::ControlFlow::Break(Err(e)),
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
