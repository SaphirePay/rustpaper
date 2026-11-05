// Written by tools/gen_bundle.py.
fn paper_hook_paper_cart_loot(game: &mut crate::game::Game<'_>, i: usize) {
    if game.carts[i].loot.is_some() {
        let id = game.carts[i].id;
        crate::game::paper_fam_paper::cart_loot_opened(game, id);
    }
}
