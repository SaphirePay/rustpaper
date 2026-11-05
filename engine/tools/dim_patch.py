#!/usr/bin/env python3
"""Worlds beyond the three: what a mixin can't do, as `Dim` is a closed enum.

`build_engine.py` runs this on the applied source, before it reaches the stage.

A created world is `Dim::Custom(n)`, with its own level, save folder, seed and generator. Bukkit's worlds have an
environment, so `dim.kind()` says which of the three it plays as. Every comparison with a named dimension and every
match on one goes by kind, which is how a created overworld gets weather, sky and beds. Two dimensions compared with
each other stay by identity, so a created overworld is never taken for the real one.
"""
import pathlib
import re
import sys

DIM_ENUM = '''#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Dim {
    Overworld,
    Nether,
    End,
}
impl Dim {
    fn id(self) -> &'static str {
        match self {
            Self::Overworld => "minecraft:overworld",
            Self::Nether => "minecraft:the_nether",
            Self::End => "minecraft:the_end",
        }
    }
    fn from_id(id: &str) -> Self {
        match id {
            "minecraft:the_nether" => Self::Nether,
            "minecraft:the_end" => Self::End,
            _ => Self::Overworld,
        }
    }
'''

DIM_NEW = '''#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Dim {
    Overworld,
    Nether,
    End,
    /// A world a plugin created: its number in `CREATED_WORLDS`.
    Custom(u16),
}
/// A created world, for as long as the server runs: a plugin asks for its worlds again at each start, as on Bukkit.
#[derive(Clone, Copy)]
struct CreatedWorld {
    id: &'static str,
    name: &'static str,
    kind: Dim,
    seed: Option<i64>,
    flat: bool,
    structures: bool,
    /// `WorldCreator.type`: the world preset of its terrain.
    preset: &'static str,
    /// `WorldCreator.generatorSettings` of a FLAT world.
    layers: &'static str,
    /// A plugin's `BiomeProvider` chooses its biomes.
    provider: bool,
    /// A plugin's `ChunkGenerator` runs between the game's steps, which leave out `left_out`.
    stepped: bool,
    left_out: u8,
    decorations: bool,
    /// `ChunkGenerator.shouldGenerateMobs`.
    mobs: bool,
    /// `WorldCreator.hardcore`.
    hardcore: bool,
    /// A plugin writes over its chunks: `ChunkGenerator`, `BiomeProvider`.
    generated: bool,
    /// The game makes the chunks a plugin writes over.
    vanilla: bool,
    open: bool,
}
static CREATED_WORLDS: std::sync::RwLock<Vec<CreatedWorld>> = std::sync::RwLock::new(Vec::new());
impl Dim {
    fn created(self) -> Option<CreatedWorld> {
        match self {
            Self::Custom(index) => CREATED_WORLDS.read().unwrap_or_else(|e| e.into_inner()).get(usize::from(index)).copied(),
            _ => None,
        }
    }
    /// Which of the three it plays as: Bukkit's `World.Environment`.
    fn kind(self) -> Dim {
        self.created().map_or(self, |world| world.kind)
    }
    /// Every level a server can have, open or not.
    fn all() -> Vec<Dim> {
        let worlds = CREATED_WORLDS.read().unwrap_or_else(|e| e.into_inner());
        let mut dims = vec![Self::Overworld, Self::Nether, Self::End];
        dims.extend(worlds.iter().enumerate().filter(|(_, world)| world.open).map(|(index, _)| Self::Custom(index as u16)));
        dims
    }
    /// A name `Dim::from_id` knows.
    fn known(id: &str) -> bool {
        matches!(id, "minecraft:overworld" | "minecraft:the_nether" | "minecraft:the_end") || CREATED_WORLDS.read().unwrap_or_else(|e| e.into_inner()).iter().any(|world| world.open && world.id == id)
    }
    fn seed(self, world_seed: i64) -> i64 {
        self.created().and_then(|world| world.seed).unwrap_or(world_seed)
    }
    fn id(self) -> &'static str {
        match self {
            Self::Overworld => "minecraft:overworld",
            Self::Nether => "minecraft:the_nether",
            Self::End => "minecraft:the_end",
            Self::Custom(_) => self.created().map_or("minecraft:overworld", |world| world.id),
        }
    }
    fn from_id(id: &str) -> Self {
        match id {
            "minecraft:the_nether" => Self::Nether,
            "minecraft:the_end" => Self::End,
            "minecraft:overworld" => Self::Overworld,
            _ => CREATED_WORLDS.read().unwrap_or_else(|e| e.into_inner()).iter().position(|world| world.id == id).map_or(Self::Overworld, |index| Self::Custom(index as u16)),
        }
    }
'''

