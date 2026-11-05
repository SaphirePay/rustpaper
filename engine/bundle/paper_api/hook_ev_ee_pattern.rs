// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_pattern(game: &mut crate::game::Game<'_>, cells: &[[i32; 3]]) {
    if crate::game::paper_bridge::hears("ee_pattern") {
        let payload = serde_json::json!({ "ee_cells": crate::game::paper_fam_ev_entity::note_pattern(game, cells) });
        crate::game::paper_bridge::on_trigger(game, "ee_pattern", payload);
    }
}
