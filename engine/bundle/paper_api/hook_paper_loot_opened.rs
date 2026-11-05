// Written by tools/gen_bundle.py.
fn paper_hook_paper_loot_opened(game: &mut crate::game::Game<'_>, key: (i32, i32, i32)) {
    crate::game::paper_fam_paper::loot_opened(game, key);
}
