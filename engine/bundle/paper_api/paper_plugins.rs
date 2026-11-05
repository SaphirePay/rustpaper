// The Paper engine's plugin loader: what Paper's loader does with the jars in `plugins/`, for plugin
// libraries. Each library describes itself from its `plugin.yml` (`horizon_bukkit::plugin_manifest`);
// this reads the descriptions, refuses what Paper refuses, and puts the rest in Paper's load order:
// a plugin after its `depend` and `softdepend` plugins, and before the ones it names in `loadbefore`.
// A plugin that can't load is left out with Paper's message, and the server goes on, as Paper's does.
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// What a plugin says of itself: `plugin.yml`'s fields the loader goes by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game) struct Description {
    pub(in crate::game) name: String,
    pub(in crate::game) version: String,
    pub(in crate::game) description: Option<String>,
    pub(in crate::game) authors: Vec<String>,
    pub(in crate::game) api_version: Option<String>,
    pub(in crate::game) depend: Vec<String>,
    pub(in crate::game) softdepend: Vec<String>,
    pub(in crate::game) loadbefore: Vec<String>,
    pub(in crate::game) provides: Vec<String>,
    /// The description as the library gave it: the runtime knows the library by it.
    pub(in crate::game) manifest: String,
    /// The library's file name in `plugins/`.
    pub(in crate::game) file: String,
}

impl Description {
    /// Reads a library's description, or says why Paper would refuse it.
    pub(in crate::game) fn parse(manifest: &str, file: &str) -> Result<Self, String> {
        let refuse = |why: &str| format!("Could not load 'plugins/{file}' in folder 'plugins': {why}");
        let json: Value = serde_json::from_str(manifest).map_err(|_| refuse("Invalid plugin.yml (a library built for another engine: rebuild it on the Paper SDK)"))?;
        let text = |key: &str| json[key].as_str().map(str::to_owned);
        let list = |key: &str| json[key].as_array().map(|items| items.iter().filter_map(|item| item.as_str().map(str::to_owned)).collect()).unwrap_or_default();
        let name = text("name").ok_or_else(|| refuse("Invalid plugin.yml: name is not defined"))?;
        // Bukkit's rule for a name: letters, digits, spaces, `_`, `.` and `-`.
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '.' | '-')) {
            return Err(refuse(&format!("Invalid plugin.yml: name '{name}' contains invalid characters.")));
        }
        let version = text("version").ok_or_else(|| refuse("Invalid plugin.yml: version is not defined"))?;
        Ok(Self {
            name: name.replace(' ', "_"),
            version,
            description: text("description"),
            authors: list("authors"),
            api_version: text("api-version"),
            depend: list("depend"),
            softdepend: list("softdepend"),
            loadbefore: list("loadbefore"),
            provides: list("provides"),
            manifest: manifest.to_owned(),
            file: file.to_owned(),
        })
    }

    /// `Name vVersion`, as Paper names a plugin in its log.
    pub(in crate::game) fn full_name(&self) -> String {
        format!("{} v{}", self.name, self.version)
    }
}

