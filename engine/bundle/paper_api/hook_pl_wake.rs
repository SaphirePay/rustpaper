// Written by tools/gen_bundle.py.
fn paper_hook_pl_wake(hub: &Hub, i: usize) -> ::core::ops::ControlFlow<()> {
    let g = &hub.guests[i];
    if crate::paper_hub::wake_held(g.uuid, g.bed.is_some() || g.sleeping) {
        return ::core::ops::ControlFlow::Break(());
    }
    ::core::ops::ControlFlow::Continue(())
}
