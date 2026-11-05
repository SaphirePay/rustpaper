// Written by tools/gen_bundle.py.
fn paper_hook_paper_no_load(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_fam_paper::take_no_load() {
        game.fns.load_pending = false;
    }
}
