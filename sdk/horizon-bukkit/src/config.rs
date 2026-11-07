//! `org.bukkit.configuration`: YAML files read and written by dotted path.
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

fn to_json(yaml: serde_yaml::Value) -> Value {
    match yaml {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(b) => Value::Bool(b),
        serde_yaml::Value::Number(n) => serde_json::to_value(n).unwrap_or(Value::Null),
        serde_yaml::Value::String(s) => Value::String(s),
        serde_yaml::Value::Sequence(list) => Value::Array(list.into_iter().map(to_json).collect()),
        serde_yaml::Value::Mapping(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    let key = scalar(&to_json(key)).unwrap_or_default();
                    (key, to_json(value))
                })
                .collect(),
        ),
        serde_yaml::Value::Tagged(tagged) => to_json(tagged.value),
    }
}

/// YAML's scalars as text: `version: 6.3` is a number in the file and a string to a plugin.
pub fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// A YAML document as JSON; nothing for what doesn't parse. A key written twice keeps its last value, as
/// Bukkit's SnakeYAML reads it (serde_yaml refuses the whole file: a plugin's every message was lost).
pub fn parse(text: &str) -> Value {
    match serde_yaml::from_str::<serde_yaml::Value>(text) {
        Ok(value) => to_json(value),
        Err(_) => serde_yaml::Deserializer::from_str(text).next().and_then(|document| serde::Deserialize::deserialize(document).ok()).map_or(Value::Null, |Lenient(value)| value),
    }
}

/// A YAML value read the way SnakeYAML does: a repeated key's last value stands.
struct Lenient(Value);

impl<'de> serde::Deserialize<'de> for Lenient {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(LenientVisitor)
    }
}

struct LenientVisitor;

impl<'de> serde::de::Visitor<'de> for LenientVisitor {
    type Value = Lenient;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a YAML value")
    }
    fn visit_bool<E>(self, v: bool) -> Result<Lenient, E> {
        Ok(Lenient(Value::Bool(v)))
    }
    fn visit_i64<E>(self, v: i64) -> Result<Lenient, E> {
        Ok(Lenient(Value::from(v)))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Lenient, E> {
        Ok(Lenient(Value::from(v)))
    }
    fn visit_f64<E>(self, v: f64) -> Result<Lenient, E> {
        Ok(Lenient(serde_json::Number::from_f64(v).map_or(Value::Null, Value::Number)))
    }
    fn visit_str<E>(self, v: &str) -> Result<Lenient, E> {
        Ok(Lenient(Value::String(v.to_owned())))
    }
    fn visit_string<E>(self, v: String) -> Result<Lenient, E> {
        Ok(Lenient(Value::String(v)))
    }
    fn visit_unit<E>(self) -> Result<Lenient, E> {
        Ok(Lenient(Value::Null))
    }
    fn visit_none<E>(self) -> Result<Lenient, E> {
        Ok(Lenient(Value::Null))
    }
    fn visit_some<D: serde::Deserializer<'de>>(self, d: D) -> Result<Lenient, D::Error> {
        serde::Deserialize::deserialize(d)
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Lenient, A::Error> {
        let mut items = Vec::new();
        while let Some(Lenient(item)) = seq.next_element()? {
            items.push(item);
        }
        Ok(Lenient(Value::Array(items)))
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Lenient, A::Error> {
        let mut out = Map::new();
        while let Some((Lenient(key), Lenient(value))) = map.next_entry()? {
            out.insert(scalar(&key).unwrap_or_default(), value);
        }
        Ok(Lenient(Value::Object(out)))
    }
    fn visit_enum<A: serde::de::EnumAccess<'de>>(self, data: A) -> Result<Lenient, A::Error> {
        // A tagged value (`!!str 1`): its value.
        let (_, variant): (Lenient, _) = data.variant()?;
        serde::de::VariantAccess::newtype_variant(variant)
    }
}

/// `YamlConfiguration`.
#[derive(Clone, Debug)]
pub struct YamlConfiguration {
    pub(crate) root: Value,
    pub(crate) path: Option<PathBuf>,
    /// `setDefaults`: read when the file doesn't say, never saved.
    pub(crate) defaults: Option<Value>,
    /// `options().pathSeparator`: what parts of a path are split on.
    pub(crate) separator: char,
}

