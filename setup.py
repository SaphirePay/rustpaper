#!/usr/bin/env python3
"""Horizon Paper: a Minecraft Java 26.3 server written in Rust that runs Bukkit/Paper plugins written in Rust.

    python setup.py runtime                  download the Horizon server runtime this engine is built for
    python setup.py prepare                  prepare the game files from your own Minecraft (once; Minecraft 26.3
                                             played once from the official launcher, and Java 25 on the PATH)
    python setup.py mixins                   install horizon-mixins (the tool that applies the Paper layer)
    python setup.py engine [--quick|--check] build the Paper engine (out/horizon-client[.exe])
    python setup.py server DIR [--port N] [--offline] [--op NAME] [--world NAME] [--pause] [--nogui]
                                             make a server in DIR that runs this engine (--pause: pause the
                                             ticks while nobody is on, as vanilla; plugins then load at a join;
                                             --nogui: `start` runs it headless)
    python setup.py start DIR [--nogui|--gui]
                                             start a server in the background and open the panel on it
                                             (--nogui: headless, and remembered; --gui: the panel again)
    python setup.py stop DIR                 stop a server
    python setup.py panel [DIR]              open the panel: every server on this computer in one window
                                             (built the first time, about 2 minutes)
    python setup.py plugin NAME [--dir DIR]  start a plugin from the template (default: plugins/NAME)
    python setup.py doctor                   what is installed and what is missing

Everything it downloads is pinned in pins.json. Python 3.9+, no packages needed.
"""
import hashlib
import json
import os
import pathlib
import platform
import shutil
import subprocess
import sys
import tarfile
import urllib.request
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent
PINS = json.loads((ROOT / 'pins.json').read_text(encoding='utf-8'))
WINDOWS = os.name == 'nt'
EXE = '.exe' if WINDOWS else ''
RUNTIME = ROOT / 'runtime'
OUT = ROOT / 'out'
ENGINE = OUT / f'horizon-client{EXE}'
PANEL = OUT / f'rustpaper-panel{EXE}'
RESOURCES = RUNTIME / '.local' / 'horizon-client' / PINS['minecraft']


def registry_file():
    """The panel's list of servers (the panel reads the same file: panel/src/registry.rs)."""
    if WINDOWS:
        base = pathlib.Path(os.environ.get('APPDATA', pathlib.Path.home()))
    elif sys.platform == 'darwin':
        base = pathlib.Path.home() / 'Library' / 'Application Support'
    else:
        base = pathlib.Path(os.environ.get('XDG_CONFIG_HOME', pathlib.Path.home() / '.config'))
    return base / 'rustpaper' / 'servers.json'


def register(folder, gui=None):
    """Puts a server on the panel's list; `gui` True/False remembers whether `start` opens the panel."""
    path = registry_file()
    try:
        data = json.loads(path.read_text(encoding='utf-8-sig'))
    except OSError:
        data = {}
    except ValueError:
        # A list that can't be read is kept aside, never written over: it may hold networks.
        path.replace(path.with_name(f'servers.json.unreadable-{int(__import__("time").time())}'))
        data = {}
    servers = data.setdefault('servers', [])
    key = os.path.normcase(str(folder))
    data['hidden'] = [h for h in data.get('hidden', []) if os.path.normcase(h) != key]
    entry = next((e for e in servers if os.path.normcase(e.get('dir', '')) == key), None)
    if entry is None:
        entry = {'dir': str(folder), 'tool': str(server_tool()), 'gui': True}
        servers.append(entry)
    entry.setdefault('tool', str(server_tool()))
    if gui is not None:
        entry['gui'] = gui
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_suffix('.json.tmp')
    temp.write_text(json.dumps(data, indent=2), encoding='utf-8')
    os.replace(temp, path)
    return entry


def say(*parts):
    print(*parts, flush=True)


def fail(text):
    sys.exit(f'error: {text}')


def run(command, **kw):
    say('>', ' '.join(str(c) for c in command))
    return subprocess.run([str(c) for c in command], check=True, **kw)


