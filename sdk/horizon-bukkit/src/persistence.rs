//! `org.bukkit.persistence`, `org.bukkit.NamespacedKey`, `org.bukkit.plugin.ServicesManager` and
//! `org.bukkit.plugin.messaging`.
use crate::{Player, Plugin};
use horizon_mod::{Answer, Value, query};
use serde_json::{Map, json};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// `org.bukkit.NamespacedKey`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamespacedKey(String);

impl NamespacedKey {
    /// bukkit: org/bukkit/NamespacedKey.<init>
    pub fn new(plugin: &Plugin, key: &str) -> Self {
        Self(format!("{}:{key}", plugin.get_name().to_lowercase()))
    }
    /// bukkit: org/bukkit/NamespacedKey.minecraft
    pub fn minecraft(key: &str) -> Self {
        Self(format!("minecraft:{key}"))
    }
    /// bukkit: org/bukkit/NamespacedKey.fromString
    pub fn from_string(text: &str) -> Option<Self> {
        // Java refuses capitals here, where `new NamespacedKey(plugin, key)` lowers the plugin's name.
        let key = if text.contains(':') { text.to_owned() } else { format!("minecraft:{text}") };
        let valid = key.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-./:".contains(&b));
        (valid && key.matches(':').count() == 1).then_some(Self(key))
    }
    /// bukkit: org/bukkit/NamespacedKey.getKey
    pub fn get_key(&self) -> &str {
        self.0.split_once(':').map_or(&self.0, |(_, key)| key)
    }
    /// bukkit: org/bukkit/NamespacedKey.getNamespace
    pub fn get_namespace(&self) -> &str {
        self.0.split_once(':').map_or("minecraft", |(space, _)| space)
    }
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// `org.bukkit.persistence.PersistentDataContainer`: `BukkitValues` of a player's file, or a file of
/// a plugin's own.
#[derive(Clone, Debug)]
pub struct PersistentDataContainer {
    pub(crate) file: PathBuf,
    pub(crate) holder: String,
    /// Whose `BukkitValues` these are. A nested container's `holder` is its path inside them.
    pub(crate) player: Option<String>,
}

pub(crate) fn read_all(file: &PathBuf) -> Map<String, Value> {
    std::fs::read(file).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default()
}

impl PersistentDataContainer {
    pub(crate) fn of_player(uuid: String) -> Self {
        Self { file: PathBuf::new(), holder: String::new(), player: Some(uuid.replace('-', "")) }
    }
    fn nesting(&self) -> impl Iterator<Item = &str> {
        self.holder.split('/').filter(|part| !part.is_empty())
    }
    /// What this holder has.
    pub(crate) fn values(&self) -> Map<String, Value> {
        let Some(uuid) = &self.player else {
            return read_all(&self.file).remove(&self.holder).and_then(|values| values.as_object().cloned()).unwrap_or_default();
        };
        let all = query(json!({ "op": "core_player_values", "uuid": uuid }));
        self.nesting().fold(&all, |at, part| &at[part]).as_object().cloned().unwrap_or_default()
    }
    /// What NBT each value is, by the same keys: JSON can't say, and a byte has to stay one.
    pub(crate) fn kinds(&self) -> Map<String, Value> {
        let Some(uuid) = &self.player else {
            return read_all(&self.file).remove(&format!("{}\u{0}kinds", self.holder)).and_then(|kinds| kinds.as_object().cloned()).unwrap_or_default();
        };
        let all = query(json!({ "op": "core_player_values", "uuid": uuid, "kinds": true }));
        self.nesting().fold(&all, |at, part| &at[part]).as_object().cloned().unwrap_or_default()
    }
    /// This holder's values, as they now are.
    pub(crate) fn keep(&self, values: Map<String, Value>) {
        self.keep_kinds(values, self.kinds());
    }
    /// With what each value is, for those whose JSON doesn't say.
    pub(crate) fn keep_kinds(&self, values: Map<String, Value>, mut kinds: Map<String, Value>) {
        kinds.retain(|key, _| values.contains_key(key));
        let Some(uuid) = &self.player else {
            let mut all = read_all(&self.file);
            all.insert(self.holder.clone(), Value::Object(values));
            all.insert(format!("{}\u{0}kinds", self.holder), Value::Object(kinds));
            if let Some(parent) = self.file.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&self.file, Value::Object(all).to_string());
            return;
        };
        let mut all = query(json!({ "op": "core_player_values", "uuid": uuid }));
        let mut tree = query(json!({ "op": "core_player_values", "uuid": uuid, "kinds": true }));
        let (mut at, mut kind) = (&mut all, &mut tree);
        for part in self.nesting() {
            if !at.is_object() {
                *at = json!({});
            }
            if !kind.is_object() {
                *kind = json!({});
            }
            at = &mut at[part];
            kind = &mut kind[part];
        }
        *at = Value::Object(values);
        *kind = Value::Object(kinds);
        query(json!({ "op": "core_player_values_set", "uuid": uuid, "values": typed_snbt(&all, &tree) }));
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.set
    ///
    /// By what the JSON is: `set_typed` for a type of Java's.
    pub fn set(&self, key: &NamespacedKey, value: Value) {
        self.set_kind(key, value, Value::Null);
    }
    /// `set(key, type, value)`.
    pub fn set_typed<T: crate::fam_core::PersistentDataType>(&self, key: &NamespacedKey, kind: &T, value: &T::Complex) {
        let primitive = kind.to_primitive(value, &crate::fam_core::PersistentDataAdapterContext);
        let nbt = kind.nbt_kind(&primitive);
        self.set_kind(key, primitive, nbt);
    }
    pub(crate) fn set_kind(&self, key: &NamespacedKey, value: Value, kind: Value) {
        let mut values = self.values();
        values.insert(key.as_str().to_owned(), value);
        let mut kinds = self.kinds();
        if kind.is_null() {
            kinds.remove(key.as_str());
        } else {
            kinds.insert(key.as_str().to_owned(), kind);
        }
        self.keep_kinds(values, kinds);
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.get, io/papermc/paper/persistence/PersistentDataContainerView.get, org/bukkit/persistence/PersistentDataContainerView.get
    pub fn get(&self, key: &NamespacedKey) -> Option<Value> {
        self.values().get(key.as_str()).cloned()
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.has, io/papermc/paper/persistence/PersistentDataContainerView.has
    pub fn has(&self, key: &NamespacedKey) -> bool {
        self.get(key).is_some()
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.remove
    pub fn remove(&self, key: &NamespacedKey) {
        let mut values = self.values();
        values.remove(key.as_str());
        self.keep(values);
    }
}

/// One tag's letter, as `tag_kinds` in `paper_fam_paper.rs` gives it, for a value with none kept.
pub(crate) fn natural_kind(value: &Value) -> Value {
    match value {
        Value::Bool(_) => json!("b"),
        Value::String(_) => json!("t"),
        Value::Number(number) => match number.as_i64() {
            Some(whole) if i32::try_from(whole).is_ok() => json!("i"),
            Some(_) => json!("l"),
            None => json!("d"),
        },
        Value::Array(values) => Value::Array(values.iter().map(natural_kind).collect()),
        Value::Object(values) => Value::Object(values.iter().map(|(key, value)| (key.clone(), natural_kind(value))).collect()),
        Value::Null => Value::Null,
    }
}

/// SNBT of `value` with the tags `kind` says, and those of its JSON where it says nothing.
pub(crate) fn typed_snbt(value: &Value, kind: &Value) -> String {
    let list = |values: &[Value], suffix: &str| values.iter().map(|one| format!("{one}{suffix}")).collect::<Vec<_>>().join(",");
    match (value, kind.as_str()) {
        (Value::Array(values), Some("B")) => format!("[B;{}]", list(values, "b")),
        (Value::Array(values), Some("I")) => format!("[I;{}]", list(values, "")),
        (Value::Array(values), Some("L")) => format!("[L;{}]", list(values, "L")),
        (Value::Number(_), Some(letter @ ("b" | "s" | "f" | "d"))) => format!("{value}{letter}"),
        (Value::Number(_), Some("l")) => format!("{value}L"),
        (Value::Number(_), Some("i")) => value.to_string(),
        (Value::Number(number), None) if number.as_i64().is_some_and(|whole| i32::try_from(whole).is_err()) => format!("{value}L"),
        (Value::Array(values), _) => {
            let one = |at: usize| kind.get(at).unwrap_or(&Value::Null).clone();
            format!("[{}]", values.iter().enumerate().map(|(at, value)| typed_snbt(value, &one(at))).collect::<Vec<_>>().join(","))
        }
        (Value::Object(values), _) => {
            let one = |key: &str| kind.get(key).unwrap_or(&Value::Null).clone();
            format!("{{{}}}", values.iter().map(|(key, value)| format!("{}:{}", crate::data::quoted(key), typed_snbt(value, &one(key)))).collect::<Vec<_>>().join(","))
        }
        _ => crate::data::snbt(value),
    }
}

impl Player {
    /// bukkit: org/bukkit/persistence/PersistentDataHolder.getPersistentDataContainer, org/bukkit/entity/Player.getPersistentDataContainer
    ///
    /// `BukkitValues` of their file: written with it, and every plugin's, as Java's.
    pub fn get_persistent_data_container(&self) -> PersistentDataContainer {
        PersistentDataContainer::of_player(self.get_unique_id())
    }
}

impl Plugin {
    /// `Player::get_persistent_data_container`, by way of the plugin, as ports had it.
    pub fn persistent_data(&self, player: &Player) -> PersistentDataContainer {
        player.get_persistent_data_container()
    }
}

// ---- ServicesManager ----

type Service = Arc<dyn Any + Send + Sync>;

pub(crate) static SERVICES: Mutex<Option<HashMap<TypeId, Service>>> = Mutex::new(None);

/// A service inside this library, by its Rust type. It's listed by that type's name like any other,
/// and has no answer for another plugin: `provide_service` is for those.
pub fn register_service<T: Any + Send + Sync>(service: T) {
    SERVICES.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(TypeId::of::<T>(), Arc::new(service));
    if let Some(own) = crate::fam_core::get_own_plugin() {
        provide_service(&own, std::any::type_name::<T>(), "Normal", |_, _| Value::Null);
    }
}

/// `register_service`'s service.
pub fn get_service<T: Any + Send + Sync>() -> Option<Arc<T>> {
    SERVICES.lock().unwrap_or_else(|e| e.into_inner()).as_ref()?.get(&TypeId::of::<T>())?.clone().downcast().ok()
}

/// `register_service`'s, taken back.
pub fn unregister_service<T: Any + Send + Sync>() {
    if let Some(all) = SERVICES.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        all.remove(&TypeId::of::<T>());
    }
    if let Some(own) = crate::fam_core::get_own_plugin() {
        withdraw_service(&own, std::any::type_name::<T>());
    }
}

// ---- Services between plugins ----

type ServiceHandler = Arc<dyn Fn(&str, &horizon_mod::Value) -> horizon_mod::Value + Send + Sync>;

static PROVIDED: Mutex<Option<HashMap<String, ServiceHandler>>> = Mutex::new(None);

/// What a service did while the game wasn't there to tell: as a plugin starts, for the first tick.
static SERVICE_NEWS: Mutex<Vec<(&'static str, Value)>> = Mutex::new(Vec::new());

/// `ServiceRegisterEvent` and `ServiceUnregisterEvent`, for every plugin.
fn raise_service(hook: &'static str, row: Value) {
    if query(json!({ "op": "core_raise", "event": hook, "payload": row })).is_null() {
        SERVICE_NEWS.lock().unwrap_or_else(|e| e.into_inner()).push((hook, row));
    }
}

/// Each tick: what couldn't be told yet.
pub(crate) fn raise_waiting_services() {
    let waiting = std::mem::take(&mut *SERVICE_NEWS.lock().unwrap_or_else(|e| e.into_inner()));
    for (hook, row) in waiting {
        raise_service(hook, row);
    }
}

/// bukkit: org/bukkit/plugin/ServicesManager.register
///
/// A service any plugin can reach, by its Java class's name and a priority from `service_priority`. A Rust type can't
/// cross from one library to another, so a call is a method's name and JSON, both ways.
pub fn provide_service(plugin: &crate::Plugin, name: &str, priority: &str, handler: impl Fn(&str, &horizon_mod::Value) -> horizon_mod::Value + Send + Sync + 'static) {
    PROVIDED.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(name.to_owned(), Arc::new(handler));
    query(json!({ "op": "service_add", "name": name, "plugin": plugin.get_name(), "priority": priority }));
    raise_service("service_register", json!({ "name": name, "plugin": plugin.get_name(), "priority": priority }));
}

/// bukkit: org/bukkit/plugin/ServicesManager.unregister
pub fn withdraw_service(plugin: &crate::Plugin, name: &str) {
    if let Some(all) = PROVIDED.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        all.remove(name);
    }
    let row = service_registrations(name).into_iter().find(|provider| provider.plugin == plugin.get_name());
    query(json!({ "op": "service_remove", "name": name, "plugin": plugin.get_name() }));
    if let Some(row) = row {
        raise_service("service_unregister", json!({ "name": row.name, "plugin": row.plugin, "priority": row.priority }));
    }
}

/// bukkit: org/bukkit/plugin/ServicesManager.unregisterAll
///
/// Every service of a plugin, as going off does.
pub fn unregister_all_services(plugin: &crate::Plugin) {
    if crate::fam_core::get_own_plugin().is_some_and(|own| own.get_name() == plugin.get_name()) {
        SERVICES.lock().unwrap_or_else(|e| e.into_inner()).take();
    }
    for provider in plugin_services(plugin.get_name()) {
        withdraw_service(plugin, &provider.name);
    }
}

/// bukkit: org/bukkit/plugin/ServicesManager.getRegistrations
///
/// What one plugin provides.
pub fn plugin_services(plugin: &str) -> Vec<ServiceProvider> {
    query(json!({ "op": "service_list" })).as_array().into_iter().flatten().filter(|row| row["plugin"].as_str().is_some_and(|name| name == plugin)).filter_map(ServiceProvider::read).collect()
}

/// bukkit: org/bukkit/plugin/ServicesManager.isProvidedFor
pub fn is_service_provided(name: &str) -> bool {
    service_registration(name).is_some()
}

/// `org.bukkit.plugin.ServicesManager`: what's here is the functions of this module, for a port that goes through one.
#[derive(Clone, Copy, Debug, Default)]
pub struct ServicesManager;

/// bukkit: org/bukkit/Bukkit.getServicesManager, org/bukkit/Server.getServicesManager
pub fn get_services_manager() -> ServicesManager {
    ServicesManager
}

impl ServicesManager {
    /// bukkit: org/bukkit/plugin/ServicesManager.register
    pub fn register(&self, plugin: &crate::Plugin, name: &str, priority: &str, handler: impl Fn(&str, &horizon_mod::Value) -> horizon_mod::Value + Send + Sync + 'static) {
        provide_service(plugin, name, priority, handler);
    }
    /// bukkit: org/bukkit/plugin/ServicesManager.load, org/bukkit/plugin/ServicesManager.getRegistration
    pub fn load(&self, name: &str) -> Option<ServiceProvider> {
        service_registration(name)
    }
    /// bukkit: org/bukkit/plugin/ServicesManager.getRegistrations
    pub fn get_registrations(&self, name: &str) -> Vec<ServiceProvider> {
        service_registrations(name)
    }
    /// bukkit: org/bukkit/plugin/ServicesManager.getKnownServices
    pub fn get_known_services(&self) -> Vec<String> {
        service_names()
    }
    /// bukkit: org/bukkit/plugin/ServicesManager.isProvidedFor
    pub fn is_provided_for(&self, name: &str) -> bool {
        is_service_provided(name)
    }
    /// bukkit: org/bukkit/plugin/ServicesManager.unregister
    pub fn unregister(&self, plugin: &crate::Plugin, name: &str) {
        withdraw_service(plugin, name);
    }
    /// bukkit: org/bukkit/plugin/ServicesManager.unregisterAll
    pub fn unregister_all(&self, plugin: &crate::Plugin) {
        unregister_all_services(plugin);
    }
}

/// bukkit: org/bukkit/plugin/RegisteredServiceProvider
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceProvider {
    name: String,
    plugin: String,
    priority: String,
}

impl ServiceProvider {
    pub(crate) fn read(row: &horizon_mod::Value) -> Option<Self> {
        Some(Self { name: row["name"].as_str()?.to_owned(), plugin: row["plugin"].as_str()?.to_owned(), priority: row["priority"].as_str()?.to_owned() })
    }
    /// bukkit: org/bukkit/plugin/RegisteredServiceProvider.getService
    pub fn get_service(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/plugin/RegisteredServiceProvider.getPlugin
    pub fn get_plugin(&self) -> Option<crate::fam_core::LoadedPlugin> {
        crate::fam_core::PluginManager.get_plugin(&self.plugin)
    }
    /// The plugin's name, which holds whether it's in `get_plugin`'s list yet or not.
    pub fn get_plugin_name(&self) -> &str {
        &self.plugin
    }
    /// bukkit: org/bukkit/plugin/RegisteredServiceProvider.getPriority
    pub fn get_priority(&self) -> &str {
        &self.priority
    }
    /// bukkit: org/bukkit/plugin/RegisteredServiceProvider.getProvider
    ///
    /// One method of the provider, in whichever plugin it lives.
    pub fn call(&self, method: &str, args: &horizon_mod::Value) -> horizon_mod::Value {
        horizon_mod::query(serde_json::json!({ "op": "core_command_call", "plugin": self.plugin, "kind": "service", "service": self.name, "method": method, "call": args }))
    }
}

/// The other end of `ServiceProvider::call`.
pub(crate) fn answer_service(value: &horizon_mod::Value) -> horizon_mod::Value {
    let handler = PROVIDED.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|all| all.get(value["service"].as_str().unwrap_or_default()).cloned());
    handler.map_or(horizon_mod::Value::Null, |handler| handler(value["method"].as_str().unwrap_or_default(), &value["call"]))
}

/// bukkit: org/bukkit/plugin/ServicesManager.getRegistrations
///
/// Everyone providing a service, the highest priority first.
pub fn service_registrations(name: &str) -> Vec<ServiceProvider> {
    horizon_mod::query(serde_json::json!({ "op": "service_list", "name": name })).as_array().into_iter().flatten().filter_map(ServiceProvider::read).collect()
}

/// bukkit: org/bukkit/plugin/ServicesManager.getRegistration, org/bukkit/plugin/ServicesManager.load
pub fn service_registration(name: &str) -> Option<ServiceProvider> {
    service_registrations(name).into_iter().next()
}

/// bukkit: org/bukkit/plugin/ServicesManager.getKnownServices
pub fn service_names() -> Vec<String> {
    let mut names: Vec<String> = horizon_mod::query(serde_json::json!({ "op": "service_list" })).as_array().into_iter().flatten().filter_map(|row| row["name"].as_str().map(str::to_owned)).collect();
    names.dedup();
    names
}

// ---- Plugin messaging ----

impl Plugin {
    /// bukkit: org/bukkit/plugin/messaging/Messenger.registerOutgoingPluginChannel, org/bukkit/Server.getMessenger, org/bukkit/plugin/messaging/Messenger.unregisterOutgoingPluginChannel
    pub fn register_outgoing_plugin_channel(&self, channel: &str) {
        OUTGOING.lock().unwrap_or_else(|e| e.into_inner()).insert(channel.to_owned());
    }

    /// bukkit: org/bukkit/plugin/messaging/Messenger.registerIncomingPluginChannel, org/bukkit/plugin/messaging/PluginMessageListener.onPluginMessageReceived
    pub fn register_incoming_plugin_channel(&self, channel: &str, listener: impl Fn(&str, &Player, &[u8]) + Send + Sync + 'static) {
        let listener = std::sync::Arc::new(listener);
        crate::fam_core::note_channel_listener(channel, listener.clone());
        INCOMING.lock().unwrap_or_else(|e| e.into_inner()).insert(channel.to_owned());
        query(json!({ "op": "listen_channel", "channel": channel }));
        let wanted = channel.to_owned();
        self.host.on("plugin_message", move |value| {
            if value["channel"].as_str() == Some(wanted.as_str())
                && let Some(player) = value["uuid"].as_str().and_then(crate::fam_ess::get_player_by_id).or_else(|| Player::read(value))
            {
                let bytes: Vec<u8> = value["data"].as_array().into_iter().flatten().filter_map(|b| b.as_u64().map(|b| b as u8)).collect();
                listener(&wanted, &player, &bytes);
            }
            Answer::Pass
        });
    }
}

impl Player {
    /// bukkit: org/bukkit/entity/Player.sendPluginMessage, org/bukkit/plugin/messaging/PluginMessageRecipient.sendPluginMessage
    pub fn send_plugin_message(&self, channel: &str, data: &[u8]) {
        query(json!({ "op": "plugin_message", "uuid": self.uuid(), "channel": channel, "data": data }));
    }
    /// bukkit: org/bukkit/entity/Player.sendPluginMessage, org/bukkit/plugin/messaging/PluginMessageRecipient.sendPluginMessage
    ///
    /// Java's form, the sending plugin first (`player.sendPluginMessage(plugin, "bungeecord:main", data)`).
    pub fn send_plugin_message_from(&self, _plugin: &crate::Plugin, channel: &str, data: &[u8]) {
        self.send_plugin_message(channel, data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_split_and_refuse_bad_text() {
        let key = NamespacedKey::from_string("myplugin:some_key").unwrap();
        assert_eq!((key.get_namespace(), key.get_key()), ("myplugin", "some_key"));
        assert!(NamespacedKey::from_string("MyPlugin:key").is_none());
        assert_eq!(NamespacedKey::from_string("stone").unwrap().get_namespace(), "minecraft");
        assert!(NamespacedKey::from_string("bad key!").is_none());
    }

    #[test]
    fn a_container_keeps_values_by_holder() {
        let file = std::env::temp_dir().join(format!("horizon-pdc-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&file);
        let (one, two) = (PersistentDataContainer { file: file.clone(), holder: "a".into(), player: None }, PersistentDataContainer { file: file.clone(), holder: "b".into(), player: None });
        let key = NamespacedKey::minecraft("score");
        one.set(&key, json!(5));
        assert_eq!(one.get(&key), Some(json!(5)));
        assert!(!two.has(&key));
        one.remove(&key);
        assert!(!one.has(&key));
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn services_come_back_by_type() {
        struct Economy(i32);
        register_service(Economy(7));
        assert_eq!(get_service::<Economy>().unwrap().0, 7);
        assert!(get_service::<String>().is_none());
    }
}

/// bukkit: org/bukkit/plugin/messaging/Messenger.isReservedChannel
pub fn is_reserved_channel(channel: &str) -> bool {
    matches!(channel, "minecraft:register" | "minecraft:unregister" | "REGISTER" | "UNREGISTER")
}

static OUTGOING: std::sync::Mutex<std::collections::BTreeSet<String>> = std::sync::Mutex::new(std::collections::BTreeSet::new());
static INCOMING: std::sync::Mutex<std::collections::BTreeSet<String>> = std::sync::Mutex::new(std::collections::BTreeSet::new());

impl Plugin {
    /// bukkit: org/bukkit/plugin/messaging/Messenger.unregisterOutgoingPluginChannel
    pub fn unregister_outgoing_plugin_channel(&self, channel: &str) {
        OUTGOING.lock().unwrap_or_else(|e| e.into_inner()).remove(channel);
    }
    /// bukkit: org/bukkit/plugin/messaging/Messenger.unregisterIncomingPluginChannel
    pub fn unregister_incoming_plugin_channel(&self, channel: &str) {
        INCOMING.lock().unwrap_or_else(|e| e.into_inner()).remove(channel);
    }
    /// bukkit: org/bukkit/plugin/messaging/Messenger.isOutgoingChannelRegistered
    pub fn is_outgoing_channel_registered(&self, channel: &str) -> bool {
        OUTGOING.lock().unwrap_or_else(|e| e.into_inner()).contains(channel)
    }
    /// bukkit: org/bukkit/plugin/messaging/Messenger.isIncomingChannelRegistered
    pub fn is_incoming_channel_registered(&self, channel: &str) -> bool {
        INCOMING.lock().unwrap_or_else(|e| e.into_inner()).contains(channel)
    }
}

/// bukkit: org/bukkit/plugin/messaging/Messenger.getOutgoingChannels
pub fn get_outgoing_channels() -> Vec<String> {
    OUTGOING.lock().unwrap_or_else(|e| e.into_inner()).iter().cloned().collect()
}

/// bukkit: org/bukkit/plugin/messaging/Messenger.getIncomingChannels
pub fn get_incoming_channels() -> Vec<String> {
    INCOMING.lock().unwrap_or_else(|e| e.into_inner()).iter().cloned().collect()
}
