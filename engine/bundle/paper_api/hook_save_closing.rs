// Written by tools/gen_bundle.py.
fn paper_hook_save_closing(game: &mut crate::game::Game<'_>) {
    crate::game::paper_events::closing_save_begins(game, true);
}
