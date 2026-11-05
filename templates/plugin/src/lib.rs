//! {{NAME}}: a starting point. `/{{COMMAND}}` answers, and players are greeted when they join.
use horizon_bukkit::event::PlayerJoinEvent;
use horizon_bukkit::{CommandSender, Plugin, bukkit_plugin};

fn hello(sender: &CommandSender, _label: &str, args: &[&str]) -> bool {
    if !sender.has_permission("{{CRATE}}.use") {
        sender.send_message("§cYou don't have permission to do that.");
        return true;
    }
    sender.send_message(&format!("§aHello from {{NAME}}! You gave {} argument(s).", args.len()));
    true
}

fn on_enable(plugin: Plugin) {
    plugin.set_executor("{{COMMAND}}", hello);
    plugin.register_event(|event: &mut PlayerJoinEvent| {
        let name = event.get_player().get_name().to_owned();
        event.get_player().send_message(&format!("§eWelcome, {name}! This server runs {{NAME}}."));
    });
    plugin.info("{{NAME}} enabled");
}

fn on_disable(_plugin: Plugin) {}

bukkit_plugin!(on_enable, on_disable);
