// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_add_orb(game: &mut crate::game::Game<'_>, at: [f64; 3], value: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_add_orb") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::breed_orb(game) });
        if crate::game::paper_bridge::on_refusable(game, "ee_add_orb", payload) {
            return ::core::ops::ControlFlow::Break(game.add_orb(at, crate::game::paper_fam_ev_entity::breed_amount(value)));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
