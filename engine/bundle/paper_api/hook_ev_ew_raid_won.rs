// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_raid_won(game: &mut crate::game::Game<'_>, id: i32, action: &Action) {
    if crate::game::paper_bridge::hears("ew_raid_won") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "raid": id, "winners": match action { Action::Victory { heroes, .. } => Some(heroes.iter().map(|hero| format!("{:032x}", horizon_client::lan::playerdata::ints_uuid(*hero))).collect::<Vec<_>>()), _ => None } });
        crate::game::paper_bridge::on_trigger(game, "ew_raid_won", payload);
    }
}
