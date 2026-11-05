// Written by tools/gen_bundle.py.
fn paper_hook_command_message(game: &mut crate::game::Game<'_>, id: &str, ctx: &Context) -> ::core::ops::ControlFlow<R<i32>> {
    let message = match ctx.get(if id == "me" { "action" } else { "message" }) {
        Some(Arg::Str(text)) => text.as_str(),
        _ => "",
    };
    let (sender, name) = game.message_source();
    let kind = if id == "say" { "say_command" } else { "emote_command" };
    if id == "msg" || horizon_client::paper_hub::allow_command(kind, message, sender, &name) {
        ::core::ops::ControlFlow::Continue(())
    } else {
        ::core::ops::ControlFlow::Break(Ok(1))
    }
}
