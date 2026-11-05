// Written by tools/gen_bundle.py.
fn paper_hook_block_changed(dimension: crate::game::LevelDimension, pos: horizon_core::position::BlockPos, old: horizon_world::registry::BlockStateId, new: horizon_world::registry::BlockStateId) {
    crate::game::paper_events::on_block_changed(dimension, pos, old, new);
}
