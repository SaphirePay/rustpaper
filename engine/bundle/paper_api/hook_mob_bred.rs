// Written by tools/gen_bundle.py.
fn paper_hook_mob_bred(game: &mut crate::game::Game<'_>, parent: u32, partner: u32) {
    crate::game::paper_bridge::on_mob_bred(game, parent, partner);
}
