// Written by tools/gen_bundle.py.
fn paper_hook_join_line(hub: &Hub, packet: &C) -> ::core::ops::ControlFlow<()> {
    if let C::SystemChat(chat) = packet && (crate::paper_hub::hold_line(&hub.english(&chat.content), &chat.content) || !crate::paper_hub::allow_game(&hub.english(&chat.content))) {
        return ::core::ops::ControlFlow::Break(());
    }
    ::core::ops::ControlFlow::Continue(())
}