/// Puts plugins in Paper's load order and leaves out the ones Paper wouldn't load, each with its reason.
/// `found` is in the order of the files, which breaks every tie.
pub(in crate::game) fn order(found: Vec<Description>) -> (Vec<Description>, Vec<String>) {
    let mut errors = Vec::new();
    // Two plugins of one name: the first file wins.
    let mut plugins: Vec<Description> = Vec::new();
    for plugin in found {
        if let Some(first) = plugins.iter().find(|p| p.name == plugin.name) {
            errors.push(format!("Ambiguous plugin name `{}' for files `plugins/{}' and `plugins/{}' in `plugins'", plugin.name, first.file, plugin.file));
        } else {
            plugins.push(plugin);
        }
    }
    // A missing hard dependency leaves a plugin out, and with it each plugin that needs that one.
    loop {
        let names = known_names(&plugins);
        let Some(index) = plugins.iter().position(|p| p.depend.iter().any(|d| !names.contains(d))) else {
            break;
        };
        let plugin = plugins.remove(index);
        let missing: Vec<&str> = plugin.depend.iter().filter(|d| !names.contains(*d)).map(String::as_str).collect();
        errors.push(format!(
            "Could not load 'plugins/{}' in folder 'plugins': Unknown/missing dependency plugins: [{}]. Please download and install these plugins to run '{}'.",
            plugin.file,
            missing.join(", "),
            plugin.name
        ));
    }
    // Who comes before whom: (before, after), by index, hard or soft.
    let owner = owners(&plugins);
    let mut edges: Vec<(usize, usize, bool)> = Vec::new();
    for (index, plugin) in plugins.iter().enumerate() {
        for name in &plugin.depend {
            if let Some(&before) = owner.get(name.as_str()) {
                edges.push((before, index, true));
            }
        }
        for name in &plugin.softdepend {
            if let Some(&before) = owner.get(name.as_str()) {
                edges.push((before, index, false));
            }
        }
        for name in &plugin.loadbefore {
            if let Some(&after) = owner.get(name.as_str()) {
                edges.push((index, after, false));
            }
        }
    }
    edges.retain(|(before, after, _)| before != after);
    let (placed, stuck) = topological(plugins.len(), &edges);
    let mut order = placed;
    if !stuck.is_empty() {
        // Paper breaks a loop of soft rules; a loop of `depend` can't load.
        let in_loop: HashSet<usize> = stuck.iter().copied().collect();
        let hard: Vec<(usize, usize, bool)> = edges.iter().copied().filter(|&(b, a, hard)| hard || !(in_loop.contains(&b) && in_loop.contains(&a))).collect();
        let (again, still) = topological(plugins.len(), &hard);
        order = again;
        if !still.is_empty() {
            let names: Vec<&str> = still.iter().map(|&i| plugins[i].name.as_str()).collect();
            for &i in &still {
                errors.push(format!("Could not load 'plugins/{}' in folder 'plugins': Circular dependency detected: [{}]", plugins[i].file, names.join(", ")));
            }
        }
    }
    let mut slots: Vec<Option<Description>> = plugins.into_iter().map(Some).collect();
    (order.into_iter().filter_map(|i| slots[i].take()).collect(), errors)
}

/// What the engine itself does that a Java server takes a plugin for: ProtocolLib's and PacketEvents' packet
/// listeners are the engine's own (`protocol_engine.py`, `packetevents_engine.py`, jport's bridge), so a plugin that
/// depends on one loads, as it would beside that plugin on Paper.
pub(in crate::game) const BUILT_IN: [&str; 2] = ["ProtocolLib", "packetevents"];

/// Each name a plugin answers to, its own and the ones it `provides`, and what the engine provides.
fn known_names(plugins: &[Description]) -> HashSet<String> {
    plugins.iter().flat_map(|p| std::iter::once(p.name.clone()).chain(p.provides.iter().cloned())).chain(BUILT_IN.iter().map(|name| (*name).to_owned())).collect()
}

/// Which plugin answers to a name: its own name first, then a `provides`.
fn owners(plugins: &[Description]) -> HashMap<&str, usize> {
    let mut owner = HashMap::new();
    for (index, plugin) in plugins.iter().enumerate() {
        for name in &plugin.provides {
            owner.entry(name.as_str()).or_insert(index);
        }
    }
    for (index, plugin) in plugins.iter().enumerate() {
        owner.insert(plugin.name.as_str(), index);
    }
    owner
}

/// Kahn's order, taking the earliest file each time: the order placed, and what a loop held back.
fn topological(count: usize, edges: &[(usize, usize, bool)]) -> (Vec<usize>, Vec<usize>) {
    let mut waiting = vec![0usize; count];
    for &(_, after, _) in edges {
        waiting[after] += 1;
    }
    let mut placed = Vec::with_capacity(count);
    let mut done = vec![false; count];
    while let Some(next) = (0..count).find(|&i| !done[i] && waiting[i] == 0) {
        done[next] = true;
        placed.push(next);
        for &(before, after, _) in edges {
            if before == next {
                waiting[after] -= 1;
            }
        }
    }
    let stuck = (0..count).filter(|&i| !done[i]).collect();
    (placed, stuck)
}

/// The plugins this server runs, in enable order, once `launch` has read them.
pub(in crate::game) struct Plugins {
    pub(in crate::game) game_dir: PathBuf,
    /// A dedicated server: plugins enable once its worlds are loaded, as on Paper.
    pub(in crate::game) dedicated: bool,
    pub(in crate::game) loaded: Vec<Description>,
}

impl Plugins {
    pub(in crate::game) fn folder(&self) -> PathBuf {
        self.game_dir.join("plugins")
    }

    pub(in crate::game) fn find(&self, name: &str) -> Option<&Description> {
        self.loaded.iter().find(|p| p.name == name)
    }
}

static PLUGINS: OnceLock<Plugins> = OnceLock::new();

pub(in crate::game) fn get() -> Option<&'static Plugins> {
    PLUGINS.get()
}

