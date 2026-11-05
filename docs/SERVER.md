# Running a Horizon Paper server

A server is a folder made by `python setup.py server DIR`. It runs the Paper engine (`out/horizon-client`) with the
game files prepared from your Minecraft (`runtime/.local/horizon-client/26.3`), through `horizon-server`, the
Horizon runtime's supervisor. Below, `hs` stands for `runtime/horizon-server` (`runtime\horizon-server.exe` on
Windows).

## Day to day

| Task | Command |
|---|---|
| Run in the foreground (type console commands, Ctrl-C stops it) | `hs --dir myserver run` |
| Run in the background / stop / restart | `hs --dir myserver start`, `stop`, `restart` |
| TPS, tick time (MSPT), players, RAM, CPU | `hs --dir myserver status` |
| A console command on the running server | `hs --dir myserver cmd "say hello"` |
| Live log, stats and a command line | `hs --dir myserver console` (Ctrl-D detaches) |
| The last log lines | `hs --dir myserver logs -n 100` (`-f` follows) |
| Ops and the whitelist | `hs --dir myserver op NAME`, `whitelist on\|off\|add NAME\|remove NAME` |
| Settings | `hs --dir myserver config` and `config set KEY=VALUE` (for example `world.difficulty=hard server.max_players=50`) |
| Backups | `hs --dir myserver backup [NAME]`, `backups`, `restore BACKUP` |
| Start at boot (systemd on Linux, a logon task on Windows) | `hs --dir myserver service install` |
| A performance report | `hs --dir myserver report 60` |

Settings live in `myserver/server.toml`; `hs --dir myserver import server.properties` takes a vanilla server's.

## Plugins

- Put plugin libraries in `myserver/plugins/`: `.dll` on Windows, `.so` on Linux. Each one's data folder is
  `plugins/<Name>/`, as on Paper.
- A new or changed file loads while the server runs (the old copy is disabled first); a removed file's plugin is
  turned off. The folder is checked on the server's ticks: `setup.py server` turns off vanilla's pause while
  nobody is on (`pause-when-empty-seconds=0`) so this works on an empty test server too. A public server can
  have the pause back: `hs --dir myserver config set properties.pause-when-empty-seconds=60`.
- `/hotswap NAME` (operators) reloads one plugin now, changed or not.
- The log says `[PaperEngine] server started: N plugins enabled` once the plugins are on.
- Plugins load in Paper's order from their `plugin.yml` (`depend`, `softdepend`, `loadbefore`), and are refused as
  Paper refuses them (a missing dependency, two of one name, a dependency loop), with Paper's messages.

## Updating the engine

`git pull`, `python setup.py engine`, then `hs --dir myserver restart`. The server folder points at `out/`, so a
rebuilt engine is what it runs next. Rebuild your plugins too when `sdk/` changed.

## A public server

- Leave out `--offline` when making it (or set it back with `config set`), so players sign in with their
  Microsoft accounts.
- Open the port (25565 by default) in the firewall.
- On Linux, `hs --dir myserver service install --system` keeps it running across reboots.