impl YamlConfiguration {
    /// bukkit: org/bukkit/configuration/file/YamlConfiguration.<init>
    pub fn new() -> Self {
        Self { root: Value::Object(Map::new()), path: None, defaults: None, separator: '.' }
    }
    /// bukkit: org/bukkit/configuration/file/YamlConfiguration.loadConfiguration
    pub fn load_configuration(path: &Path) -> Self {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let root = match parse(&text) {
            Value::Object(map) => Value::Object(map),
            _ => Value::Object(Map::new()),
        };
        Self { root, path: Some(path.to_owned()), defaults: None, separator: '.' }
    }
    fn in_tree<'a>(root: &'a Value, path: &str, separator: char) -> Option<&'a Value> {
        path.split(separator).try_fold(root, |value, key| value.get(key)).filter(|value| !value.is_null())
    }
    pub(crate) fn at(&self, path: &str) -> Option<&Value> {
        Self::in_tree(&self.root, path, self.separator).or_else(|| Self::in_tree(self.defaults.as_ref()?, path, self.separator))
    }
    /// bukkit: org/bukkit/configuration/Configuration.setDefaults, org/bukkit/configuration/MemoryConfiguration.setDefaults
    ///
    /// `JavaPlugin.getConfig` does this with the jar's `config.yml`: a key the file lacks still reads.
    pub fn set_defaults(&mut self, defaults: Value) {
        self.defaults = Some(defaults);
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.contains, org/bukkit/configuration/MemorySection.contains, org/bukkit/configuration/file/FileConfiguration.contains
    pub fn contains(&self, path: &str) -> bool {
        self.at(path).is_some()
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getString, org/bukkit/configuration/MemorySection.getString, org/bukkit/configuration/file/FileConfiguration.getString
    pub fn get_string(&self, path: &str) -> Option<String> {
        self.at(path).and_then(scalar)
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getInt, org/bukkit/configuration/MemorySection.getInt, org/bukkit/configuration/file/FileConfiguration.getInt
    pub fn get_int(&self, path: &str) -> i32 {
        self.at(path).and_then(Value::as_f64).map_or(0, |value| value as i32)
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getDouble, org/bukkit/configuration/MemorySection.getDouble, org/bukkit/configuration/file/FileConfiguration.getDouble
    pub fn get_double(&self, path: &str) -> f64 {
        self.at(path).and_then(Value::as_f64).unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isConfigurationSection, org/bukkit/configuration/MemorySection.isConfigurationSection, org/bukkit/configuration/file/FileConfiguration.isConfigurationSection
    pub fn is_configuration_section(&self, path: &str) -> bool {
        self.at(path).is_some_and(Value::is_object)
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getConfigurationSection, org/bukkit/configuration/MemorySection.getConfigurationSection, org/bukkit/configuration/file/FileConfiguration.getConfigurationSection
    pub fn get_configuration_section(&self, path: &str) -> Option<Self> {
        self.at(path).filter(|value| value.is_object()).map(|value| Self { root: value.clone(), path: None, defaults: None, separator: self.separator })
    }
    /// bukkit: org/bukkit/configuration/Configuration.addDefault, org/bukkit/configuration/MemoryConfiguration.addDefault, org/bukkit/configuration/file/FileConfiguration.addDefault
    ///
    /// With `copyDefaults(true)`, as nearly every plugin uses it: set unless the file has it.
    pub fn add_default(&mut self, path: &str, value: Value) {
        if !self.contains(path) {
            self.set(path, value);
        }
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.createSection, org/bukkit/configuration/MemorySection.createSection, org/bukkit/configuration/file/FileConfiguration.createSection
    pub fn create_section(&mut self, path: &str, section: Value) {
        self.set(path, section);
    }
    /// `getInt(path, default)`.
    pub fn get_int_or(&self, path: &str, default: i32) -> i32 {
        self.at(path).and_then(Value::as_f64).map_or(default, |value| value as i32)
    }
    /// `getString(path, default)`.
    pub fn get_string_or(&self, path: &str, default: &str) -> String {
        self.get_string(path).unwrap_or_else(|| default.to_owned())
    }
    /// `getBoolean(path, default)`.
    pub fn get_boolean_or(&self, path: &str, default: bool) -> bool {
        self.at(path).and_then(Value::as_bool).unwrap_or(default)
    }
    /// bukkit: org/bukkit/configuration/file/YamlConfiguration.saveToString, org/bukkit/configuration/file/FileConfiguration.saveToString
    pub fn save_to_string(&self) -> String {
        serde_yaml::to_string(&self.root).unwrap_or_default()
    }
    /// What `setDefaults` and `copyDefaults(true)` come to in one step: the missing keys, copied in.
    pub fn copy_defaults(&mut self, defaults: &Value) {
        fn merge(into: &mut Value, from: &Value) {
            if let (Some(into), Some(from)) = (into.as_object_mut(), from.as_object()) {
                for (key, value) in from {
                    match into.get_mut(key) {
                        Some(existing) => merge(existing, value),
                        None => {
                            into.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }
        merge(&mut self.root, defaults);
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getBoolean, org/bukkit/configuration/MemorySection.getBoolean, org/bukkit/configuration/file/FileConfiguration.getBoolean
    pub fn get_boolean(&self, path: &str) -> bool {
        self.at(path).and_then(Value::as_bool).unwrap_or(false)
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getStringList, org/bukkit/configuration/MemorySection.getStringList, org/bukkit/configuration/file/FileConfiguration.getStringList
    pub fn get_string_list(&self, path: &str) -> Vec<String> {
        self.at(path).and_then(Value::as_array).into_iter().flatten().filter_map(scalar).collect()
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getKeys, org/bukkit/configuration/MemorySection.getKeys, org/bukkit/configuration/file/FileConfiguration.getKeys
    pub fn get_keys(&self, path: &str) -> Vec<String> {
        let read = |root: &Value| -> Vec<String> {
            let section = if path.is_empty() { Some(root) } else { Self::in_tree(root, path, self.separator) };
            section.and_then(Value::as_object).map(|map| map.keys().cloned().collect()).unwrap_or_default()
        };
        let mut keys = read(&self.root);
        for key in self.defaults.as_ref().map(read).unwrap_or_default() {
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        keys
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.set, org/bukkit/configuration/MemorySection.set, org/bukkit/configuration/file/FileConfiguration.set
    pub fn set(&mut self, path: &str, value: Value) {
        let mut keys: Vec<&str> = path.split(self.separator).collect();
        let Some(last) = keys.pop() else {
            return;
        };
        let mut here = &mut self.root;
        for key in keys {
            if !here.get(key).is_some_and(Value::is_object) {
                here[key] = Value::Object(Map::new());
            }
            here = &mut here[key];
        }
        if value.is_null() {
            if let Some(map) = here.as_object_mut() {
                map.remove(last);
            }
        } else {
            here[last] = value;
        }
    }
    /// bukkit~: org/bukkit/configuration/file/FileConfiguration.save, org/bukkit/configuration/file/YamlConfiguration.save | comments in the file are lost
    pub fn save(&self, path: Option<&Path>) -> std::io::Result<()> {
        let Some(path) = path.or(self.path.as_deref()) else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_yaml::to_string(&self.root).unwrap_or_default())
    }
}

impl Default for YamlConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dotted_paths_read_nested_values() {
        let mut config = YamlConfiguration::new();
        config.root = parse("homes:\n  max: 3\n  names: [a, b]\nversion: 6.3\nflag: true\n");
        assert_eq!(config.get_int("homes.max"), 3);
        assert_eq!(config.get_string_list("homes.names"), ["a", "b"]);
        assert_eq!(config.get_string("version").as_deref(), Some("6.3"));
        assert!(config.get_boolean("flag") && !config.get_boolean("missing"));
        assert_eq!(config.get_keys("homes"), ["max", "names"]);
    }

    #[test]
    fn defaults_read_when_the_file_is_silent_and_are_not_saved() {
        let mut config = YamlConfiguration::new();
        config.root = parse("a: 1\n");
        config.set_defaults(parse("a: 9\nb: 2\nc:\n  d: 3\n"));
        assert_eq!(config.get_int("a"), 1, "the file wins");
        assert_eq!(config.get_int("b"), 2);
        assert_eq!(config.get_keys(""), ["a", "b", "c"]);
        assert!(!config.save_to_string().contains('b'));
    }

    #[test]
    fn set_makes_sections_and_null_removes() {
        let mut config = YamlConfiguration::new();
        config.set("a.b.c", json!(5));
        assert!(config.contains("a.b.c"));
        config.set("a.b.c", Value::Null);
        assert!(!config.contains("a.b.c"));
    }

    #[test]
    fn a_saved_file_reads_back() {
        let path = std::env::temp_dir().join(format!("horizon-bukkit-{}.yml", std::process::id()));
        let mut config = YamlConfiguration::new();
        config.set("x.y", json!("z"));
        config.save(Some(&path)).unwrap();
        let again = YamlConfiguration::load_configuration(&path);
        let _ = std::fs::remove_file(&path);
        assert_eq!(again.get_string("x.y").as_deref(), Some("z"));
    }
}

#[cfg(test)]
mod lenient_tests {
    #[test]
    fn a_repeated_key_keeps_its_last_value() {
        let value = super::parse("a: 1\nb:\n  c: x\na: 2\n");
        assert_eq!(value["a"], 2);
        assert_eq!(value["b"]["c"], "x");
    }
}
