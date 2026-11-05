// Written by tools/gen_bundle.py.
fn paper_hook_menu_load(g: &Guest) {
    horizon_client::paper_hub::set_menu_player(Some(g.uuid));
}
