// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_painting_break(game: &mut crate::game::Game<'_>, index: usize, by_player: bool) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_painting_break") {
        let payload = serde_json::json!({ "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "entity": game.paintings[index].id, "by_player": by_player, "survives": ({ let p = game.paintings[index].clone(); game.painting_survives(&p) }) });
        if crate::game::paper_bridge::on_refusable(game, "ew_painting_break", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
