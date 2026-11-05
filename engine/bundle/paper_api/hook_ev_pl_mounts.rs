// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_mounts(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("pl_mounts") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::mounts_are(|host, conn| host.play.get(&conn)?.mount_window.as_ref().map(|mount| (mount.window, mount.mob, mount.columns)), |game, index, columns| game.mount_engine_menu(index, columns), |game, index, columns, menu| game.lan_mount_store(index, columns, menu)) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_mounts", payload);
    }
}
