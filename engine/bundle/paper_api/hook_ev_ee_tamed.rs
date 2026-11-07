// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_tamed(game: &mut crate::game::Game<'_>, name: &str, target: &EntityView) -> ::core::ops::ControlFlow<()> {
    // Every event on this hook answers from `ee_refused` alone (`verdict` first: tools' echoes check), so its listeners never ran here; the round trip to each plugin is gone.
    if crate::game::paper_bridge::hears("ee_tamed") && crate::game::paper_fam_ev_entity::tamed(game, name, &target.kind, target.position) {
        return ::core::ops::ControlFlow::Break(());
    }
    ::core::ops::ControlFlow::Continue(())
}