def asset_name():
    machine = platform.machine().lower()
    if WINDOWS:
        return 'horizon-windows-x64.zip' if machine in ('amd64', 'x86_64') else 'horizon-windows-arm64.zip'
    if sys.platform.startswith('linux'):
        return 'horizon-server-linux-x86_64.tar.gz'
    fail('the server runtime is published for Windows and Linux (x86_64) only')


def download(url, to):
    say('downloading', url)
    with urllib.request.urlopen(url) as response, open(to, 'wb') as out:
        shutil.copyfileobj(response, out)


def cmd_runtime(_args):
    tag = PINS['horizon_runtime']
    base = f'https://github.com/horizon-rust/horizon/releases/download/{tag}'
    name = asset_name()
    RUNTIME.mkdir(exist_ok=True)
    archive = RUNTIME / name
    if not archive.exists():
        download(f'{base}/{name}', archive)
    sums = RUNTIME / 'SHA256SUMS'
    download(f'{base}/SHA256SUMS', sums)
    want = next((line.split()[0] for line in sums.read_text().splitlines() if line.strip().endswith(name)), None)
    got = hashlib.sha256(archive.read_bytes()).hexdigest()
    if want != got:
        archive.unlink()
        fail(f'{name}: SHA-256 {got} is not the release\'s {want}; deleted, run it again')
    say('checksum ok')
    if name.endswith('.zip'):
        with zipfile.ZipFile(archive) as z:
            z.extractall(RUNTIME)
    else:
        with tarfile.open(archive) as t:
            t.extractall(RUNTIME, filter='data')
    # The archive may hold one top folder: its contents go to runtime/ itself.
    tops = [p for p in RUNTIME.iterdir() if p.is_dir() and p.name not in ('.local',)]
    if not server_tool().exists() and len(tops) == 1:
        for item in tops[0].iterdir():
            shutil.move(str(item), RUNTIME / item.name)
        tops[0].rmdir()
    if not server_tool().exists():
        fail(f'horizon-server{EXE} not found in {RUNTIME}')
    say(f'runtime {tag} in {RUNTIME}')
    if not RESOURCES.is_dir():
        say('next: python setup.py prepare')


def server_tool():
    return RUNTIME / f'horizon-server{EXE}'


def fetch_checked(url, to, sha1):
    if to.exists() and hashlib.sha1(to.read_bytes()).hexdigest() == sha1:
        return
    to.parent.mkdir(parents=True, exist_ok=True)
    download(url, to)
    got = hashlib.sha1(to.read_bytes()).hexdigest()
    if got != sha1:
        to.unlink()
        fail(f'{url}: SHA-1 {got} is not {sha1}')


def mojang_files():
    """Mojang's own 26.3 version file (the one Horizon pins: the official launcher may rewrite its copy) and,
    from it, the 26.3 server jar; both checked by SHA-1."""
    lock = json.loads((RUNTIME / 'research-lock.json').read_text(encoding='utf-8'))['minecraft']
    version, sha1 = lock['version'], lock['manifest_sha1']
    manifest = RUNTIME / '.local' / 'minecraft' / version / 'manifest.json'
    fetch_checked(f'https://piston-meta.mojang.com/v1/packages/{sha1}/{version}.json', manifest, sha1)
    server = json.loads(manifest.read_text(encoding='utf-8'))['downloads']['server']
    jar = RUNTIME / '.local' / 'minecraft' / version / 'server.jar'
    fetch_checked(server['url'], jar, server['sha1'])
    return jar


def cmd_prepare(args):
    tool = RUNTIME / f'horizon-prepare{EXE}'
    if not tool.exists():
        fail('run `python setup.py runtime` first')
    if not shutil.which('java'):
        fail('a Java 25 runtime is needed to prepare the game files (https://adoptium.net), on the PATH')
    jar = mojang_files()
    if '--server-jar' in args:
        jar = args[args.index('--server-jar') + 1]
    command = [tool, '--prepare', '--server-jar', jar, '--root', RUNTIME]
    if '--minecraft-dir' in args:
        command += ['--minecraft-dir', args[args.index('--minecraft-dir') + 1]]
    run(command)
    if not RESOURCES.is_dir():
        fail(f'the game files were not made in {RESOURCES}')
    say('game files ready:', RESOURCES)


