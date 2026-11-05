//! JoinMessage: a join listener, a config with defaults, & colour codes.
use horizon_bukkit::chat_color::translate_alternate_color_codes;
use horizon_bukkit::event::PlayerJoinEvent;
use horizon_bukkit::{Plugin, bukkit_plugin};

const DEFAULT_CONFIG: &str = include_str!("../config.yml");

fn colour(text: &str, name: &str) -> String {
    translate_alternate_color_codes('&', &text.replace("%player%", name))
}

fn on_enable(plugin: Plugin) {
    plugin.save_default_config(DEFAULT_CONFIG);
    let config = plugin.get_config();
    let welcome = config.get_string_or("welcome", "&6Welcome, %player%!");
    let broadcast = config.get_string_or("join-broadcast", "&7+ %player%");
    plugin.register_event(move |event: &mut PlayerJoinEvent| {
        let name = event.get_player().get_name().to_owned();
        event.set_join_message(Some(&colour(&broadcast, &name)));
        event.get_player().send_message(&colour(&welcome, &name));
    });
}

fn on_disable(_plugin: Plugin) {}

bukkit_plugin!(on_enable, on_disable);
