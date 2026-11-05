// Written by tools/gen_bundle.py.
fn paper_hook_chat(hub: &Hub, i: usize, message: &ChatMessage) -> ::core::ops::ControlFlow<()> {
    let g = &hub.guests[i];
    if crate::paper_hub::allow_chat(g.uuid, &g.name, &message.content) {
        ::core::ops::ControlFlow::Continue(())
    } else {
        ::core::ops::ControlFlow::Break(())
    }
}
