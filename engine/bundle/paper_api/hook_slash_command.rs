// Written by tools/gen_bundle.py.
fn paper_hook_slash_command(host: &mut LanHost, conn: u32, command: &str) -> ::core::ops::ControlFlow<()> {
    // `//set` arrives as `/set`: the mod's that took `/set` (WorldEdit's), which trimming every slash would
    // turn into `/set`'s.
    if !command.starts_with('/') || command.starts_with("//") {
        return ::core::ops::ControlFlow::Continue(());
    }
    let Some(level) = Game::native_command_level(host, command) else { return ::core::ops::ControlFlow::Continue(()) };
    let Some(permission) = host.hub.guest(conn).map(|g| g.permission) else { return ::core::ops::ControlFlow::Continue(()) };
    if permission < level {
        return ::core::ops::ControlFlow::Continue(());
    }
    let integrated = host.hub.config.integrated;
    host.commands.push(GuestCommand { conn, command: command.to_owned(), permission, integrated, quiet: false });
    ::core::ops::ControlFlow::Break(())
}
