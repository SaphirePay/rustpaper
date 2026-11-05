# Paper 26.3 Rewritten in Rust

Paper, rewritten in Rust: a Minecraft Java Edition 26.3 server built on [Horizon](https://github.com/horizon-rust/horizon)
that loads Bukkit and Paper plugins written in Rust, from `plugins/`, by their `plugin.yml`, the way Paper loads
jars. This repo has everything to run one and to write plugins for it:

| Folder | What |
|---|---|
| `setup.py` | one script: download the runtime, prepare the game files, build the engine, make a server, start a plugin |
| `engine/` | the Paper layer (`bundle/paper_api`), the engine rows (`family/`) and the build tools |
| `sdk/` | what plugins build on: `horizon-bukkit` (Bukkit/Paper's API in Rust), `horizon-mod`, and `examples/` |
| `panel/` | the server panel: one window for every server on your computer ([docs/PANEL.md](docs/PANEL.md)) |
| `templates/plugin/` | a new plugin, ready to build |
| `docs/` | [PLUGINS.md](docs/PLUGINS.md) (writing plugins), [SERVER.md](docs/SERVER.md) (running a server), [PANEL.md](docs/PANEL.md) (the panel) |
| `pins.json` | the exact versions everything is built from |

This is an unofficial project, not made, endorsed or supported by PaperMC, Mojang or Microsoft. It contains no
Minecraft code or assets: the game files are prepared from your own copy of Minecraft Java Edition.

## Make a server (about 20 minutes the first time)

The same repo runs natively on Windows and Linux (x86_64): `setup.py` downloads the server runtime built for your
system and builds the engine, the panel and your plugins on your own computer.

**Linux, in one command** (Ubuntu, Debian, Fedora, Arch, openSUSE): it installs the build tools and Rust,
downloads this repo, the runtime and a Java runtime, builds the engine and the panel, and prepares the game files
if Minecraft is installed:

```bash
curl -fsSL https://raw.githubusercontent.com/SaphirePay/rustpaper/main/install.sh | bash
```

The game files are always made from your own copy of Minecraft Java Edition 26.3 (played once from the official
launcher); nothing of Minecraft is downloaded. On a server without a desktop, prepare them on a computer that has
Minecraft and copy its `runtime/.local/horizon-client/26.3` folder over.

**Step by step** (Windows, or Linux by hand). You need: [Rust](https://rustup.rs), Python 3.10+, Git, and
Minecraft Java Edition 26.3 played once from the official launcher (`python setup.py java` fetches the Java
runtime that preparing uses). Linux also needs `pkg-config libasound2-dev libudev-dev` (Debian/Ubuntu names).

```bash
git clone https://github.com/SaphirePay/rustpaper.git
cd rustpaper
python setup.py runtime                                  # the Horizon server runtime (checked by SHA-256)
python setup.py prepare                                  # game files from your Minecraft (Java 25 on the PATH)
python setup.py engine                                   # builds the Paper engine (~5 min on a fast PC, first time)
python setup.py server myserver --offline --op YourName  # a server in ./myserver
python setup.py start myserver                           # starts it and opens the panel (--nogui: headless)
```

Join at `localhost:25565` with a Minecraft Java 26.3 client. `prepare` downloads Mojang's 26.3 server jar and version
file (both checked by their SHA-1) and reads the rest from your installed Minecraft. Drop out `--offline` for a public server (players must own
Minecraft). More in [docs/SERVER.md](docs/SERVER.md).

`python setup.py doctor` shows what is done and what is missing.

## The panel

`python setup.py start myserver` opens the panel: one window for every server on your computer, instead of one
window per server. It shows each server's TPS, tick time, players, memory and CPU with graphs, its console and
chat (type as the server), and its players (kick, ban, op, whitelist), plugins (upload, replace, turn off, delete,
and a built-in editor for their config files), backups and settings. Servers can be grouped into networks, with
one page for the whole network: start or stop all, one chat, and bans on every server at once.

Headless works as always: `python setup.py start myserver --nogui` (remembered for that server), or run it in a
terminal with `runtime/horizon-server --dir myserver run`. The panel finds servers that are already running by
itself, and closing it leaves every server running. More in [docs/PANEL.md](docs/PANEL.md).

## Write a plugin (5 minutes)

```bash
python setup.py plugin Hello          # plugins/hello: Cargo.toml, plugin.yml, src/lib.rs
cd plugins/hello
cargo build --release
cp target/release/hello.dll ../../myserver/plugins/    # Linux: libhello.so
```

The server loads it within seconds, without a restart; type `/hello` in game. Copy a new build over the old one
to reload it. Everything else (the API, events, threads, porting a Java plugin, the traps) is in
[docs/PLUGINS.md](docs/PLUGINS.md).

## Engine development

`engine/README.md` explains the Paper layer and its rows. `python setup.py engine --check` type-checks a change in
a minute or two; `--quick` builds a test engine faster than the optimized one.

## License

[GPL-3.0](LICENSE): use it, change it, sell it. If you share a changed version (or a plugin built on the SDK),
share its source under GPL-3.0 too.
