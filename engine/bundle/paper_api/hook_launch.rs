// Written by tools/gen_bundle.py.
fn paper_hook_launch() {
    crate::game::paper_events::at_launch();
    crate::game::paper_plugins::launch();
}
