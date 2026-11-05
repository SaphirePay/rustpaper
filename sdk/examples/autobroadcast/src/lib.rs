//! AutoBroadcast: a repeating timer from config (`interval-seconds`, `messages`), and a reload command.
use horizon_bukkit::prelude::*;
use std::sync::Mutex;

/// The running timer: `cancel()` stops it.
static TASK: Mutex<Option<BukkitTask>> = Mutex::new(None);

fn start(plugin: Plugin) {
    // `get_config()` reads config.yml again each call, so this is also the reload.
    let config = plugin.get_config();
    let seconds = config.get_int_or("interval-seconds", 300).max(1) as u64;
    // `&` colour codes become `§` ones, which messages take.
    let messages: Vec<String> = config.get_string_list("messages").iter().map(|line| translate_alternate_color_codes('&', line)).collect();
    if let Some(old) = TASK.lock().unwrap().take() {
        old.cancel();
    }
    if messages.is_empty() {
        plugin.warning("no messages in config.yml");
        return;
    }
    let ticks = seconds * 20; // scheduler times are ticks: 20 a second
    let mut next = 0;
    let task = plugin.run_task_timer(ticks, ticks, move || {
        broadcast_message(&messages[next]);
        next = (next + 1) % messages.len();
    });
    *TASK.lock().unwrap() = Some(task);
}

fn on_enable(plugin: Plugin) {
    // Written to plugins/AutoBroadcast/config.yml only the first time.
    plugin.save_default_config(include_str!("../config.yml"));
    start(plugin);
    // `Plugin` is Copy: closures can capture it by value.
    plugin.set_executor("autobroadcast", move |sender: &CommandSender, _label: &str, args: &[&str]| {
        if args.first().is_some_and(|word| word.eq_ignore_ascii_case("reload")) {
            if !sender.has_permission("autobroadcast.reload") {
                sender.send_message("§cYou don't have permission to do that.");
                return true;
            }
            start(plugin);
            sender.send_message("§aAutoBroadcast reloaded.");
            return true;
        }
        false // false: Bukkit shows the command's usage from plugin.yml
    });
}

fn on_disable(_plugin: Plugin) {
    // A disabled plugin's tasks are cancelled anyway; this keeps TASK honest.
    TASK.lock().unwrap().take();
}

bukkit_plugin!(on_enable, on_disable);
