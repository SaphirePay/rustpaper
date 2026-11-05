// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_velocity(game: &mut crate::game::Game<'_>, power: f64, xd: f64, zd: f64) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_velocity") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::velocity_gate(game, |game| game.knock_back_player(power, xd, zd)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_velocity", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