SHAPE_OLD = '''        use horizon_client::save::LevelShape;
        match self {
            Self::Overworld => LevelShape::OVERWORLD,
            Self::Nether => LevelShape::NETHER,
            Self::End => LevelShape::END,
        }
'''
SHAPE_NEW = '''        use horizon_client::save::LevelShape;
        match self.kind() {
            Self::Nether => LevelShape::NETHER,
            Self::End => LevelShape::END,
            _ => LevelShape::OVERWORLD,
        }
'''

NAMED = r'(?:super::)?Dim::(?:Overworld|Nether|End)\b'
# `a.b == Dim::Nether`, with the left side a plain path.
COMPARE = re.compile(rf'(?<![\w.)\]])((?:\*\s*)*[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*)(\s*(?:==|!=)\s*)({NAMED})')
# The same with a call on the left: what `COMPARE` can't take.
CALLED = re.compile(rf'(?<!\.kind\()\)(\s*(?:==|!=)\s*)({NAMED})')
# `match x {` over arms that name a dimension.
MATCH = re.compile(rf'\bmatch ([^{{}}\n()]+?) \{{\n(\s*)({NAMED}\s*(?:=>|\|))')
MATCHES = re.compile(rf'\bmatches!\((\*?[A-Za-z_][\w.]*),\s*({NAMED})')
# Portal scale, by the two kinds.
PAIR = re.compile(r'\bmatch \((\*?[\w.]+), (\*?[\w.]+)\) \{\n(\s*)\(((?:super::)?Dim::)')

