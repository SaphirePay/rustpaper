// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_bed_blast(game: &mut crate::game::Game<'_>, head: [i32; 3]) -> ::core::ops::ControlFlow<Result<()>> {
    if crate::game::paper_bridge::hears("pl_bed_blast") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::bed_blast(game, head, Ok(())) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_bed_blast", payload) {
            return ::core::ops::ControlFlow::Break(Ok(()));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
