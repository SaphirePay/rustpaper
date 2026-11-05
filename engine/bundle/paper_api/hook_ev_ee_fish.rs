// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_fish(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("ee_fish") {
        let payload = serde_json::json!({ "level": crate::game::paper_fam_ev_entity::here(game), "uuid": format!("{:032x}", game.native_acting_id()), "hook": game.hook.as_ref().map(|hook| hook.id), "state": game.hook.as_ref().map(|hook| format!("{:?}", hook.state)) });
        crate::game::paper_bridge::on_trigger(game, "ee_fish", payload);
    }
}
