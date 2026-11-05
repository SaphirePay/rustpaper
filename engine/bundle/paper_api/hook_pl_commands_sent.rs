// Written by tools/gen_bundle.py.
fn paper_hook_pl_commands_sent(hub: &Hub, i: usize) -> ::core::ops::ControlFlow<()> {
    if crate::paper_hub::commands_sent(hub.guests[i].uuid) {
        return ::core::ops::ControlFlow::Break(());
    }
    ::core::ops::ControlFlow::Continue(())
}
