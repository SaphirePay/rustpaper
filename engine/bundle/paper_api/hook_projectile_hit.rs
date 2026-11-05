// Written by tools/gen_bundle.py.
fn paper_hook_projectile_hit(game: &mut crate::game::Game<'_>, p: &Projectile) {
    crate::game::paper_bridge::on_projectile_hit(game, p.id, p.kind.id(), [p.position.x, p.position.y, p.position.z], p.guest);
}
