// Written by tools/gen_bundle.py.
fn paper_hook_flight_flags(value: i8, g: &Guest) -> i8 {
    crate::paper_hub::ability_flags(g.uuid, value)
}
