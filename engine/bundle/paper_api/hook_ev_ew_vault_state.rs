// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_vault_state(game: &mut crate::game::Game<'_>, key: Pos, state: &VaultState) -> ::core::ops::ControlFlow<Result<()>> {
    if crate::game::paper_bridge::hears("ew_vault_state") {
        let payload = serde_json::json!({ "uuid": if game.lan_acting.is_some() { serde_json::json!(format!("{:032x}", game.native_acting_id())) } else { serde_json::Value::Null }, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "pos": [key.0, key.1, key.2], "new": state.name(), "old": game.pack.assets.registry.state(game.world.block(key.0, key.1, key.2)).and_then(|block| block.properties.get("vault_state").cloned()) });
        if crate::game::paper_bridge::on_refusable(game, "ew_vault_state", payload) {
            return ::core::ops::ControlFlow::Break(Ok(()));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
