// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_cooldown(game: &mut crate::game::Game<'_>, group: &Arc<str>, ticks: i32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_cooldown") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::cooldown_gate(game, group, ticks, |game, group, ticks| game.add_cooldown(group, ticks)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_cooldown", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
