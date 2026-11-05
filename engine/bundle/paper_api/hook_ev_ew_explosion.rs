// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_explosion(game: &mut crate::game::Game<'_>, dimension: &str, center: [f64; 3], blocks: &mut Vec<[i32; 3]>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_explosion") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", dimension.trim_start_matches("minecraft:")), "center": center, "block": center.map(|value| value.floor() as i32), "exploded": crate::game::paper_fam_ev_world::exploded_block(), "blocks": &*blocks });
        if crate::game::paper_bridge::on_refusable(game, "ew_explosion", payload) {
            return ::core::ops::ControlFlow::Break(crate::game::paper_fam_ev_world::keep_blocks(blocks));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
