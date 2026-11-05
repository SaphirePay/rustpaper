// Written by tools/gen_bundle.py.
fn paper_hook_explosion_detonate(game: &mut crate::game::Game<'_>, dimension: &str, center: [f64; 3], blocks: &mut Vec<[i32; 3]>) {
    crate::game::paper_bridge::on_explosion(game, dimension, center, blocks);
}
