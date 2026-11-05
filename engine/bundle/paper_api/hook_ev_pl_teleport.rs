// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_teleport(game: &mut crate::game::Game<'_>, t: Target, pos: [f64; 3], rotation: Option<[f32; 2]>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_teleport") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::teleport_gate(game, t == Target::Player, pos, rotation, |game, pos, rotation| game.teleport_to_in_level(Target::Player, pos, rotation)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_teleport", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
