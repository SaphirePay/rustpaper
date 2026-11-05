// Written by tools/gen_bundle.py.
fn paper_hook_chunk_load(game: &mut crate::game::Game<'_>, dimension: &str, chunk: [i32; 2], new: bool) {
    crate::game::paper_events::on_chunk_load(game, dimension, chunk, new);
}