def cmd_mixins(_args):
    if not shutil.which('cargo'):
        fail('Rust is not installed: https://rustup.rs')
    have = subprocess.run(['rustup', 'run', PINS['rust'], 'rustc', '--version'], capture_output=True).returncode == 0
    if not have:
        run(['rustup', 'toolchain', 'install', PINS['rust'], '--profile', 'minimal', '--no-self-update'])
    run(['cargo', f'+{PINS["rust"]}', 'install', '--locked', '--git', 'https://github.com/horizon-rust/horizon-mixins',
         '--rev', PINS['horizon_mixins'], 'horizon-mixins-cli', '--root', ROOT / 'tools-bin'])


def cmd_engine(args):
    mode = [a for a in args if a in ('--quick', '--check')]
    if not (ROOT / 'tools-bin' / 'bin' / f'horizon-mixins{EXE}').exists() and not os.environ.get('HORIZON_MIXINS'):
        cmd_mixins([])
    run([sys.executable, ROOT / 'engine' / 'tools' / 'build_engine.py', *mode, PINS['horizon'], ENGINE, ROOT / 'engine' / 'bundle' / 'paper_api'])


def option(args, name, default=None):
    return args[args.index(name) + 1] if name in args else default


def cmd_server(args):
    if not args or args[0].startswith('--'):
        fail('usage: python setup.py server DIR [--port N] [--offline] [--op NAME] [--world NAME]')
    folder = pathlib.Path(args[0]).resolve()
    if not server_tool().exists():
        fail('run `python setup.py runtime` first')
    if not RESOURCES.is_dir():
        fail('run `python setup.py prepare` first')
    if not ENGINE.exists():
        fail('run `python setup.py engine` first')
    folder.mkdir(parents=True, exist_ok=True)
    init = [server_tool(), '--dir', folder, 'init', '--eula', '--world', option(args, '--world', 'world'), '--port', option(args, '--port', '25565'), '--no-whitelist', '--yes']
    if '--offline' in args:
        init.append('--offline')
    if '--op' in args:
        init += ['--op', option(args, '--op')]
    if '--seed' in args:
        init += ['--seed', option(args, '--seed')]
    run(init)
    # The server checks plugins/ on its ticks, and vanilla pauses the ticks while nobody is on: a development
    # server keeps ticking, so a dropped plugin loads at once (`--pause` keeps vanilla's pause).
    settings = [f'engine.path={ENGINE}', f'engine.resources={RESOURCES}']
    if '--pause' not in args:
        settings.append('properties.pause-when-empty-seconds=0')
    run([server_tool(), '--dir', folder, 'config', 'set', *settings])
    (folder / 'plugins').mkdir(exist_ok=True)
    register(folder, False if '--nogui' in args else None)
    say('')
    say(f'server ready in {folder}')
    say(f'  start:     python setup.py start {args[0]}      (opens the panel; --nogui runs it headless)')
    say(f'  or:        {server_tool()} --dir {folder} run   (in this terminal: type console commands, Ctrl-C stops)')
    say(f'  plugins:   copy .{"dll" if WINDOWS else "so"} files into {folder / "plugins"} (they load while it runs)')


def server_folder(args, usage):
    if not args or args[0].startswith('--'):
        fail(f'usage: python setup.py {usage}')
    folder = pathlib.Path(args[0]).resolve()
    if not (folder / 'server.toml').is_file():
        fail(f'{folder} is not a server (no server.toml): make one with `python setup.py server {args[0]}`')
    return folder


def cmd_start(args):
    folder = server_folder(args, 'start DIR [--nogui|--gui]')
    if not server_tool().exists():
        fail('run `python setup.py runtime` first')
    gui = False if '--nogui' in args else True if '--gui' in args else None
    entry = register(folder, gui)
    run([server_tool(), '--dir', folder, 'start'])
    if entry.get('gui', True):
        open_panel(folder)
    else:
        say('running headless (the panel stays closed for this server: `python setup.py start DIR --gui` changes that)')
        say(f'  open the panel any time: python setup.py panel {args[0]}')
        say(f'  stop it:                 python setup.py stop {args[0]}')


