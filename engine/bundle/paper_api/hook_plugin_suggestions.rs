// Written by tools/gen_bundle.py.
fn paper_hook_plugin_suggestions(value: Commands) -> Commands {
    crate::paper_hub::ask_server_for_plugins(value)
}
