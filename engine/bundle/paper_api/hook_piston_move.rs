// Written by tools/gen_bundle.py.
fn paper_hook_piston_move(piston: horizon_core::position::BlockPos, direction: [i32; 3], push: &[horizon_core::position::BlockPos], canceled: &mut bool) {
    crate::game::paper_bridge::on_piston(piston, direction, push, canceled);
}
