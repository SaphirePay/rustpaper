// Written by tools/gen_bundle.py.
fn paper_hook_ev_core_dragon_tick(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("core_dragon_tick") {
        let payload = serde_json::json!({ "done": ({
    if game.dim == Dim::End {
        game.load_fight();
        for order in crate::game::paper_fam_core::take_dragon_orders() {
            match order.as_str() {
                "respawn" => game.try_respawn(),
                "reset" => game.reset_spike_crystals(),
                "gateway" => {
                    if let Some(g) = game.fight.gateways.pop() {
                        let a = 2.0 * (-std::f64::consts::PI + std::f64::consts::PI / 20.0 * f64::from(g));
                        let pos = [(96.0 * a.cos()).floor() as i32, 75, (96.0 * a.sin()).floor() as i32];
                        game.level_event_at(3000, pos, 0);
                        let _ = game.place_gateway_at(pos);
                        game.fight.dirty = true;
                    }
                }
                "end" => {
                    if game.fight.respawn.is_some() {
                        game.finish_respawn();
                    }
                }
                "none" => {
                    if game.fight.respawn.is_some() {
                        game.abort_respawn();
                    }
                }
                phase => {
                    if game.fight.respawn.is_some() && let Some(stage) = Respawn::from_name(phase) {
                        game.set_respawn(Some(stage));
                    }
                }
            }
        }
        let crystals = game.fight.respawn_crystals.iter().map(|at| [at.x, at.y, at.z]).collect();
        crate::game::paper_fam_core::note_dragon(game.fight.gateways.len(), crystals);
    }
    0
}) });
        crate::game::paper_bridge::on_trigger(game, "core_dragon_tick", payload);
    }
}
