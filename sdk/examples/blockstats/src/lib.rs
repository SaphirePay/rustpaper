//! BlockStats: counting events per player, saved to a file in the data folder.
use horizon_bukkit::chat_color::{GOLD, RED, YELLOW};
use horizon_bukkit::enums::event_priority;
use horizon_bukkit::event::{BlockBreakEvent, PlayerQuitEvent};
use horizon_bukkit::{CommandSender, Plugin, bukkit_plugin, get_player};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// UUID -> (last known name, blocks broken).
static COUNTS: Mutex<Option<HashMap<String, (String, u64)>>> = Mutex::new(None);

fn file(plugin: Plugin) -> PathBuf {
    plugin.get_data_folder().join("counts.txt")
}

fn with_counts<R>(f: impl FnOnce(&mut HashMap<String, (String, u64)>) -> R) -> R {
    let mut guard = COUNTS.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(HashMap::new))
}

fn load(plugin: Plugin) {
    let mut counts = HashMap::new();
    if let Ok(text) = std::fs::read_to_string(file(plugin)) {
        for line in text.lines() {
            let mut parts = line.split('\t');
            if let (Some(uuid), Some(name), Some(count)) = (parts.next(), parts.next(), parts.next())
                && let Ok(count) = count.trim().parse::<u64>()
            {
                counts.insert(uuid.to_owned(), (name.to_owned(), count));
            }
        }
    }
    *COUNTS.lock().unwrap_or_else(|e| e.into_inner()) = Some(counts);
}

fn save(plugin: Plugin) {
    let text = with_counts(|counts| {
        let mut rows: Vec<_> = counts.iter().collect();
        rows.sort();
        rows.iter().map(|(uuid, (name, count))| format!("{uuid}\t{name}\t{count}\n")).collect::<String>()
    });
    let path = file(plugin);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(error) = std::fs::write(&path, text) {
        plugin.severe(&format!("could not save {}: {error}", path.display()));
    }
}

fn on_command(sender: &CommandSender, _label: &str, args: &[&str]) -> bool {
    let (name, count) = match args.first() {
        None => match sender.as_player() {
            Some(player) => {
                let uuid = player.get_unique_id();
                (player.get_name().to_owned(), with_counts(|counts| counts.get(&uuid).map_or(0, |row| row.1)))
            }
            None => {
                sender.send_message(&format!("{RED}Usage: /blockstats <player>"));
                return true;
            }
        },
        Some(wanted) => {
            let online = get_player(wanted);
            let found = with_counts(|counts| match &online {
                Some(player) => Some((player.get_name().to_owned(), counts.get(&player.get_unique_id()).map_or(0, |row| row.1))),
                None => counts.values().find(|(name, _)| name.eq_ignore_ascii_case(wanted)).cloned(),
            });
            match found {
                Some(row) => row,
                None => {
                    sender.send_message(&format!("{RED}No stats for player {wanted}."));
                    return true;
                }
            }
        }
    };
    sender.send_message(&format!("{GOLD}{name}{YELLOW} has broken {GOLD}{count}{YELLOW} blocks."));
    true
}

fn on_enable(plugin: Plugin) {
    load(plugin);
    plugin.set_executor("blockstats", on_command);
    plugin.register_event_with(event_priority::MONITOR, true, |event: &mut BlockBreakEvent| {
        let player = event.get_player();
        let (uuid, name) = (player.get_unique_id(), player.get_name().to_owned());
        with_counts(|counts| {
            let row = counts.entry(uuid).or_insert_with(|| (name.clone(), 0));
            row.0 = name;
            row.1 += 1;
        });
    });
    plugin.register_event(move |_event: &mut PlayerQuitEvent| save(plugin));
}

fn on_disable(plugin: Plugin) {
    save(plugin);
}

bukkit_plugin!(on_enable, on_disable);
