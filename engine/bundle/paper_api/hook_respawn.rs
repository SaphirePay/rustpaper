// Written by tools/gen_bundle.py.
fn paper_hook_respawn(hub: &Hub, i: usize) {
    let g = &hub.guests[i];
    crate::paper_hub::push(crate::paper_hub::Pending::Respawned { player: g.uuid, name: g.name.clone() });
}
