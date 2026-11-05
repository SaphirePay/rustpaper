// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_cart_hurt(game: &mut crate::game::Game<'_>, i: usize, damage: f32, creative: bool, fire: bool) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_cart_hurt") {
        let payload = serde_json::json!({ "uuid": if game.lan_acting.is_some() && !fire { serde_json::json!(format!("{:032x}", game.native_acting_id())) } else { serde_json::Value::Null }, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "vehicle": game.carts[i].id, "damage": damage, "creative": creative, "fire": fire, "breaks": creative || game.carts[i].damage + damage * 10.0 > 40.0 });
        if crate::game::paper_bridge::on_refusable(game, "ew_cart_hurt", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
