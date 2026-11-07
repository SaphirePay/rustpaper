// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_no_wear(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<()> {
    // Every event on this hook answers from `ee_refused` alone (`verdict` first: tools' echoes check), so its listeners never ran here; the round trip to each plugin is gone.
    if crate::game::paper_bridge::hears("ee_no_wear") && crate::game::paper_fam_ev_entity::skip_wear() {
        return ::core::ops::ControlFlow::Break(());
    }
    ::core::ops::ControlFlow::Continue(())
}
