// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_player_dying(game: &mut crate::game::Game<'_>, host: &mut LanHost, gi: usize, conn: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_player_dying") {
        let payload = serde_json::json!({ "ee_refused": ({ let cause = host.play.get(&conn).and_then(|play| play.death_cause.clone()); crate::game::paper_fam_ev_entity::player_dying(game, host, gi, cause) }) });
        if crate::game::paper_bridge::on_refusable(game, "ee_player_dying", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
