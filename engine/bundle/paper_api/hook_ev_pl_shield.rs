// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_shield(game: &mut crate::game::Game<'_>, component: &horizon_client::items::BlocksAttacks, attacker: [f64; 2], fully: bool, disable: f32, at: Option<[f64; 3]>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_shield") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::shield_gate(game, disable, component.disable_scale, |game, disable| game.blocked_by_attacker(component, attacker, fully, disable, at)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_shield", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
