// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_merchant_closed(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, window: Option<i32>) {
    if crate::game::paper_bridge::hears("items_merchant_closed") {
        let payload = serde_json::json!({ "uuid": crate::game::paper_bridge::uuid_of(host, conn), "closed": (crate::game::paper_fam_items::menu_closed(host, conn, window)) });
        crate::game::paper_bridge::on_trigger(game, "items_merchant_closed", payload);
    }
}
