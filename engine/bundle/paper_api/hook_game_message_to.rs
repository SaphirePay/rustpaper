// Written by tools/gen_bundle.py.
fn paper_hook_game_message_to(hub: &Hub, content: &Tag) -> ::core::ops::ControlFlow<()> {
    if let Some(line) = crate::paper_hub::take_death_line() {
        if let Some(line) = line {
            hub.system_message(line);
        }
        return ::core::ops::ControlFlow::Break(());
    }
    if crate::paper_hub::allow_game(&hub.english(content)) {
        ::core::ops::ControlFlow::Continue(())
    } else {
        ::core::ops::ControlFlow::Break(())
    }
}
