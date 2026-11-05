// Written by tools/gen_bundle.py.
fn paper_hook_station_valid(g: &horizon_client::lan::guest::Guest, gs: &GuestStation) -> ::core::ops::ControlFlow<bool> {
    if horizon_client::paper_hub::forced_menu(g.uuid, gs.pos) {
        return ::core::ops::ControlFlow::Break(true);
    }
    ::core::ops::ControlFlow::Continue(())
}
