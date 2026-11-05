// Written by tools/gen_bundle.py.
fn paper_hook_flight_wanted(hub: &mut Hub, i: usize) {
    let g = &mut hub.guests[i];
    if let Some(flying) = crate::paper_hub::flight_answer(g.uuid) {
        g.flying = flying;
    }
    for (packet, _) in &g.inbox {
        if let S::PlayerAbilities(p) = packet {
            crate::paper_hub::flight_wanted(g.uuid, p.flags & 2 != 0);
        }
        crate::paper_hub::note_packet(g.uuid, packet);
    }
}
