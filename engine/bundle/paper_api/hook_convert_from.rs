// Written by tools/gen_bundle.py.
fn paper_hook_convert_from(game: &crate::game::Game<'_>, from: &Mob, kind: &str, params: horizon_client::mobs::conversion::Params) {
    if game.res.mob_types().get(kind).is_some() {
        crate::game::paper_events::on_converting(from.id, from.kind.id.as_ref(), params.keep_equipment);
    }
}
