// Written by tools/gen_bundle.py.
fn paper_hook_container_opened(game: &mut crate::game::Game<'_>, conn: u32, pos: (i32, i32, i32)) {
    crate::game::paper_bridge::on_container_opened(game, conn, pos);
}
