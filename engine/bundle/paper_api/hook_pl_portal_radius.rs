// Written by tools/gen_bundle.py.
fn paper_hook_pl_portal_radius(game: &crate::game::Game<'_>, center: BlockPos) -> ::core::ops::ControlFlow<Vec<BlockPos>> {
    match crate::game::paper_fam_ev_player::portal_radius() {
        Some(radius) => ::core::ops::ControlFlow::Break(game.portal_blocks(center, radius)),
        None => ::core::ops::ControlFlow::Continue(()),
    }
}
