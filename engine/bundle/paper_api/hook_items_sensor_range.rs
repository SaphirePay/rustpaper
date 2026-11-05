// Written by tools/gen_bundle.py.
fn paper_hook_items_sensor_range(entity: &SculkEntity) -> ::core::ops::ControlFlow<i32> {
    if entity.warning_level > 0 && matches!(entity.kind, SculkKind::Sensor | SculkKind::Calibrated) {
        ::core::ops::ControlFlow::Break(entity.warning_level)
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
