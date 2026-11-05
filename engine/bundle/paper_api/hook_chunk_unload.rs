// Written by tools/gen_bundle.py.
fn paper_hook_chunk_unload(game: &mut crate::game::Game<'_>, dimension: &str, chunk: [i32; 2]) {
    crate::game::paper_events::on_chunk_unload(game, dimension, chunk);
}
