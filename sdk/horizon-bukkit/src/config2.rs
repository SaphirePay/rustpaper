//! The rest of `org.bukkit.configuration`, and `org.bukkit.permissions`.
use crate::config::YamlConfiguration;
use crate::{Player, get_online_players};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Mutex;

macro_rules! list_getter {
    ($(#[$doc:meta])* $name:ident, $item:ty, $read:expr) => {
        $(#[$doc])*
        pub fn $name(&self, path: &str) -> Vec<$item> {
            self.get(path).and_then(|value| value.as_array().cloned()).unwrap_or_default().iter().filter_map($read).collect()
        }
    };
}

impl YamlConfiguration {
    /// bukkit: org/bukkit/configuration/ConfigurationSection.get, org/bukkit/configuration/MemorySection.get
    pub fn get(&self, path: &str) -> Option<Value> {
        self.at(path).cloned()
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isSet, org/bukkit/configuration/MemorySection.isSet
    pub fn is_set(&self, path: &str) -> bool {
        self.contains(path)
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isString, org/bukkit/configuration/MemorySection.isString
    pub fn is_string(&self, path: &str) -> bool {
        self.get(path).is_some_and(|value| value.is_string())
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isInt, org/bukkit/configuration/MemorySection.isInt
    pub fn is_int(&self, path: &str) -> bool {
        self.get(path).is_some_and(|value| value.is_i64())
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isLong, org/bukkit/configuration/MemorySection.isLong
    pub fn is_long(&self, path: &str) -> bool {
        self.is_int(path)
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isDouble, org/bukkit/configuration/MemorySection.isDouble
    pub fn is_double(&self, path: &str) -> bool {
        self.get(path).is_some_and(|value| value.is_f64())
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isBoolean, org/bukkit/configuration/MemorySection.isBoolean
    pub fn is_boolean(&self, path: &str) -> bool {
        self.get(path).is_some_and(|value| value.is_boolean())
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isList, org/bukkit/configuration/MemorySection.isList
    pub fn is_list(&self, path: &str) -> bool {
        self.get(path).is_some_and(|value| value.is_array())
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getLong, org/bukkit/configuration/MemorySection.getLong
    pub fn get_long(&self, path: &str) -> i64 {
        self.get(path).and_then(|value| value.as_i64()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getList, org/bukkit/configuration/MemorySection.getList
    pub fn get_list(&self, path: &str) -> Vec<Value> {
        self.get(path).and_then(|value| value.as_array().cloned()).unwrap_or_default()
    }
    list_getter!(
        /// bukkit: org/bukkit/configuration/ConfigurationSection.getIntegerList, org/bukkit/configuration/MemorySection.getIntegerList
        get_integer_list, i32, |value| value.as_i64().map(|value| value as i32)
    );
    list_getter!(
        /// bukkit: org/bukkit/configuration/ConfigurationSection.getBooleanList, org/bukkit/configuration/MemorySection.getBooleanList
        get_boolean_list, bool, Value::as_bool
    );
    list_getter!(
        /// bukkit: org/bukkit/configuration/ConfigurationSection.getDoubleList, org/bukkit/configuration/MemorySection.getDoubleList
        get_double_list, f64, Value::as_f64
    );
    list_getter!(
        /// bukkit: org/bukkit/configuration/ConfigurationSection.getFloatList, org/bukkit/configuration/MemorySection.getFloatList
        get_float_list, f32, |value| value.as_f64().map(|value| value as f32)
    );
    list_getter!(
        /// bukkit: org/bukkit/configuration/ConfigurationSection.getLongList, org/bukkit/configuration/MemorySection.getLongList
        get_long_list, i64, Value::as_i64
    );
    list_getter!(
        /// bukkit: org/bukkit/configuration/ConfigurationSection.getByteList, org/bukkit/configuration/MemorySection.getByteList
        get_byte_list, i8, |value| value.as_i64().map(|value| value as i8)
    );
    list_getter!(
        /// bukkit: org/bukkit/configuration/ConfigurationSection.getShortList, org/bukkit/configuration/MemorySection.getShortList
        get_short_list, i16, |value| value.as_i64().map(|value| value as i16)
    );
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getMapList, org/bukkit/configuration/MemorySection.getMapList
    pub fn get_map_list(&self, path: &str) -> Vec<serde_json::Map<String, Value>> {
        self.get_list(path).into_iter().filter_map(|value| value.as_object().cloned()).collect()
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getValues, org/bukkit/configuration/MemorySection.getValues
    pub fn get_values(&self) -> serde_json::Map<String, Value> {
        self.root.as_object().cloned().unwrap_or_default()
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getVector, org/bukkit/configuration/MemorySection.getVector
    pub fn get_vector(&self, path: &str) -> Option<crate::util::Vector> {
        let value = self.get(path)?;
        Some(crate::util::Vector::new(value["x"].as_f64()?, value["y"].as_f64()?, value["z"].as_f64()?))
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isVector, org/bukkit/configuration/MemorySection.isVector
    pub fn is_vector(&self, path: &str) -> bool {
        self.get_vector(path).is_some()
    }
}

// ---- Permissions ----

/// `org.bukkit.permissions.Permission`: what `plugin.yml` declares, or a plugin in code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permission {
    pub name: String,
    pub default: String,
    pub description: String,
    pub children: BTreeMap<String, bool>,
}

impl Permission {
    pub fn new(name: &str, default: &str, description: &str) -> Self {
        Self { name: name.to_owned(), default: default.to_owned(), description: description.to_owned(), children: BTreeMap::new() }
    }
    /// `new Permission(name)`: Bukkit's default for a permission is operators only.
    /// bukkit: org/bukkit/permissions/Permission.<init>
    pub fn named(name: &str) -> Self {
        Self::new(name, "op", "")
    }
    /// `new Permission(name, description)`, Java's order.
    /// bukkit: org/bukkit/permissions/Permission.<init>
    pub fn described(name: &str, description: &str) -> Self {
        Self::new(name, "op", description)
    }
    /// bukkit: org/bukkit/permissions/Permission.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/permissions/Permission.getDefault
    pub fn get_default(&self) -> &str {
        &self.default
    }
    /// bukkit: org/bukkit/permissions/Permission.setDefault
    pub fn set_default(&mut self, default: &str) {
        default.clone_into(&mut self.default);
        crate::set_permission_default(&self.name, default);
    }
    /// bukkit: org/bukkit/permissions/Permission.getDescription
    pub fn get_description(&self) -> &str {
        &self.description
    }
    /// bukkit: org/bukkit/permissions/Permission.setDescription
    pub fn set_description(&mut self, description: &str) {
        description.clone_into(&mut self.description);
    }
    /// bukkit: org/bukkit/permissions/Permission.getChildren
    pub fn get_children(&self) -> &BTreeMap<String, bool> {
        &self.children
    }
    /// bukkit: org/bukkit/permissions/Permission.getPermissibles
    ///
    /// The players who have it.
    pub fn get_permissibles(&self) -> Vec<Player> {
        get_online_players().into_iter().filter(|player| player.has_permission(&self.name)).collect()
    }
}

/// bukkit: org/bukkit/permissions/PermissionDefault.getValue
pub fn permission_default_value(default: &str, operator: bool) -> bool {
    match default.to_ascii_uppercase().as_str() {
        "TRUE" => true,
        "FALSE" => false,
        "NOT_OP" | "NOT OP" => !operator,
        _ => operator,
    }
}

/// `org.bukkit.permissions.PermissionAttachment`: what a plugin granted one player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionAttachment {
    player: Player,
    pub(crate) plugin: String,
}

static ATTACHED: Mutex<BTreeMap<(String, String), BTreeMap<String, bool>>> = Mutex::new(BTreeMap::new());

impl Player {
    /// bukkit: org/bukkit/permissions/Permissible.addAttachment, org/bukkit/permissions/PermissibleBase.addAttachment
    pub fn add_attachment(&self, plugin: &crate::Plugin) -> PermissionAttachment {
        PermissionAttachment { player: self.clone(), plugin: plugin.get_name().to_owned() }
    }
    /// bukkit: org/bukkit/permissions/Permissible.isPermissionSet, org/bukkit/permissions/PermissibleBase.isPermissionSet
    pub fn is_permission_set(&self, node: &str) -> bool {
        // Any plugin's grant counts: the engine holds them all, as a Permissible does.
        crate::permission_is_granted(self.uuid(), node).is_some() || crate::fam_core::shared_permission(self.uuid(), self.is_op(), node).is_some()
    }
    /// bukkit: org/bukkit/permissions/Permissible.recalculatePermissions, org/bukkit/permissions/PermissibleBase.recalculatePermissions
    pub fn recalculate_permissions(&self) {}
}

/// The attachment of `player` that says something of `node`.
pub(crate) fn attachment_granting(player: &Player, node: &str) -> Option<PermissionAttachment> {
    let attached = ATTACHED.lock().unwrap_or_else(|e| e.into_inner());
    let ((_, plugin), _) = attached.iter().find(|((uuid, _), nodes)| uuid == player.uuid() && nodes.contains_key(node))?;
    Some(PermissionAttachment { player: player.clone(), plugin: plugin.clone() })
}

impl PermissionAttachment {
    /// bukkit: org/bukkit/permissions/PermissionAttachment.setPermission
    pub fn set_permission(&self, node: &str, value: bool) {
        ATTACHED.lock().unwrap_or_else(|e| e.into_inner()).entry((self.player.uuid().to_owned(), self.plugin.clone())).or_default().insert(node.to_owned(), value);
        crate::grant_permission(self.player.uuid(), node, Some(value));
    }
    /// bukkit: org/bukkit/permissions/PermissionAttachment.unsetPermission
    pub fn unset_permission(&self, node: &str) {
        if let Some(nodes) = ATTACHED.lock().unwrap_or_else(|e| e.into_inner()).get_mut(&(self.player.uuid().to_owned(), self.plugin.clone())) {
            nodes.remove(node);
        }
        crate::grant_permission(self.player.uuid(), node, None);
    }
    /// bukkit: org/bukkit/permissions/PermissionAttachment.getPermissions
    pub fn get_permissions(&self) -> BTreeMap<String, bool> {
        ATTACHED.lock().unwrap_or_else(|e| e.into_inner()).get(&(self.player.uuid().to_owned(), self.plugin.clone())).cloned().unwrap_or_default()
    }
    /// bukkit: org/bukkit/permissions/PermissionAttachment.getPermissible
    pub fn get_permissible(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/permissions/PermissionAttachment.remove
    pub fn remove(&self) -> bool {
        let nodes = ATTACHED.lock().unwrap_or_else(|e| e.into_inner()).remove(&(self.player.uuid().to_owned(), self.plugin.clone()));
        let had = nodes.is_some();
        crate::fam_core::attachment_removed(self);
        for (node, _) in nodes.into_iter().flatten() {
            crate::grant_permission(self.player.uuid(), &node, None);
        }
        had
    }
}