def cmd_stop(args):
    folder = server_folder(args, 'stop DIR')
    run([server_tool(), '--dir', folder, 'stop'])


def build_panel():
    if not shutil.which('cargo'):
        fail('Rust is not installed: https://rustup.rs')
    target = OUT / 'panel-target'
    say('building the panel (once, about 2 minutes)')
    run(['cargo', f'+{PINS["rust"]}', 'build', '--release', '--manifest-path', ROOT / 'panel' / 'Cargo.toml', '--target-dir', target])
    OUT.mkdir(exist_ok=True)
    shutil.copy2(target / 'release' / f'rustpaper-panel{EXE}', PANEL)


def open_panel(folder=None):
    if not PANEL.exists() or PANEL.stat().st_mtime < max(p.stat().st_mtime for p in (ROOT / 'panel').rglob('*') if p.is_file()):
        build_panel()
    command = [str(PANEL)] + (['--select', str(folder)] if folder else [])
    flags = {}
    if WINDOWS:
        flags['creationflags'] = 0x00000008 | 0x00000200  # DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
    else:
        flags['start_new_session'] = True
    subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, **flags)
    say('the panel is open' + (f' on {folder.name}' if folder else ''))


def cmd_panel(args):
    folder = None
    if args and not args[0].startswith('--'):
        folder = server_folder(args, 'panel [DIR]')
        register(folder)
    if '--build' in args:
        build_panel()
    open_panel(folder)


def cmd_plugin(args):
    if not args:
        fail('usage: python setup.py plugin NAME [--dir DIR]')
    name = args[0]
    crate = ''.join(c if c.isalnum() else '_' for c in name.lower()).strip('_') or 'plugin'
    folder = pathlib.Path(option(args, '--dir', ROOT / 'plugins' / crate)).resolve()
    if folder.exists() and any(folder.iterdir()):
        fail(f'{folder} is not empty')
    shutil.copytree(ROOT / 'templates' / 'plugin', folder, dirs_exist_ok=True)
    sdk = (ROOT / 'sdk' / 'horizon-bukkit').as_posix()
    for path in folder.rglob('*'):
        if path.is_file():
            text = path.read_text(encoding='utf-8')
            text = text.replace('{{NAME}}', name).replace('{{CRATE}}', crate).replace('{{SDK}}', sdk).replace('{{COMMAND}}', crate)
            path.write_text(text, encoding='utf-8')
    say(f'plugin {name} in {folder}')
    say(f'  build:   cd {folder} && cargo build --release')
    say(f'  install: copy target/release/{"" if WINDOWS else "lib"}{crate}.{"dll" if WINDOWS else "so"} into your server\'s plugins/')


def cmd_doctor(_args):
    rows = [
        ('Rust (cargo)', bool(shutil.which('cargo'))),
        (f'Rust {PINS["rust"]} toolchain', subprocess.run(['rustup', 'run', PINS['rust'], 'rustc', '--version'], capture_output=True).returncode == 0 if shutil.which('rustup') else False),
        ('runtime (horizon-server)', server_tool().exists()),
        ('game files (prepared)', RESOURCES.is_dir()),
        ('horizon-mixins', (ROOT / 'tools-bin' / 'bin' / f'horizon-mixins{EXE}').exists() or bool(shutil.which('horizon-mixins'))),
        ('Paper engine built', ENGINE.exists()),
        ('panel built (optional: `python setup.py panel` builds it)', PANEL.exists()),
    ]
    for what, ok in rows:
        say(f'  [{"x" if ok else " "}] {what}')


COMMANDS = {'runtime': cmd_runtime, 'prepare': cmd_prepare, 'mixins': cmd_mixins, 'engine': cmd_engine, 'server': cmd_server,
            'start': cmd_start, 'stop': cmd_stop, 'panel': cmd_panel, 'plugin': cmd_plugin, 'doctor': cmd_doctor}

if __name__ == '__main__':
    if len(sys.argv) < 2 or sys.argv[1] not in COMMANDS:
        print(__doc__)
        sys.exit(1)
    COMMANDS[sys.argv[1]](sys.argv[2:])
