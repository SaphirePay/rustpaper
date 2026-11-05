// Written by tools/gen_bundle.py.
fn paper_hook_ev_enchant_offers(game: &mut crate::game::Game<'_>, host: &LanHost, gi: usize) {
    if crate::game::paper_bridge::hears("enchant_offers") {
        let payload = serde_json::json!({ "uuid": format!("{:032x}", host.hub.guests[gi].uuid) });
        crate::game::paper_bridge::on_trigger(game, "enchant_offers", payload);
    }
}
