// Written by tools/gen_bundle.py.
fn paper_hook_table_valid(g: &horizon_client::lan::guest::Guest, pos: [i32; 3]) -> ::core::ops::ControlFlow<bool> {
    if horizon_client::paper_hub::forced_menu(g.uuid, pos) {
        return ::core::ops::ControlFlow::Break(true);
    }
    ::core::ops::ControlFlow::Continue(())
}