EDITS = {
    'game/levels.rs': [
        ('[Dim::Overworld, Dim::Nether, Dim::End]\n            .into_iter()', 'Dim::all()\n            .into_iter()'),
        ('            self.level.seed,\n            &self.level.enabled_features,', '            dim.seed(self.level.seed),\n            &self.level.enabled_features,'),
        ('World::new(self.res.facts().air, self.level.seed)', 'World::new(self.res.facts().air, dim.seed(self.level.seed))'),
    ],
    'game/levels.rs#tail': [],
    'game/lan_actions.rs': [
        ('''        let ok = if self.perform_native_command(c.conn, c.permission, &c.command) {
            true
        } else if c.quiet {
            self.run_command_quietly(&c.command)
        } else {
            self.run_command(&c.command)
        };
''', '''        // `minecraft:list` is vanilla's whatever a plugin took, and `plugin:word` is that plugin's.
        let (vanilla_only, line) = match c.command.strip_prefix("minecraft:") {
            Some(rest) => (true, rest),
            None => (false, super::paper_bridge::unprefixed(&c.command)),
        };
        let ok = if !vanilla_only && self.perform_native_command(c.conn, c.permission, line) {
            true
        } else if c.quiet {
            self.run_command_quietly(line)
        } else {
            self.run_command(line)
        };
'''),
    ],
    'game/native_server.rs': [
        ('''        if vanilla(&nodes[0].0) {
            problems.push(format!("`{}`: vanilla has /{}", command.usage, nodes[0].0));
            continue;
        }
''', '''        // The word is the mod's now, as in Bukkit's command map, and `minecraft:` reaches vanilla's. The
        // client's tree keeps vanilla's shape: it only suggests.
        if vanilla(&nodes[0].0) {
            levels.insert(nodes[0].0.clone(), command.level);
            continue;
        }
'''),
    ],
    'game/world_loading.rs': [
        ('seed, preset, dim.id(), structures)', 'seed, preset, dim.kind().id(), structures)'),
        ('.then(|| horizon_client::save::world_gen_settings(world_root))', '.then(|| super::paper_worlds::settings_of(dim, world_root))'),
        ('.then(|| horizon_client::save::world_gen_settings(world_root))', '.then(|| super::paper_worlds::settings_of(dim, world_root))'),
        ('.then(|| horizon_client::save::world_gen_settings(world_root))', '.then(|| super::paper_worlds::settings_of(dim, world_root))'),
        ('horizon_client::save::WorldGen::read(world_root).is_none_or(|w| w.structures);\n        let preset', 'horizon_client::save::WorldGen::read(world_root).is_none_or(|w| w.structures) && dim.created().is_none_or(|w| w.structures);\n        let preset'),
        ('let preset = horizon_client::save::WorldGen::preset(world_root);\n        let overrides', 'let preset = dim.created().map_or_else(|| horizon_client::save::WorldGen::preset(world_root), |world| world.preset);\n        let overrides'),
        ('if dim.kind() == Dim::Overworld {\n            inner.set_old_world', 'if dim == Dim::Overworld {\n            inner.set_old_world'),
        ('    let mut streaming = if let Some(inner) = debug {', '''    let generated = dim.created().filter(|world| world.generated);
    let mut streaming = if generated.is_some_and(|world| !world.vanilla) {
        let biome = u32::from(res.biomes().id("minecraft:plains"));
        let blank = Arc::new(super::paper_worlds::Blank { shape, biome });
        // No terrain, but a plugin's `BiomeProvider` can still ask for the game's climate and biomes.
        let overrides = world_worldgen_overrides(world_root, &options.resources, &biomes, enabled_features);
        let vanilla = VanillaTerrain::for_world_with(&options.resources.join("worldgen"), &overrides, pack.assets.registry, &biomes, seed, dim.created().map_or("minecraft:normal", |world| world.preset), dim.kind().id(), false).ok().map(Arc::new);
        Streaming::new(
            Arc::new(Persistent {
                inner: Arc::new(super::paper_worlds::Overlay::new(blank, vanilla, None, dim, seed, shape, pack.assets.clone(), biomes.clone())),
                save: save.clone(),
                registry: pack.assets.clone(),
                biomes: biomes.clone(),
            }),
            pack.context.clone(),
            res.threads(),
        )
    } else if let Some(inner) = debug {'''),
        ('''        locator = Some(inner.clone());
        Streaming::new(
            Arc::new(Persistent {
                inner,
                save: save.clone(),
                registry: pack.assets.clone(),
                biomes: biomes.clone(),
            }),
            pack.context.clone(),
            res.threads(),
        )
    };
''', '''        locator = Some(inner.clone());
        if generated.is_some() {
            Streaming::new(
                Arc::new(Persistent {
                    inner: Arc::new(super::paper_worlds::Overlay::new(inner.clone(), Some(inner), generated, dim, seed, shape, pack.assets.clone(), biomes.clone())),
                    save: save.clone(),
                    registry: pack.assets.clone(),
                    biomes: biomes.clone(),
                }),
                pack.context.clone(),
                res.threads(),
            )
        } else {
            Streaming::new(
                Arc::new(Persistent {
                    inner,
                    save: save.clone(),
                    registry: pack.assets.clone(),
                    biomes: biomes.clone(),
                }),
                pack.context.clone(),
                res.threads(),
            )
        }
    };
'''),
    ],
    'game/commands/execute.rs': [
        ('''if !matches!(
                    dim.as_str(),
                    "minecraft:overworld" | "minecraft:the_nether" | "minecraft:the_end"
                ) {''', 'if !Dim::known(dim) {'),
        ('let coordinate_scale = |dim| if', 'let coordinate_scale = |dim: Dim| if'),
    ],
}


CLOSE_LEVEL = '''
impl Game<'_> {
    /// `Server.unloadWorld`: its chunks leave, saved if asked, and the level goes.
    pub(super) fn close_created_level(&mut self, dim: Dim, save: bool) -> Result<()> {
        let back = if self.dim == dim { Dim::Overworld } else { self.dim };
        if self.dim != dim && !self.levels.parked.contains_key(&dim) {
            return Ok(());
        }
        self.enter_level(None, dim)?;
        let chunks: Vec<_> = self.world.chunks.values().cloned().collect();
        self.native_chunks_changed(&chunks);
        if save {
            self.save_level(true)?;
        }
        self.enter_level(None, back)?;
        self.levels.parked.remove(&dim);
        Ok(())
    }
}
'''


