// Written by tools/gen_bundle.py.
fn paper_hook_chat_decorate(hub: &mut Hub, message: &ChatMessage, bound: &play::ChatTypeBound, line: String, events: &mut Vec<HostEvent>, original: impl FnOnce(&mut Hub, &ChatMessage, &play::ChatTypeBound, String, &mut Vec<HostEvent>) -> ()) {
    match crate::paper_hub::decorate(message.sender, &message.content) {
        Some(decorated) => {
            let mut message = message.clone();
            message.unsigned_content = Some(decorated);
            original(hub, &message, bound, line, events);
        }
        None => original(hub, message, bound, line, events),
    }
}