/// Reads the plugin libraries in `folder`: each one's description, in file order, and what couldn't be read.
pub(in crate::game) fn read_folder(folder: &Path) -> (Vec<Description>, Vec<String>) {
    let (libraries, mut errors) = horizon_mod_runtime::open_folder(folder);
    let mut found = Vec::new();
    for (manifest, file) in libraries {
        match Description::parse(&manifest, &file) {
            Ok(description) => found.push(description),
            Err(error) => errors.push(error),
        }
    }
    (found, errors)
}

/// The platform folder a converted mod's library sits in inside its `.hmod` (Horizon Mod Converter).
const MOD_CODE_PLATFORM: &str = if cfg!(windows) { "windows-x64" } else { "linux-x64" };

/// Converted mods (Horizon Mod Converter): a content mod's `.hmod` in `horizon-mods/`,
/// switched on in `horizon-mods.txt` (`mod.<id>:true`, Options > Mods) as its content is, carrying
/// `native/<platform>/<library>`, has that library unpacked into `.horizon-cache/mod-code/`, the folder it then
/// loads from as a plugin's does. None when no mod that is on carries code (nothing is unpacked or loaded).
fn unpack_mod_code(game_dir: &std::path::Path) -> Option<PathBuf> {
    use std::io::Read;
    let mods = game_dir.join("horizon-mods");
    let on: Vec<String> = std::fs::read_to_string(game_dir.join("horizon-mods.txt"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.trim().strip_prefix("mod.")?.strip_suffix(":true").map(str::to_owned))
        .collect();
    let cache = game_dir.join(".horizon-cache").join("mod-code");
    let _ = std::fs::remove_dir_all(&cache);
    let prefix = format!("native/{MOD_CODE_PLATFORM}/");
    let mut any = false;
    for path in std::fs::read_dir(&mods).ok()?.flatten().map(|e| e.path()) {
        if path.extension().is_none_or(|x| x != "hmod") {
            continue;
        }
        let Ok(file) = std::fs::File::open(&path) else { continue };
        let Ok(mut zip) = zip::ZipArchive::new(file) else { continue };
        // The mod's id from its manifest: its code runs only while the mod is on.
        let mut manifest = String::new();
        if zip.by_name("horizon-mod.toml").is_ok_and(|mut e| e.read_to_string(&mut manifest).is_ok()) {
            let id = manifest
                .lines()
                .find_map(|l| l.trim().strip_prefix("id")?.trim_start().strip_prefix('=').map(|v| v.trim().trim_matches('"').to_owned()));
            if id.is_none_or(|id| !on.contains(&id)) {
                continue;
            }
        } else {
            continue;
        }
        // zip 9 names entries as `Result`s: a name it can't read is skipped.
        let names: Vec<String> = zip
            .file_names()
            .filter_map(|n| n.ok().map(|n| n.into_owned()))
            .filter(|n| n.starts_with(&prefix))
            .collect();
        for name in names {
            let Some(base) = name.rsplit('/').next().filter(|b| !b.is_empty() && !b.contains("..")) else { continue };
            let mut bytes = Vec::new();
            if zip.by_name(&name).is_ok_and(|mut e| e.read_to_end(&mut bytes).is_ok()) {
                let _ = std::fs::create_dir_all(&cache);
                if std::fs::write(cache.join(base), bytes).is_ok() {
                    any = true;
                }
            }
        }
    }
    any.then_some(cache)
}

/// The ids of the content mods switched on (`horizon-mods.txt`, `mod.<id>:true`), and each `.hmod` of
/// `horizon-mods/` with its id.
fn mods_on(game_dir: &std::path::Path) -> (Vec<String>, Vec<(String, PathBuf)>) {
    use std::io::Read;
    let on: Vec<String> = std::fs::read_to_string(game_dir.join("horizon-mods.txt"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.trim().strip_prefix("mod.")?.strip_suffix(":true").map(str::to_owned))
        .collect();
    let mut files = Vec::new();
    for path in std::fs::read_dir(game_dir.join("horizon-mods")).into_iter().flatten().flatten().map(|e| e.path()) {
        if path.extension().is_none_or(|x| x != "hmod") {
            continue;
        }
        let Ok(file) = std::fs::File::open(&path) else { continue };
        let Ok(mut zip) = zip::ZipArchive::new(file) else { continue };
        let mut manifest = String::new();
        if zip.by_name("horizon-mod.toml").is_ok_and(|mut e| e.read_to_string(&mut manifest).is_ok())
            && let Some(id) = manifest
                .lines()
                .find_map(|l| l.trim().strip_prefix("id")?.trim_start().strip_prefix('=').map(|v| v.trim().trim_matches('"').to_owned()))
        {
            files.push((id, path));
        }
    }
    (on, files)
}

/// Converted data-pack mods (Horizon Mod Converter): a mod that is on and carries `datapack/` (files that change
/// vanilla's own data, which a content mod may not hold) has it unpacked into `.horizon-cache/mod-packs/hmod-<id>/`
/// and added to the packs plugins add (`PLUGIN_PACKS`, read wherever a world's packs are found, its preload too), as a
/// required pack: on in every world while the mod is on.
fn register_mod_datapacks(game_dir: &std::path::Path) {
    use std::io::Read;
    let (on, files) = mods_on(game_dir);
    let root = game_dir.join(".horizon-cache").join("mod-packs");
    let _ = std::fs::remove_dir_all(&root);
    for (id, path) in files.iter().filter(|(id, _)| on.contains(id)) {
        let Ok(file) = std::fs::File::open(path) else { continue };
        let Ok(mut zip) = zip::ZipArchive::new(file) else { continue };
        let names: Vec<String> = zip
            .file_names()
            .filter_map(|n| n.ok().map(|n| n.into_owned()))
            .filter(|n| n.starts_with("datapack/") && !n.ends_with('/') && !n.contains(".."))
            .collect();
        if names.is_empty() {
            continue;
        }
        let dest = root.join(format!("hmod-{id}"));
        for name in names {
            let mut bytes = Vec::new();
            if zip.by_name(&name).is_ok_and(|mut e| e.read_to_end(&mut bytes).is_ok()) {
                let target = dest.join(&name["datapack/".len()..]);
                if let Some(parent) = target.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(target, bytes);
            }
        }
        let pack = horizon_client::datapack::PluginPack {
            id: format!("hmod-{id}"),
            path: dest,
            title: serde_json::json!(id),
            required: true,
            top: true,
            fixed: false,
        };
        let mut packs = horizon_client::datapack::PLUGIN_PACKS.lock().unwrap_or_else(|e| e.into_inner());
        packs.retain(|known| known.id != pack.id);
        packs.push(pack);
        log::info!("[Horizon Mod Converter] {id}: its data pack is on");
    }
}

/// The server's start: read `plugins/`, order what can load, and start each plugin (`onLoad`). On a dedicated
/// server their `onEnable` waits for the worlds (`paper_bridge::enable_waiting`).
pub(in crate::game) fn launch() {
    let args: Vec<String> = std::env::args().collect();
    let dedicated = args.iter().any(|a| a == "--dedicated");
    let game_dir = args
        .iter()
        .position(|a| a == "--game-dir" || a == "--gameDir")
        .and_then(|i| args.get(i + 1))
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let (mut found, mut read_errors) = (Vec::new(), Vec::new());
    if dedicated {
        let folder = game_dir.join("plugins");
        let _ = std::fs::create_dir_all(&folder);
        (found, read_errors) = read_folder(&folder);
    }
    // Converted mods' data packs (vanilla data they change), on in every world while the mod is on.
    register_mod_datapacks(&game_dir);
    // Converted mods' code, in single player too: the libraries inside `.hmod` files in `horizon-mods/`.
    if let Some(code) = unpack_mod_code(&game_dir) {
        let (more, errors) = read_folder(&code);
        found.extend(more);
        read_errors.extend(errors);
    }
    let (ordered, order_errors) = order(found);
    for error in read_errors.iter().chain(&order_errors) {
        log::error!("[PluginLoader] {error}");
    }
    let loaded = ordered;
    let plugins = PLUGINS.get_or_init(|| Plugins { game_dir, dedicated, loaded });
    if !plugins.loaded.is_empty() {
        log::info!("[PluginLoader] Loading {} plugins: {}", plugins.loaded.len(), plugins.loaded.iter().map(Description::full_name).collect::<Vec<_>>().join(", "));
    }
    // Paper's line for each plugin as its `onLoad` runs.
    for plugin in &plugins.loaded {
        log::info!("[{}] Loading server plugin {}", plugin.name, plugin.full_name());
    }
    let start: Vec<(String, String)> = plugins.loaded.iter().map(|p| (p.name.clone(), p.manifest.clone())).collect();
    crate::game::paper_bridge::start_mods(&start);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plugin(name: &str, depend: &[&str], softdepend: &[&str], loadbefore: &[&str]) -> Description {
        let words = |list: &[&str]| list.iter().map(|w| (*w).to_owned()).collect::<Vec<_>>();
        Description {
            name: name.to_owned(),
            version: "1".to_owned(),
            description: None,
            authors: Vec::new(),
            api_version: None,
            depend: words(depend),
            softdepend: words(softdepend),
            loadbefore: words(loadbefore),
            provides: Vec::new(),
            manifest: name.to_owned(),
            file: format!("{name}.dll"),
        }
    }

    fn names(list: &[Description]) -> Vec<&str> {
        list.iter().map(|p| p.name.as_str()).collect()
    }

    #[test]
    fn a_plugin_loads_after_what_it_depends_on() {
        let (order, errors) = order(vec![plugin("Shop", &["Vault"], &[], &[]), plugin("Vault", &[], &[], &[])]);
        assert_eq!(names(&order), ["Vault", "Shop"]);
        assert!(errors.is_empty());
    }

    #[test]
    fn a_missing_depend_leaves_the_plugin_and_its_dependents_out() {
        let (order, errors) = order(vec![plugin("A", &["Gone"], &[], &[]), plugin("B", &["A"], &[], &[]), plugin("C", &[], &[], &[])]);
        assert_eq!(names(&order), ["C"]);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("Unknown/missing dependency plugins: [Gone]. Please download and install these plugins to run 'A'."));
        assert!(errors[1].contains("[A]") && errors[1].contains("'B'"));
    }

    #[test]
    fn a_missing_softdepend_is_no_error() {
        let (order, errors) = order(vec![plugin("A", &[], &["Gone"], &[])]);
        assert_eq!(names(&order), ["A"]);
        assert!(errors.is_empty());
    }

    #[test]
    fn softdepend_and_loadbefore_order_the_start() {
        let (order, _) = order(vec![plugin("A", &[], &["B"], &[]), plugin("B", &[], &[], &[]), plugin("C", &[], &[], &["B"])]);
        assert_eq!(names(&order), ["C", "B", "A"]);
    }

    #[test]
    fn provides_satisfies_a_depend() {
        let mut vault = plugin("VaultUnlocked", &[], &[], &[]);
        vault.provides = vec!["Vault".to_owned()];
        let (order, errors) = order(vec![plugin("Shop", &["Vault"], &[], &[]), vault]);
        assert_eq!(names(&order), ["VaultUnlocked", "Shop"]);
        assert!(errors.is_empty());
    }

    #[test]
    fn what_the_engine_provides_satisfies_a_depend() {
        let (order, errors) = order(vec![plugin("Packets", &["ProtocolLib"], &[], &[]), plugin("Other", &["packetevents"], &[], &[])]);
        assert_eq!(names(&order), ["Packets", "Other"]);
        assert!(errors.is_empty());
    }

    #[test]
    fn the_first_file_of_a_name_wins() {
        let mut second = plugin("A", &[], &[], &[]);
        second.file = "A-2.dll".to_owned();
        let (order, errors) = order(vec![plugin("A", &[], &[], &[]), second]);
        assert_eq!(order.len(), 1);
        assert!(errors[0].starts_with("Ambiguous plugin name `A'"));
    }

    #[test]
    fn a_soft_loop_is_broken_and_a_hard_loop_refused() {
        let (placed, errors) = order(vec![plugin("A", &[], &["B"], &[]), plugin("B", &[], &["A"], &[])]);
        assert_eq!(names(&placed), ["A", "B"]);
        assert!(errors.is_empty());
        let (placed, errors) = order(vec![plugin("A", &["B"], &[], &[]), plugin("B", &["A"], &[], &[]), plugin("C", &[], &[], &[])]);
        assert_eq!(names(&placed), ["C"]);
        assert!(errors.iter().all(|e| e.contains("Circular dependency detected")));
    }

    #[test]
    fn a_description_is_read_as_paper_reads_plugin_yml() {
        let manifest = r#"{"name":"My Plugin","version":"2.1","depend":["Vault"],"softdepend":[],"loadbefore":[],"provides":[],"authors":["x"]}"#;
        let description = Description::parse(manifest, "my.dll").unwrap();
        assert_eq!(description.name, "My_Plugin");
        assert_eq!(description.full_name(), "My_Plugin v2.1");
        assert!(Description::parse(r#"{"version":"1"}"#, "x.dll").unwrap_err().contains("name is not defined"));
        assert!(Description::parse(r#"{"id":"old","version":"1"}"#, "old.dll").is_err());
        assert!(Description::parse("not json", "old.dll").unwrap_err().contains("rebuild it on the Paper SDK"));
        assert!(Description::parse(r#"{"name":"Bad/Name","version":"1"}"#, "b.dll").unwrap_err().contains("invalid characters"));
    }
}
