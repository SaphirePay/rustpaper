// Written by tools/gen_bundle.py.
fn paper_hook_guest_command(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32, command: &str) -> ::core::ops::ControlFlow<()> {
    let who = host.hub.guests.iter().find(|g| g.conn == conn).map(|g| (g.uuid, g.name.clone()));
    match who {
        Some((uuid, name)) if crate::game::paper_bridge::on_command_preprocess(game, uuid, &name, command) => ::core::ops::ControlFlow::Break(()),
        _ => ::core::ops::ControlFlow::Continue(()),
    }
}
