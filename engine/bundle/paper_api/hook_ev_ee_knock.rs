// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_knock(game: &mut crate::game::Game<'_>, power: f64, xd: f64, zd: f64) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_knock") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::knock(game, power, xd, zd) });
        if crate::game::paper_bridge::on_refusable(game, "ee_knock", payload) {
            return ::core::ops::ControlFlow::Break(crate::game::paper_fam_ev_entity::knock_again(game));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
