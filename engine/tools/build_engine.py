#!/usr/bin/env python3
"""Build the Horizon Paper engine: a Horizon server with the Paper plugin layer compiled in.

Usage: build_engine.py [--check | --quick] COMMIT OUT_EXE BUNDLE_DIR [BUNDLE_DIR...]

Takes the pinned commit's source from GitHub, applies the bundles with horizon-mixins into a
stage, compiles the server build (no audio or controllers) at below-normal priority, and
copies the result to OUT_EXE. Unchanged files keep their times, so a rebuild is incremental.

  --check   only type-checks: no program, the fastest way to find a mistake
  --quick   a test engine: little optimization, many code units and a faster linker
  (neither) the optimized build, for a real server

Each mode keeps its own cache in target/, so switching doesn't throw work away.
"""
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tarfile
import time
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parent.parent
TARGET = ROOT / 'target'
TOOLCHAIN = '+1.99.1'
BELOW_NORMAL = 0x00004000


def log(*args):
    print(*args, flush=True)


def source(commit):
    tree = TARGET / 'source' / commit
    if (tree / 'Cargo.toml').is_file():
        return tree
    archive = TARGET / f'{commit}.tar.gz'
    TARGET.mkdir(exist_ok=True)
    if not archive.exists():
        log('downloading the source of', commit[:9])
        urllib.request.urlretrieve(f'https://codeload.github.com/horizon-rust/horizon/tar.gz/{commit}', archive)
    log('unpacking')
    with tarfile.open(archive) as tar:
        tar.extractall(TARGET / 'source', filter=lambda member, path: None if member.issym() or member.islnk() else tarfile.data_filter(member, path))
    (TARGET / 'source' / f'horizon-{commit}').rename(tree)
    return tree


def sync(fresh, stage):
    """Makes `stage` match `fresh`, writing only changed files: cargo goes by file times."""
    changed = 0
    wanted = set()
    for path in fresh.rglob('*'):
        relative = path.relative_to(fresh)
        wanted.add(relative)
        target = stage / relative
        if path.is_dir():
            target.mkdir(parents=True, exist_ok=True)
        elif not target.is_file() or target.read_bytes() != path.read_bytes():
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)
            changed += 1
    if stage.is_dir():
        for path in sorted(stage.rglob('*'), reverse=True):
            if path.relative_to(stage) not in wanted:
                if path.is_dir():
                    shutil.rmtree(path, ignore_errors=True)
                else:
                    path.unlink(missing_ok=True)
    log(changed, 'files changed in the stage')


def stage_key(commit, bundles, tool):
    """What the stage is made from, as one hash: the commit, every bundle file, the mixins tool, dim_patch and the family rows."""
    import hashlib
    digest = hashlib.sha256(commit.encode())
    for bundle in bundles:
        root = pathlib.Path(bundle)
        for path in sorted(p for p in root.rglob('*') if p.is_file() and p.name != 'Cargo.lock'):
            digest.update(str(path.relative_to(root)).encode())
            digest.update(path.read_bytes())
    for path in (tool, ROOT / 'tools' / 'dim_patch.py'):
        digest.update(path.read_bytes() if path.is_file() else b'-')
    # The family rows dim_patch applies: a pulled row changes the stage too.
    for path in sorted((ROOT / 'family').glob('*_engine.py')):
        digest.update(path.name.encode())
        digest.update(path.read_bytes())
    return digest.hexdigest()


EXE = '.exe' if os.name == 'nt' else ''


def mixins_tool():
    """horizon-mixins: HORIZON_MIXINS, else the one `setup.py mixins` installed (tools-bin/), else the PATH's."""
    if os.environ.get('HORIZON_MIXINS'):
        return pathlib.Path(os.environ['HORIZON_MIXINS'])
    for candidate in (ROOT.parent / 'tools-bin' / 'bin' / f'horizon-mixins{EXE}', TARGET / 'mixins' / 'release' / f'horizon-mixins{EXE}'):
        if candidate.is_file():
            return candidate
    found = shutil.which('horizon-mixins')
    if not found:
        sys.exit('horizon-mixins not found: run `python setup.py mixins` first (or set HORIZON_MIXINS)')
    return pathlib.Path(found)


def rust_text(text):
    fence = '#' * 3
    return f'r{fence}"{text}"{fence}'


