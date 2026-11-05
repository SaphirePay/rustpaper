// Written by tools/gen_bundle.py.
fn paper_hook_pl_kick(conn: u32, reason: &Tag) -> ::core::ops::ControlFlow<()> {
    if crate::paper_hub::kick_held(conn, reason) {
        return ::core::ops::ControlFlow::Break(());
    }
    ::core::ops::ControlFlow::Continue(())
}
