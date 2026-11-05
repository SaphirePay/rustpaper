# Horizon Paper engine

A Horizon server that runs Bukkit and Paper plugins rewritten in Rust, the way Paper runs their jars. There is no
Fabric in it: no Fabric loader, no `fabric.mod.json`, no Fabric API events or mod APIs, no `mods/` folder.

## Plugins

- A plugin is a library (`.dll` / `.so`) in the server's `plugins/` folder, built on `sdk/horizon-bukkit`. Its
  data folder is `plugins/<Name>/`, as on Paper.
- It describes itself from its own `plugin.yml` (`bukkit_plugin!` / `java_plugin!` embed it): `name`, `version`,
  `depend`, `softdepend`, `loadbefore`, `provides`, or `paper-plugin.yml`'s `dependencies.server`
  (`required`, `load: BEFORE|AFTER|OMIT`).
- `bundle/paper_api/paper_plugins.rs` loads them in Paper's order and refuses what Paper refuses, with Paper's
  messages: a missing `depend` (and every plugin that needs it), two plugins of one name (the first file wins),
  a loop of `depend`s (a loop of soft rules is broken instead). The server goes on without them.
- ProtocolLib and PacketEvents are the engine's own (`family/protocol_engine.py`, `family/packetevents_engine.py`):
  a plugin that depends on one loads.
- Plugin interface 3 (`sdk/horizon-mod`). A library built for the old Fabric engine (interface 2) is refused:
  "a Fabric engine library: rebuild it on the Paper SDK". Rebuilding is all it takes: plugin source doesn't change.
- `[PaperEngine] server started: N plugins enabled` once the worlds are open and the plugins are on.

## Layout

| Path | What |
|---|---|
| `bundle/paper_api/` | the plugin layer, applied to Horizon with horizon-mixins: `paper_events.rs` (the hook points, each a plain call), `paper_bridge.rs` (events to plugin libraries), `paper_plugins.rs` (the loader), `paper_hub.rs`, `paper_query.rs`, `paper_fam_*.rs` (the Bukkit families), `paper_entity_events.rs`, `runtime/` (the library loader) |
| `family/` | the rows: engine edits a mixin can't make, applied by `tools/dim_patch.py` in file-name order |
| `../sdk/horizon-bukkit`, `../sdk/horizon-mod` | what plugins build on |
| `tools/` | `build_engine.py`, `dim_patch.py`, `gen_bundle.py` (writes `horizon-mixin.toml` and the hook helpers), `event_hooks.py` |

## Build

From the repo root, `python setup.py engine` does all of it. By hand:

```
python -I engine/tools/build_engine.py --check <commit> out/horizon-client engine/bundle/paper_api   # type-check only
python -I engine/tools/build_engine.py --quick <commit> out/horizon-client engine/bundle/paper_api   # a test engine
python -I engine/tools/build_engine.py <commit> out/horizon-client engine/bundle/paper_api           # optimized
python -I engine/tools/gen_bundle.py target/source/<commit>/native-hooks.json <commit>               # after changing a hook table
```

The commit is `horizon` in `pins.json`. horizon-mixins (`horizon_mixins` in `pins.json`, installed by
`python setup.py mixins`) reformats only the files it injects into, so a row's text is matched with any whitespace,
`//` comments and trailing commas when the exact text isn't there (`dim_patch.loose_span`). Check rows against a
fresh apply, never the already-patched stage.
## Still to do

- `paper_entity_events.rs` keeps three pipelines that only Fabric's events used and that are now inert (held
  mob effects, mob sleep, the asks before a hit): each returns "go ahead" and the hub holds nothing for them.
  They can be deleted with their rows.
- `video_recorder.rs` is the client's recorder: a server doesn't use it.
