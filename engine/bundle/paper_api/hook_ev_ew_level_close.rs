// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_level_close(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<Result<()>> {
    if crate::game::paper_bridge::hears("ew_level_close") {
        let payload = serde_json::json!({ "uuid": null, "closing": ({ let occupied = game.levels_to_tick(); game.levels.parked.iter().filter(|(dim, parked)| **dim != Dim::Overworld && parked.idle + 1 >= IDLE_TICKS && !occupied.contains(*dim)).map(|(dim, _)| format!("minecraft:{}", dim.id().trim_start_matches("minecraft:"))).collect::<Vec<_>>() }) });
        if crate::game::paper_bridge::on_refusable(game, "ew_level_close", payload) {
            return ::core::ops::ControlFlow::Break({ for parked in game.levels.parked.values_mut() { parked.idle = 0; } Ok(()) });
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
