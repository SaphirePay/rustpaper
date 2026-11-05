// Written by tools/gen_bundle.py.
fn paper_hook_crossbow_shot(game: &mut crate::game::Game<'_>) {
    crate::game::paper_bridge::on_rod_like(game, "bow_shot");
}
