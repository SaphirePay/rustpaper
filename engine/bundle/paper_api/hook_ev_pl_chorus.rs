// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_chorus(game: &mut crate::game::Game<'_>, teleport: &TeleportConsume) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_chorus") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::chorus(game, |game| game.teleport_randomly(teleport)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_chorus", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
