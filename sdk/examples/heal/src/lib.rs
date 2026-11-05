//! Heal: a command with an optional player argument and two permissions.
use horizon_bukkit::chat_color::{GREEN, RED};
use horizon_bukkit::{CommandSender, Player, Plugin, bukkit_plugin, get_player};

fn heal(player: &Player) {
    player.set_health(player.get_max_health());
    player.set_food_level(20);
    player.set_saturation(20.0);
}

fn on_heal(sender: &CommandSender, _label: &str, args: &[&str]) -> bool {
    if !sender.has_permission("heal.use") {
        sender.send_message(&format!("{RED}You don't have permission to use this command."));
        return true;
    }
    let target = match args.first() {
        None => match sender.as_player() {
            Some(player) => player.clone(),
            None => {
                sender.send_message(&format!("{RED}Usage: /heal <player>"));
                return true;
            }
        },
        Some(name) => {
            if sender.as_player().is_some_and(|me| me.get_name().eq_ignore_ascii_case(name)) {
                sender.as_player().unwrap().clone()
            } else {
                if !sender.has_permission("heal.others") {
                    sender.send_message(&format!("{RED}You don't have permission to heal other players."));
                    return true;
                }
                match get_player(name) {
                    Some(player) => player,
                    None => {
                        sender.send_message(&format!("{RED}Unknown player: {name}"));
                        return true;
                    }
                }
            }
        }
    };
    heal(&target);
    sender.send_message(&format!("{GREEN}Healed {}.", target.get_name()));
    true
}

fn on_enable(plugin: Plugin) {
    plugin.set_executor("heal", on_heal);
}

fn on_disable(_plugin: Plugin) {}

bukkit_plugin!(on_enable, on_disable);
