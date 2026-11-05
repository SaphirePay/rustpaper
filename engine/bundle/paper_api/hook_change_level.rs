// Written by tools/gen_bundle.py.
fn paper_hook_change_level(hub: &Hub, i: usize) {
    let g = &hub.guests[i];
    crate::paper_hub::push(crate::paper_hub::Pending::ChangedLevel { player: g.uuid, name: g.name.clone(), from: g.dimension.to_string(), to: hub.config.dimension.to_string() });
}
