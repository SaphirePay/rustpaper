# Writing a Horizon plugin in Rust (for AIs and people)

Horizon runs Paper plugins natively: a plugin is a Rust DLL built on `horizon-bukkit`, which is the Bukkit/Paper API
with the same classes and **Java names in snake_case** (`player.setHealth(20)` -> `player.set_health(20.0)`). If you
know how to write the plugin in Java for Paper, you know how to write it here. This file is all you need to start;
look anything else up instead of guessing:

- **`API_INDEX.md`** (beside this file): every Bukkit/Paper method the SDK has -> its Rust function and signature,
  one line each as `Class.method -> rust::path — signature`. Grep it for the Java name you know:
  `grep "Player.setHealth" API_INDEX.md`, `grep "BossBar\." API_INDEX.md`. `API_INDEX.tsv` has the same rows as
  tab-separated columns: java_class, java_method, rust_path, signature, approx (`~` = works, not exact).
- **`../examples/`**: tested plugins (`heal`, `joinmessage`, `autobroadcast`, `blockstats`, `menu`). Copy one.
- **`tools/plugin_dev.py`** (in the engine repo, also an MCP server with `--mcp`): `lookup`, `search`, `event`,
  `example`, `new`, `build` (errors with their usual fix), `test` (runs it on a real server), `deploy` (hot reload).

## 1. A plugin's files

```
myplugin/
  Cargo.toml     name = folder name; crate-type cdylib; horizon-bukkit path dep; [workspace]
  plugin.yml     exactly as on Paper: name, version, main (any string), commands, permissions
  config.yml     optional: defaults, included with include_str!
  src/lib.rs
```

`Cargo.toml`:
```toml
[package]
name = "myplugin"
version = "1.0.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
horizon-bukkit = { path = "PATH/TO/sdk/horizon-bukkit" }

[profile.release]
panic = "unwind"

[workspace]
```

Build: `cargo build --release` (share one `CARGO_TARGET_DIR` between plugins: the SDK builds once, then a plugin takes
about a second). The DLL is `target/release/<name>.dll`: put it in the server's `plugins/` (a running server swaps it
in by itself: hot reload).

## 2. The skeleton

```rust
use horizon_bukkit::prelude::*;   // every event type and the common classes: no other imports needed

fn on_enable(plugin: Plugin) {
    plugin.save_default_config(include_str!("../config.yml"));   // Java's saveDefaultConfig()
    plugin.set_executor("hello", |sender: &CommandSender, _label: &str, args: &[&str]| {
        sender.send_message(&format!("§aHello, {}!", sender.get_name()));
        true    // false = Bukkit prints the command's usage from plugin.yml
    });
    plugin.register_event(|event: &mut PlayerJoinEvent| {
        event.get_player().send_message("§6Welcome!");
    });
}

fn on_disable(_plugin: Plugin) {}   // runs when the server stops or the plugin is swapped out

bukkit_plugin!(on_enable, on_disable);
```

`Plugin` is `Copy`: capture it by value in closures (`move |...|`).

## 3. The rules that differ from Java

| Java | Rust here |
|---|---|
| `getFoo()` / `setFoo(x)` / `isFoo()` | `get_foo()` / `set_foo(x)` / `is_foo()` |
| getters that can be null | return `Option<T>`: `Bukkit.getPlayer("x")` -> `get_player("x") -> Option<Player>` |
| `UUID` | `String` (`player.get_unique_id()`) |
| enums (`Material.DIAMOND`, `EventPriority.HIGH`, `Sound.X`) | their **names as text**: `Material::of("DIAMOND")`, `enums::event_priority::HIGH`, `"ENTITY_PLAYER_LEVELUP"` |
| `ChatColor.GREEN + "hi"`, `&` codes | text takes `§` codes: `"§ahi"`; turn `&` codes into `§` with `translate_alternate_color_codes('&', s)` |
| `saveDefaultConfig()` | `plugin.save_default_config(include_str!("../config.yml"))` (you pass the default text) |
| `getConfig().getInt("a", 5)` | `plugin.get_config().get_int_or("a", 5)`; also `get_string_or`, `get_boolean_or`, `get_string_list` |
| `config.set("a", 5); saveConfig()` | `let mut c = plugin.get_config(); c.set("a", 5.into()); plugin.save_config(&c);` |
| `reloadConfig()` | just call `plugin.get_config()` again: it reads the file each time |
| `getDataFolder()` | `plugin.get_data_folder()` (a `PathBuf`, made if missing): `std::fs` reads and writes in it |
| `Bukkit.getScheduler().runTaskTimer(plugin, r, delay, period)` | `plugin.run_task_timer(delay, period, move \|\| ...)` -> a task; `.cancel()` stops it. Times are **ticks** (20 a second) |
| `runTaskLater` / `runTask` / `runTaskAsynchronously` | `plugin.run_task_later(ticks, f)` / `plugin.run_task(f)` / `plugin.run_task_asynchronously(f)` |
| `@EventHandler(priority = HIGH, ignoreCancelled = true)` | `plugin.register_event_with(enums::event_priority::HIGH, true, \|e: &mut E\| ...)` |
| `event.setCancelled(true)` | the same: `event.set_cancelled(true)` |
| fields of your plugin class | `static NAME: Mutex<T> = Mutex::new(...)` (or `LazyLock`), since listeners must be `Send + Sync` |
| `sender instanceof Player` | `if let Some(player) = sender.as_player() { ... }` |
| `getCommand("x").setTabCompleter(...)` | `plugin.set_tab_completer("x", \|sender, label, args\| vec![...])` |

