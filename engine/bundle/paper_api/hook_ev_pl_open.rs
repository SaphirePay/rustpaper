// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_open(game: &mut crate::game::Game<'_>, pos: [i32; 3]) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_open") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::menu_later(game, pos) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_open", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