def transform(text):
    text = COMPARE.sub(lambda m: f'{m.group(1).lstrip("* ")}.kind(){m.group(2)}{m.group(3)}', text)
    text = CALLED.sub(lambda m: f').kind(){m.group(1)}{m.group(2)}', text)
    text = MATCH.sub(lambda m: f'match {m.group(1)}.kind() {{\n{m.group(2)}{m.group(3)}', text)
    text = MATCHES.sub(lambda m: f'matches!({m.group(1)}.kind(), {m.group(2)}', text)
    text = PAIR.sub(lambda m: f'match ({m.group(1)}.kind(), {m.group(2)}.kind()) {{\n{m.group(3)}({m.group(4)}', text)
    # A level's shape is its kind's: `level_shapes` knows the registry's dimensions.
    text = re.sub(r'shapes\.get\((\w+)\.id\(\)\)', r'shapes.get(\1.kind().id())', text)
    text = re.sub(r'("minecraft:the_end" => ((?:super::)?Dim)::End,\n)(\s*)(_ =>)', r'\1\3other if \2::known(other) => \2::from_id(other),\n\3\4', text)
    return close_matches(text)


def close_matches(text):
    """A match on `.kind()` names the three, and `Custom` never comes out of `kind`: the compiler can't know."""
    out, at = [], 0
    for found in re.finditer(r'\bmatch [^{}\n]*\.kind\(\) \{\n', text):
        if found.start() < at:
            continue
        depth, end = 1, found.end()
        while depth:
            depth += {'{': 1, '}': -1}.get(text[end], 0)
            end += 1
        body = text[found.end():end - 1]
        arms = re.search(r'^\s*((?:super::)?Dim)::(?:Overworld|Nether|End)\b', body, re.M)
        if not arms or re.search(r'^\s*(?:_|\w+) (?:if [^\n]*)?=>', body, re.M):
            continue
        out += [text[at:end - 1].rstrip(' '), f'{arms.group(1)}::Custom(_) => unreachable!("a kind is one of the three"),\n']
        at = end - 1
    return ''.join(out) + text[at:]


def loose_span(text, old):
    """Where `old` is in `text` once if whitespace and trailing commas don't count, or None. horizon-mixins
    reformats only the files it injects into, so a row written against that form must still find its text in
    a file nothing injects into any more (and the other way round)."""
    def squeeze(s):
        # Whitespace and `//` comments go (prettyplease drops plain comments); string literals are kept whole.
        chars, where = [], []
        i, quoted = 0, False
        while i < len(s):
            c = s[i]
            if quoted:
                if c == '\\':
                    chars += [c, s[i + 1:i + 2]]
                    where += [i, i + 1]
                    i += 2
                    continue
                quoted = c != '"'
            elif c == '"' and not (i > 0 and s[i - 1] == "'" and s[i + 1:i + 2] == "'"):
                quoted = True
            elif s.startswith('//', i) and not s.startswith('///', i) and not s.startswith('//!', i):
                end = s.find('\n', i)
                i = len(s) if end < 0 else end
                continue
            if quoted or not c.isspace():
                chars.append(c)
                where.append(i)
            i += 1
        # A comma just before a closing bracket: one form writes it, the other doesn't.
        keep = [k for k in range(len(chars)) if not (chars[k] == ',' and k + 1 < len(chars) and chars[k + 1] in ')]}')]
        return ''.join(chars[k] for k in keep), [where[k] for k in keep]
    tight_text, where = squeeze(text)
    tight_old, _ = squeeze(old)
    if not tight_old or tight_text.count(tight_old) != 1:
        return None
    start = tight_text.index(tight_old)
    end = start + len(tight_old) - 1
    begin = where[start]
    # From the start of the first character's line when the row's text starts a line, as rows do, but only when
    # nothing stands before it there: one form's line can be the middle of the other's.
    line_start = text.rfind('\n', 0, begin) + 1
    if old[:1].isspace() and not text[line_start:begin].strip():
        begin = line_start
    finish = where[end] + 1
    # Take the comma a dropped one stood for, and the line's end when the row's text ends one.
    rest = text[finish:]
    if old.rstrip(' ').endswith(',') and rest.lstrip(' \n').startswith(','):
        finish += len(rest) - len(rest.lstrip(' \n')) + 1
    if old.endswith('\n') and text[finish:finish + 1] == '\n':
        finish += 1
    return begin, finish