## 4. Recipes

**Players**: `get_player(name)`, `get_online_players()`, `player.get_name()`, `player.get_health()`,
`player.set_health(player.get_max_health())`, `player.set_food_level(20)`, `player.get_location()`,
`player.teleport(&location)`, `player.get_world().get_spawn_location()`, `player.has_permission("x.y")`,
`player.get_inventory().add_item(&stack)`, `player.send_title(title, subtitle, 10, 70, 20)`,
`player.play_sound(&player.get_location(), "ENTITY_PLAYER_LEVELUP", 1.0, 1.0)`, `broadcast_message("§etext")`.

**Items**: `let mut stack = ItemStack::new(Material::of("DIAMOND"), 1); stack.set_display_name("§bName");`

**A chest GUI** (see `examples/menu`): make it once (`Inventory::create(27, "Title")`, `set_item(slot, Some(stack))`),
keep it in a `static`, open it with `player.open_inventory(&menu)`, and in the click listener:
```rust
plugin.register_event(|event: &mut InventoryClickEvent| {
    if !event.get_inventory().is(&menu()) { return; }   // Java: event.getInventory() != menu
    event.set_cancelled(true);                           // nothing moves in or out
    let Some(player) = event.get_who_clicked() else { return };
    if event.get_raw_slot() == 11 { player.close_inventory(); }   // closing inside the click is fine
});
```

**Data that outlives the server**: read a file in `on_enable`, write it in `on_disable` and when it changes
(`PlayerQuitEvent`). A YAML file other than config: `YamlConfiguration::load_configuration(&path)`, then
`get_*`/`set`/`save(Some(&path))`.

**State shared by listeners**: `static COUNTS: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());` then
`COUNTS.lock().unwrap().entry(uuid).or_default()`.

**Locations in a config**: `config.set_location("warps.home", &player.get_location())`, then
`config.get_location("warps.home") -> Option<Location>`.

**JSON / nested values**: `json!` comes with the prelude: `config.set("spawn", json!({"x": 1, "y": 64}))`.

**Chat**: `AsyncPlayerChatEvent` (in the prelude): `get_message()`, `set_message(..)`, `set_cancelled(true)`,
`get_player()`. It runs off the server thread: change the message there, and do game things (teleports, items) in
a `plugin.run_task(..)`.

**Deaths**: `PlayerDeathEvent` (the prelude's): `event.get_player()` is the player who died (a `&Player`).

**Boss bars and titles**: `BossBar::create("Starting in 10", "GREEN")` (the colour by its Bukkit name),
`bar.set_progress(0.5)` (0 to 1), `bar.add_player(&p)`, `bar.remove_all()` (takes the bar away);
`player.send_title("GO!", "", 10, 40, 10)`.

**Worlds**: `World::of("world")` (or `"minecraft:overworld"`, `"world_nether"`); `world.get_name() -> &str`
(`.to_string()` for a `String`).

**Timers that run more than once** take an `FnMut` closure: it may be called many times, so don't move a captured
value out of it; clone it inside (`let name = name.clone();`) or keep state in a `static Mutex`.

## 5. When something doesn't compile

- `cannot find type X`: you didn't `use horizon_bukkit::prelude::*;`, or X lives elsewhere: search `API_INDEX.md`.
- `no method named foo`: the Rust name differs; look up the Java one in `API_INDEX.md` (it gives the signature).
- `expected &str, found String`: pass `&s`. `expected String, found &str`: `.to_string()`.
- a closure "may outlive" or "is not Send": `move` it, keep shared state in a `static Mutex` or `Arc<Mutex<..>>`.
- `expected bool, found ()`: a command executor returns `bool`; end with `true`.

## 6. Good to know

- Commands need their entry in `plugin.yml` (`commands:`), as on Paper. A `permission:` there is checked for you;
  check any second permission (`heal.others`) yourself with `sender.has_permission`.
- Writes made inside an event (a teleport, a block change) take effect as the event ends, as Paper's do after
  your handler returns; reading the same thing back inside the handler shows the old value.
- Each plugin DLL has its own statics: two plugins share data through the server (commands, files, services),
  not through Rust statics.
