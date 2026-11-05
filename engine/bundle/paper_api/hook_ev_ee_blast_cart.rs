// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_blast_cart(game: &mut crate::game::Game<'_>, c: &Minecart) {
    if crate::game::paper_bridge::hears("ee_blast_cart") {
        let payload = serde_json::json!({ "ee_why": crate::game::paper_fam_ev_entity::blast_by_cart(c.id) });
        crate::game::paper_bridge::on_trigger(game, "ee_blast_cart", payload);
    }
}