def main():
    args = sys.argv[1:]
    mode = next((a[2:] for a in args if a in ('--check', '--quick')), 'optimized')
    commit, out_exe, *bundles = [a for a in args if a not in ('--check', '--quick')]
    started = time.monotonic()
    tree = source(commit)
    # horizon-mixins checks each bundle's pin against this, as in the launcher's own builds.
    (tree / '.horizon-revision').write_text(commit, encoding='utf-8')
    mods = TARGET / 'bundles'
    shutil.rmtree(mods, ignore_errors=True)
    for bundle in bundles:
        shutil.copytree(bundle, mods / pathlib.Path(bundle).name, ignore=shutil.ignore_patterns('Cargo.lock'))
    tool = mixins_tool()
    stage = TARGET / 'stage'
    fresh = TARGET / 'stage-next'
    # The stage only changes when what makes it does: the source, the bundles, the mixins tool and dim_patch.
    # The same inputs as last time leave it as it is (horizon-mixins alone takes ~22 s).
    key = stage_key(commit, bundles, tool)
    keyfile = TARGET / 'stage.key'
    if (stage / 'Cargo.toml').is_file() and keyfile.is_file() and keyfile.read_text(encoding='utf-8') == key:
        log('stage up to date: nothing to apply')
    else:
        keyfile.unlink(missing_ok=True)
        shutil.rmtree(fresh, ignore_errors=True)
        log('applying', [pathlib.Path(b).name for b in bundles])
        step = time.monotonic()
        # horizon-mixins replaces an existing stage only on Linux.
        subprocess.run([tool, 'apply', '--workspace', tree, '--mods', mods, '--out', fresh], check=True, stdout=subprocess.DEVNULL)
        log(f'  mixins applied in {time.monotonic() - step:.1f} s')
        step = time.monotonic()
        subprocess.run([sys.executable, '-I', str(ROOT / 'tools' / 'dim_patch.py'), str(fresh)], check=True)
        log(f'  dim_patch in {time.monotonic() - step:.1f} s')
        step = time.monotonic()
        sync(fresh, stage)
        log(f'  stage synced in {time.monotonic() - step:.1f} s')
        keyfile.write_text(key, encoding='utf-8')
    cache = TARGET / {'check': 'cargo-check', 'quick': 'cargo-quick', 'optimized': 'cargo'}[mode]
    # Cargo hides its n/total count when its output goes to a file: the status line reads it.
    env = dict(os.environ, CARGO_TARGET_DIR=str(cache), CARGO_TERM_PROGRESS_WHEN='always', CARGO_TERM_PROGRESS_WIDTH='120')
    # What `/version` and `/plugins` say of this build (paper_fam_core::engine_lines): the engine's
    # commit (with + when it has changes not committed), the day, and the Horizon source it was built on.
    head = subprocess.run(['git', 'rev-parse', '--short', 'HEAD'], cwd=ROOT, capture_output=True, text=True).stdout.strip()
    dirty = subprocess.run(['git', 'status', '--porcelain', '--untracked-files=no', '--', 'bundle', 'family', 'sdk', 'tools'], cwd=ROOT,
                           capture_output=True, text=True).stdout.strip()
    env.update(HORIZON_ENGINE_COMMIT=(head or 'local') + ('+' if dirty else ''),
               HORIZON_ENGINE_BUILT=time.strftime('%Y-%m-%d'), HORIZON_SOURCE_COMMIT=commit[:9])
    command = ['cargo', TOOLCHAIN, 'check' if mode == 'check' else 'build', '--locked', '--profile', 'fast', '--no-default-features', '-p', 'horizon-client', '--bin', 'horizon-client']
    if mode == 'quick':
        for setting in ('profile.fast.opt-level=1', 'profile.fast.package."*".opt-level=3', 'profile.fast.package.horizon-items.opt-level=2', 'profile.fast.package.horizon-world.opt-level=2', 'profile.fast.package.horizon-core.opt-level=2', 'profile.fast.package.horizon-assets.opt-level=2', 'profile.fast.codegen-units=256', 'profile.fast.debug=false'):
            command += ['--config', setting]
    # rust-lld links the 75 MB program in seconds where MSVC's link.exe takes far longer; the linker is no part of
    # what cargo caches, so this costs no rebuild. ENGINE_LINKER=msvc for the old one.
    if os.environ.get('ENGINE_LINKER', 'lld') == 'lld':
        command += ['--config', 'target.x86_64-pc-windows-msvc.linker="rust-lld"']
    # The parallel front end: horizon-client is one 460,000-line crate whose parsing, type checks and borrow checks
    # ran on one thread. ENGINE_RUSTC_THREADS=1 turns it off (a change of these flags rebuilds everything once).
    threads = os.environ.get('ENGINE_RUSTC_THREADS', '8')
    if threads not in ('', '0', '1'):
        env['RUSTC_BOOTSTRAP'] = '1'
        command += ['--config', f'build.rustflags=["-Zthreads={threads}"]']
    log('compiling', mode)
    compiling = time.monotonic()
    # For the status line: a long crate writes nothing to the log for minutes.
    active = TARGET / 'build.active'
    active.write_text(json.dumps({'mode': mode, 'start': time.time()}), encoding='utf-8')
    try:
        subprocess.run(command, cwd=stage, env=env, check=True, **({'creationflags': BELOW_NORMAL} if os.name == 'nt' else {}))
    finally:
        active.unlink(missing_ok=True)
    log(f'{mode}: compiled in {time.monotonic() - compiling:.0f} s, {time.monotonic() - started:.0f} s in all')
    if mode == 'check':
        return
    built = cache / 'fast' / f'horizon-client{EXE}'
    pathlib.Path(out_exe).parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(built, out_exe)
    log('built', out_exe, built.stat().st_size // (1024 * 1024), 'MB')
    # Each build's times, to compare settings: target/build_times.jsonl.
    with (TARGET / 'build_times.jsonl').open('a', encoding='utf-8') as f:
        f.write(json.dumps({'mode': mode, 'compile_s': round(time.monotonic() - compiling, 1), 'total_s': round(time.monotonic() - started, 1),
                            'threads': threads, 'linker': os.environ.get('ENGINE_LINKER', 'lld'), 'at': time.strftime('%Y-%m-%d %H:%M')}) + '\n')


if __name__ == '__main__':
    main()
