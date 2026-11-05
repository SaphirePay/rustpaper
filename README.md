# Paper 26.3 Rewritten in Rust

Paper, rewritten in Rust: a Minecraft Java Edition 26.3 server built on [Horizon](https://github.com/horizon-rust/horizon)
that loads Bukkit and Paper plugins written in Rust, from `plugins/`, by their `plugin.yml`, the way Paper loads
jars. This repo has everything to run one and to write plugins for it:

| Folder | What |
|---|---|
| `setup.py` | one script: download the runtime, prepare the game files, build the engine, make a server, start a plugin |
| `engine/` | the Paper layer (`bundle/paper_api`), the engine rows (`family/`) and the build tools |
| `sdk/` | what plugins build on: `horizon-bukkit` (Bukkit/Paper's API in Rust), `horizon-mod`, and `examples/` |
| `templates/plugin/` | a new plugin, ready to build |
| `docs/` | [PLUGINS.md](docs/PLUGINS.md) (writing plugins), [SERVER.md](docs/SERVER.md) (running a server) |
| `pins.json` | the exact versions everything is built from |

This is an unofficial project, not made, endorsed or supported by PaperMC, Mojang or Microsoft. It contains no
Minecraft code or assets: the game files are prepared from your own copy of Minecraft Java Edition.

## Make a server (about 20 minutes the first time)

You need: [Rust](https://rustup.rs), Python 3.10+, Git, Minecraft Java Edition 26.3 played once from the official
launcher, and a Java 25 runtime (only to prepare the game files). Linux also needs `pkg-config libasound2-dev
libudev-dev` (Debian/Ubuntu names).

```bash
git clone https://github.com/SaphirePay/paper-rust.git
cd paper-rust
python setup.py runtime                                  # the Horizon server runtime (checked by SHA-256)
python setup.py prepare                                  # game files from your Minecraft (Java 25 on the PATH)
python setup.py engine                                   # builds the Paper engine (~5 min on a fast PC, first time)
python setup.py server myserver --offline --op YourName  # a server in ./myserver
runtime/horizon-server --dir myserver run                # Windows: runtime\horizon-server.exe
```

Join at `localhost:25565` with a Minecraft Java 26.3 client. `prepare` downloads Mojang's 26.3 server jar and version
file (both checked by their SHA-1) and reads the rest from your installed Minecraft. Drop out `--offline` for a public server (players must own
Minecraft). More in [docs/SERVER.md](docs/SERVER.md).

`python setup.py doctor` shows what is done and what is missing.

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
