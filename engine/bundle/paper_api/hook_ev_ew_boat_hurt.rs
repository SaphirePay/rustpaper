// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_boat_hurt(game: &mut crate::game::Game<'_>, i: usize, damage: f32, creative: bool) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("ew_boat_hurt") {
        let payload = serde_json::json!({ "uuid": if game.lan_acting.is_some() { serde_json::json!(format!("{:032x}", game.native_acting_id())) } else { serde_json::Value::Null }, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "vehicle": game.boats[i].id, "damage": damage, "creative": creative, "breaks": creative || game.boats[i].damage + damage * 10.0 > 40.0 });
        if crate::game::paper_bridge::on_refusable(game, "ew_boat_hurt", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