def family_edits(crates):
    """What a mixin can't do, by family: `family/<f>_engine.py` (this repository's) holds `EDITS`, rows of (file under
    `crates`, text, replacement). Each text must be found once in the applied source: a private field made
    `pub(in crate::game)`, a constant turned into a static. Hooks are planned before this, so they can't target an edit."""
    texts, changed = {}, set()
    for table in sorted((pathlib.Path(__file__).resolve().parent.parent / 'family').glob('*_engine.py')):
        # perf_build.py --bare: the port without its speed table, for a before and after.
        if table.stem == 'perf_engine' and __import__('os').environ.get('HORIZON_NO_PERF'):
            continue
        # A measuring build only (perf_build.py with HORIZON_RAM_PROBE=1): it costs every allocation a header.
        if table.stem == 'ramprobe_engine' and not __import__('os').environ.get('HORIZON_RAM_PROBE'):
            continue
        scope = {}
        exec(compile(table.read_text(encoding='utf-8'), str(table), 'exec'), scope)
        for name, old, new in scope.get('EDITS', []):
            # Each file read once and written once: the virus scanner rescans a changed file on every
            # read, and a read and a write a row took ~30 s a build. `game/../game.rs` is `game.rs`.
            name = __import__('os').path.normpath(name)
            if name not in texts:
                texts[name] = (crates / name).read_text(encoding='utf-8')
            text = texts[name]
            if text.count(old) == 1:
                texts[name] = text.replace(old, new, 1)
            else:
                span = loose_span(text, old)
                assert span, f'{table.name}: {old!r} is in {name} {text.count(old)} times'
                texts[name] = text[:span[0]] + new + text[span[1]:]
            changed.add(name)
    for name in sorted(changed):
        (crates / name).write_text(texts[name], encoding='utf-8', newline='\n')


def main():
    root = pathlib.Path(sys.argv[1]) / 'crates' / 'horizon-client' / 'src'
    game = root / 'game.rs'
    text = game.read_text(encoding='utf-8')
    assert DIM_ENUM in text and SHAPE_OLD in text, 'Dim has changed: read dim_patch.py again'
    head, tail = text.split(DIM_ENUM, 1)
    tail = tail.replace(SHAPE_OLD, SHAPE_NEW, 1)
    game.write_text(transform(head) + DIM_NEW + transform(tail), encoding='utf-8', newline='\n')
    # The enum's own code above was written by hand: only what's around it is transformed.
    for path in sorted((root / 'game').rglob('*.rs')):
        text = path.read_text(encoding='utf-8')
        new = transform(text)
        for name, edits in EDITS.items():
            if path.as_posix().endswith(name):
                for old, replacement in edits:
                    assert old in new, old
                    new = new.replace(old, replacement, 1)
        if new != text:
            path.write_text(new, encoding='utf-8', newline='\n')
    levels = root / 'game' / 'levels.rs'
    levels.write_text(levels.read_text(encoding='utf-8') + CLOSE_LEVEL, encoding='utf-8', newline='\n')
    family_edits(pathlib.Path(sys.argv[1]) / 'crates')
    hub = root / 'lan' / 'hub.rs'
    text = hub.read_text(encoding='utf-8')
    fixed, count = re.subn(r'\.id\("minecraft:dimension_type", ([^)\n]+)\)', r'.id("minecraft:dimension_type", crate::paper_hub::dimension_type_of(\1))', text)
    assert count, 'the dimension type lookup moved'
    hub.write_text(fixed, encoding='utf-8', newline='\n')
    print('dim patch applied')


if __name__ == '__main__':
    main()
