// Written by tools/gen_bundle.py.
fn paper_hook_command_chat(hub: &Hub, i: usize, chat_type: &str, text: &str) -> ::core::ops::ControlFlow<()> {
    let guest = &hub.guests[i];
    if crate::paper_hub::allow_command(chat_type, text, Some(guest.uuid), &guest.name) {
        ::core::ops::ControlFlow::Continue(())
    } else {
        ::core::ops::ControlFlow::Break(())
    }
}
