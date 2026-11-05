// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_splash(game: &mut crate::game::Game<'_>, stack: &ItemStack, at: Vec3, mob: Option<Vec3>, age: u32, owner: &Option<Text>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_splash") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::splash(game, stack, at, mob, age, owner) });
        if crate::game::paper_bridge::on_refusable(game, "ee_splash", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
