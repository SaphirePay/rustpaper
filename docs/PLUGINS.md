# Writing plugins for Horizon Paper

A plugin is a Rust library (`.dll` on Windows, `.so` on Linux) built on `sdk/horizon-bukkit`, which mirrors the
Bukkit and Paper API. You build it, copy the file into a server's `plugins/` folder, and it loads while the server
runs.

## What you need

| Item | What |
|---|---|
| Rust | 1.99.1 or newer, edition 2024 ([rustup.rs](https://rustup.rs)) |
| This repo | `sdk/` is what plugins build on |
| A test server | `python setup.py server testserver --offline --op YourName` (see the main README) |
| A C compiler | only for crates with C code (SQLite and the like): MSVC Build Tools on Windows, gcc on Linux |
| Docker | only to build a Linux `.so` from Windows |

## Your first plugin

`python setup.py plugin MyPlugin` makes `plugins/myplugin` from `templates/plugin`. By hand, a plugin is three files.

`Cargo.toml` (the path is this repo's `sdk/horizon-bukkit`):

```toml
[package]
name = "heal"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
horizon-bukkit = { path = "../../sdk/horizon-bukkit" }

[profile.release]
panic = "unwind"

[workspace]
```

`plugin.yml`, next to `Cargo.toml`, is Bukkit's (`name`, `version`, `depend`, `softdepend`, `loadbefore`,
`commands`, `permissions`). It is built into the library:

```yaml
name: Heal
version: 1.0
main: heal
commands:
  heal:
    description: Heal yourself
    usage: /heal
permissions:
  heal.use:
    default: op
```

`src/lib.rs`:

```rust
use horizon_bukkit::event::PlayerJoinEvent;
use horizon_bukkit::{CommandSender, Plugin, bukkit_plugin};

fn heal(sender: &CommandSender, _label: &str, _args: &[&str]) -> bool {
    let Some(player) = sender.as_player() else {
        sender.send_message("§cOnly players can heal.");
        return true;
    };
    if !sender.has_permission("heal.use") {
        sender.send_message("§cNo permission.");
        return true;
    }
    player.set_health(player.get_max_health());
    player.set_food_level(20);
    sender.send_message("§aHealed.");
    true
}

fn on_enable(plugin: Plugin) {
    plugin.set_executor("heal", heal);
    plugin.register_event(|event: &mut PlayerJoinEvent| {
        let name = event.get_player().get_name().to_owned();
        event.set_join_message(Some(&format!("§e{name} joined")));
    });
}

fn on_disable(_plugin: Plugin) {}

bukkit_plugin!(on_enable, on_disable);
```

- `bukkit_plugin!(on_enable, on_disable)` is the entry point, like `JavaPlugin.onEnable`/`onDisable`. A plugin
  written as a class can use `java_plugin!(MyPlugin::default())`.
- Every command needs its entry in `plugin.yml`, as in Bukkit.
- More examples: `sdk/examples/heal`, `menu` (a chest GUI), `autobroadcast`, `joinmessage`, `blockstats`.

## Build and install

| For | Build | File |
|---|---|---|
| A Windows server | `cargo build --release` | `target/release/heal.dll` |
| A Linux server, built on Linux | `cargo build --release` | `target/release/libheal.so` |
| A Linux server, built on Windows | `docker run --rm -v "<repo>/sdk:/sdk:ro" -v "%cd%:/src" rust:1.99.1-bookworm bash /src/build_linux.sh` (the template's script) | `out/libheal.so` |

Copy the file into the server's `plugins/`. It loads within seconds; copying a new build over the old one disables
the old one (its `on_disable` runs) and enables the new one. A Windows build won't load on Linux or the other way
round. Keep `panic = "unwind"`: a panic in a plugin is then caught and logged.

## The API

The SDK uses Bukkit's names in snake_case; most Java lines have a direct Rust line. Each SDK item's doc comment
names the Java it stands for (`/// bukkit: org/bukkit/...`), so search `sdk/horizon-bukkit/src` for the Java class
or method you want.

| Java (Bukkit/Paper) | Rust (horizon-bukkit) |
|---|---|
| `onEnable` / `onDisable` | `bukkit_plugin!(on_enable, on_disable)` |
| `getCommand("x").setExecutor(...)` | `plugin.set_executor("x", \|sender, label, args\| { ...; true })` |
| `setTabCompleter(...)` | `plugin.set_tab_completer("x", \|sender, label, args\| vec![...])` |
| `@EventHandler void on(BlockBreakEvent e)` | `plugin.register_event(\|e: &mut BlockBreakEvent\| { ... })` |
| `e.setCancelled(true)` | `e.set_cancelled(true)` |
| `runTask` / `runTaskLater` / `runTaskTimer` | `plugin.run_task(...)`, `run_task_later(ticks, ...)`, `run_task_timer(delay, period, ...)` |
| `runTaskAsynchronously` | `plugin.run_task_asynchronously(...)` |
| `saveDefaultConfig()` / `getConfig()` | `plugin.save_default_config(include_str!("../config.yml"))`, `plugin.get_config()` |
| `config.getString/getInt/getBoolean/getStringList/getKeys/set` | `get_string`, `get_int`, `get_boolean`, `get_string_list`, `get_keys`, `set`, then `plugin.save_config(&config)` |
| `getDataFolder()` | `plugin.get_data_folder()` |
| `getLogger().info/warning/severe` | `plugin.info`, `plugin.warning`, `plugin.severe` |
| `Bukkit.getPlayer(name)` / `getOnlinePlayers()` | `horizon_bukkit::get_player(name)`, `horizon_bukkit::get_online_players()` |
| `Bukkit.getWorld("world")` | `horizon_bukkit::get_world("world")` |
| `world.getBlockAt(x, y, z)` | `world.get_block_at(x, y, z)` |
| `block.getType()` / `getBlockData().getAsString()` | `block.get_type()`, `block.get_block_data()` (a string: `minecraft:oak_stairs[facing=east,half=bottom]`) |
| `block.setBlockData(...)` | `block.set_block_data("minecraft:stone")` |
| `new ItemStack(Material.DIAMOND, 1)` | `ItemStack::new(Material::of("DIAMOND"), 1)` |
| `Bukkit.createInventory(null, 27, "Menu")` | `Inventory::create(27, "Menu")`, then `player.open_inventory(&inv)` |
| `ChatColor.GREEN` | `horizon_bukkit::chat_color::GREEN` (or `§a` in the text) |
| Adventure `Component` | `Component(serde_json::json!(...))`, `player.send_component(&c)` (click and hover text) |

`use horizon_bukkit::prelude::*;` brings the common types in at once. 381 Bukkit and Paper events can be listened
to (block, entity, player, inventory, world, server); each is a struct in the SDK, found by its Java name.

## Threads and speed

Events, commands and `run_task*` callbacks run on the server thread: what they spend is added to the tick.

1. **Game calls only on the server thread.** `Player`, `World`, `Block` and inventory methods ask the server and
   fail on other threads. Sending a chat message and your own file I/O are safe anywhere.
2. **Slow work off the thread.** Files, HTTP, databases, heavy maths: `plugin.run_task_asynchronously(...)` or a
   thread of your own. Read what it needs from the game first, on the server thread.
3. **Come back with `run_task`.** From any thread, `plugin.run_task(move || ...)` runs on the server thread at the
   next tick.
4. **Every game call is a round trip.** Read a value once and keep it, rather than asking again in a loop.
5. **Stop your threads in `on_disable`.** A reload unloads your library right after `on_disable` returns: a
   thread still running then crashes the server. Signal it and `join()` it.
6. **Shared state behind `Mutex` or atomics.**

```rust
plugin.register_event(move |e: &mut PlayerJoinEvent| {
    let name = e.get_player().get_name().to_owned();
    plugin.run_task_asynchronously(move || {
        let greeting = std::fs::read_to_string("greeting.txt").unwrap_or_default(); // slow: off the tick
        plugin.run_task(move || {
            if let Some(p) = horizon_bukkit::get_player(&name) {
                p.send_message(&greeting); // a game call: back on the server thread
            }
        });
    });
});
```

## Porting a Java plugin

1. Decompile the jar (Vineflower works well). Keep its `plugin.yml`, `config.yml` and language files as they are,
   so existing servers' files keep working.
2. Port the biggest classes first (listeners, command handlers); most lines map one to one (the API table).
3. Match its messages and file formats exactly: admins compare against the Java one.

## Testing

Make a test server (`python setup.py server testserver --offline --op YourName`), run it, join at
`localhost:25565`, and copy each build into `testserver/plugins/`. The log shows your plugin's `info`/`warning`
lines. Check the cost with `runtime/horizon-server --dir testserver status`: the tick time (MSPT) with your plugin
should be no higher than without it (wait a minute after starting before you compare).

## Traps

- **Each plugin has its own copy of the SDK.** A static in the SDK is not shared between plugins; talk to another
  plugin through its commands, events or files.
- **Writes land at the end of the tick.** Most world changes go through the server's command path: reading a
  value straight after setting it can still show the old one.
- **One world per dimension.** `world`, `world_nether`, `world_the_end`; `createWorld` does not exist yet.
- **Approximate methods are marked.** A doc comment tagged `bukkit~` works but differs from Bukkit somehow, and
  says how.
- **Materials and block states are the game's ids.** `Material::of("DIAMOND")` = `Material::of("minecraft:diamond")`;
  block data is the game's string form.
- **Rebuild after `sdk/` changes**, before installing on a server running the newer engine.
