// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_merchant(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, gm: &mut GuestMerchant) {
    if crate::game::paper_bridge::hears("items_merchant") {
        let payload = serde_json::json!({ "uuid": "", "open": ({ let who = host.hub.guest(conn).map(|g| g.uuid); match who { Some(who) => crate::game::paper_fam_items::merchant_sent(game, who, gm.window, gm.merchant.mob, &gm.slots, gm.merchant.session.active, &mut gm.merchant.session.offers), None => serde_json::Value::Null } }) });
        crate::game::paper_bridge::on_trigger(game, "items_merchant", payload);
    }
}
