// Written by tools/gen_bundle.py.
fn paper_hook_block_spread(dimension: crate::game::LevelDimension, kind: &str, from: horizon_core::position::BlockPos, to: horizon_core::position::BlockPos, canceled: &mut bool) {
    crate::game::paper_events::on_block_spread(dimension, kind, [from.x, from.y, from.z], [to.x, to.y, to.z], canceled);
}
