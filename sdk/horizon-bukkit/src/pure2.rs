//! More classes that need no game: what a config file holds beyond plain values, commands as
//! objects, and map cursors and canvases.
#![allow(missing_docs)]
use crate::config::YamlConfiguration;
use crate::{Color, ItemStack, Location, Material, OfflinePlayer, World};
use horizon_mod::Value;
use serde_json::json;
use std::collections::BTreeMap;

impl YamlConfiguration {
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getCharacterList, org/bukkit/configuration/MemorySection.getCharacterList
    pub fn get_character_list(&self, path: &str) -> Vec<char> {
        self.get_string_list(path).iter().filter_map(|text| text.chars().next()).collect()
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getItemStack, org/bukkit/configuration/MemorySection.getItemStack
    ///
    /// A stack as `{ type: STONE, amount: 2 }`, what `set_item_stack` writes.
    pub fn get_item_stack(&self, path: &str) -> Option<ItemStack> {
        let value = self.get(path)?;
        let name = value["type"].as_str()?;
        Some(ItemStack::new(Material::of(&format!("minecraft:{}", name.to_ascii_lowercase())), value["amount"].as_i64().unwrap_or(1) as i32))
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isItemStack, org/bukkit/configuration/MemorySection.isItemStack
    pub fn is_item_stack(&self, path: &str) -> bool {
        self.get_item_stack(path).is_some()
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getColor, org/bukkit/configuration/MemorySection.getColor
    pub fn get_color(&self, path: &str) -> Option<Color> {
        let value = self.get(path)?;
        let part = |key: &str| value[key].as_u64().map(|part| part as u8);
        Some(Color::from_rgb(part("RED")?, part("GREEN")?, part("BLUE")?))
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isColor, org/bukkit/configuration/MemorySection.isColor
    pub fn is_color(&self, path: &str) -> bool {
        self.get_color(path).is_some()
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getLocation, org/bukkit/configuration/MemorySection.getLocation
    pub fn get_location(&self, path: &str) -> Option<Location> {
        let value = self.get(path)?;
        let number = |key: &str| value[key].as_f64();
        let world = World::of(value["world"].as_str().unwrap_or("minecraft:overworld"));
        Some(Location::new(world, number("x")?, number("y")?, number("z")?, number("yaw").unwrap_or(0.0) as f32, number("pitch").unwrap_or(0.0) as f32))
    }
    /// bukkit~: org/bukkit/configuration/ConfigurationSection.set, org/bukkit/configuration/MemorySection.set
    ///
    /// Java's `config.set(path, location)`: world, x, y, z, yaw and pitch, read back by `get_location`.
    pub fn set_location(&mut self, path: &str, location: &Location) {
        let map: serde_json::Map<String, Value> = serialize_location(location).into_iter().collect();
        self.set(path, Value::Object(map));
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isLocation, org/bukkit/configuration/MemorySection.isLocation
    pub fn is_location(&self, path: &str) -> bool {
        self.get_location(path).is_some()
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getOfflinePlayer, org/bukkit/configuration/MemorySection.getOfflinePlayer
    pub fn get_offline_player(&self, path: &str) -> Option<OfflinePlayer> {
        Some(crate::get_offline_player(&self.get_string(path)?))
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.isOfflinePlayer, org/bukkit/configuration/MemorySection.isOfflinePlayer
    pub fn is_offline_player(&self, path: &str) -> bool {
        self.get_offline_player(path).is_some()
    }
}

// ---- Commands as objects ----

/// `org.bukkit.command.Command`: what a command says about itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    name: String,
    label: String,
    description: String,
    usage: String,
    permission: Option<String>,
    permission_message: Option<String>,
    aliases: Vec<String>,
    registered: bool,
}

impl Command {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_owned(), label: name.to_owned(), description: String::new(), usage: format!("/{name}"), permission: None, permission_message: None, aliases: Vec::new(), registered: false }
    }
    /// bukkit: org/bukkit/command/Command.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/command/Command.setName
    pub fn set_name(&mut self, name: &str) -> bool {
        if self.registered {
            return false;
        }
        name.clone_into(&mut self.name);
        true
    }
    /// bukkit: org/bukkit/command/Command.getLabel
    pub fn get_label(&self) -> &str {
        &self.label
    }
    /// bukkit: org/bukkit/command/Command.setLabel
    pub fn set_label(&mut self, label: &str) -> bool {
        if self.registered {
            return false;
        }
        label.clone_into(&mut self.label);
        true
    }
    /// bukkit: org/bukkit/command/Command.getPermission
    pub fn get_permission(&self) -> Option<&str> {
        self.permission.as_deref()
    }
    /// bukkit: org/bukkit/command/Command.setPermission
    pub fn set_permission(&mut self, permission: &str) {
        self.permission = Some(permission.to_owned());
    }
    /// bukkit: org/bukkit/command/Command.getAliases
    pub fn get_aliases(&self) -> &[String] {
        &self.aliases
    }
    /// bukkit: org/bukkit/command/Command.setAliases
    pub fn set_aliases(&mut self, aliases: &[&str]) {
        self.aliases = aliases.iter().map(|alias| (*alias).to_owned()).collect();
    }
    /// bukkit: org/bukkit/command/Command.getDescription
    pub fn get_description(&self) -> &str {
        &self.description
    }
    /// bukkit: org/bukkit/command/Command.setDescription
    pub fn set_description(&mut self, description: &str) {
        description.clone_into(&mut self.description);
    }
    /// bukkit: org/bukkit/command/Command.getUsage
    pub fn get_usage(&self) -> &str {
        &self.usage
    }
    /// bukkit: org/bukkit/command/Command.setUsage
    pub fn set_usage(&mut self, usage: &str) {
        usage.clone_into(&mut self.usage);
    }
    /// bukkit: org/bukkit/command/Command.getPermissionMessage
    pub fn get_permission_message(&self) -> Option<&str> {
        self.permission_message.as_deref()
    }
    /// bukkit: org/bukkit/command/Command.setPermissionMessage
    pub fn set_permission_message(&mut self, message: &str) {
        self.permission_message = Some(message.to_owned());
    }
    /// bukkit: org/bukkit/command/Command.isRegistered
    pub fn is_registered(&self) -> bool {
        self.registered
    }
    /// bukkit: org/bukkit/command/Command.register
    pub fn register(&mut self) -> bool {
        self.registered = true;
        true
    }
    /// bukkit: org/bukkit/command/Command.unregister
    pub fn unregister(&mut self) -> bool {
        self.registered = false;
        true
    }
}

// ---- Maps ----

/// `org.bukkit.map.MapCursor`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapCursor {
    x: i8,
    y: i8,
    direction: u8,
    kind: u8,
    visible: bool,
    caption: Option<String>,
}

impl MapCursor {
    /// bukkit: org/bukkit/map/MapCursor.<init>
    pub fn new(x: i8, y: i8, direction: u8, kind: u8, visible: bool) -> Self {
        Self { x, y, direction: direction & 15, kind, visible, caption: None }
    }
    /// bukkit: org/bukkit/map/MapCursor.getX
    pub fn get_x(&self) -> i8 {
        self.x
    }
    /// bukkit: org/bukkit/map/MapCursor.getY
    pub fn get_y(&self) -> i8 {
        self.y
    }
    /// bukkit: org/bukkit/map/MapCursor.setX
    pub fn set_x(&mut self, x: i8) {
        self.x = x;
    }
    /// bukkit: org/bukkit/map/MapCursor.setY
    pub fn set_y(&mut self, y: i8) {
        self.y = y;
    }
    /// bukkit: org/bukkit/map/MapCursor.getDirection
    pub fn get_direction(&self) -> u8 {
        self.direction
    }
    /// bukkit: org/bukkit/map/MapCursor.setDirection
    pub fn set_direction(&mut self, direction: u8) {
        self.direction = direction & 15;
    }
    /// bukkit: org/bukkit/map/MapCursor.getRawType, org/bukkit/map/MapCursor.getType
    pub fn get_raw_type(&self) -> u8 {
        self.kind
    }
    /// bukkit: org/bukkit/map/MapCursor.setRawType, org/bukkit/map/MapCursor.setType
    pub fn set_raw_type(&mut self, kind: u8) {
        self.kind = kind;
    }
    /// bukkit: org/bukkit/map/MapCursor.isVisible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    /// bukkit: org/bukkit/map/MapCursor.setVisible
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    /// bukkit: org/bukkit/map/MapCursor.getCaption, org/bukkit/map/MapCursor.caption
    pub fn get_caption(&self) -> Option<&str> {
        self.caption.as_deref()
    }
    /// bukkit: org/bukkit/map/MapCursor.setCaption
    pub fn set_caption(&mut self, caption: Option<&str>) {
        self.caption = caption.map(str::to_owned);
    }
}

/// `org.bukkit.map.MapCursorCollection`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapCursorCollection(pub Vec<MapCursor>);

impl MapCursorCollection {
    /// bukkit: org/bukkit/map/MapCursorCollection.size
    pub fn size(&self) -> usize {
        self.0.len()
    }
    /// bukkit: org/bukkit/map/MapCursorCollection.getCursor
    pub fn get_cursor(&self, index: usize) -> Option<&MapCursor> {
        self.0.get(index)
    }
    /// bukkit: org/bukkit/map/MapCursorCollection.addCursor
    pub fn add_cursor(&mut self, cursor: MapCursor) {
        self.0.push(cursor);
    }
    /// bukkit: org/bukkit/map/MapCursorCollection.removeCursor
    pub fn remove_cursor(&mut self, cursor: &MapCursor) -> bool {
        let before = self.0.len();
        self.0.retain(|held| held != cursor);
        self.0.len() != before
    }
}

/// `org.bukkit.map.MapCanvas`: the 128 by 128 colour bytes a map shows, and the base it was drawn on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapCanvas {
    pixels: Vec<Option<u8>>,
    pub(crate) base: Vec<u8>,
    pub cursors: MapCursorCollection,
}

impl Default for MapCanvas {
    fn default() -> Self {
        Self { pixels: vec![None; 128 * 128], base: vec![0; 128 * 128], cursors: MapCursorCollection::default() }
    }
}

impl MapCanvas {
    fn index(x: i32, y: i32) -> Option<usize> {
        ((0..128).contains(&x) && (0..128).contains(&y)).then_some((y * 128 + x) as usize)
    }
    /// bukkit: org/bukkit/map/MapCanvas.setPixel
    pub fn set_pixel(&mut self, x: i32, y: i32, color: u8) {
        if let Some(index) = Self::index(x, y) {
            self.pixels[index] = Some(color);
        }
    }
    /// bukkit: org/bukkit/map/MapCanvas.getPixel
    pub fn get_pixel(&self, x: i32, y: i32) -> Option<u8> {
        self.pixels[Self::index(x, y)?]
    }
    /// bukkit: org/bukkit/map/MapCanvas.getBasePixel
    pub fn get_base_pixel(&self, x: i32, y: i32) -> u8 {
        Self::index(x, y).map_or(0, |index| self.base[index])
    }
    /// bukkit: org/bukkit/map/MapCanvas.getCursors
    pub fn get_cursors(&self) -> &MapCursorCollection {
        &self.cursors
    }
    /// bukkit: org/bukkit/map/MapCanvas.setCursors
    pub fn set_cursors(&mut self, cursors: MapCursorCollection) {
        self.cursors = cursors;
    }
    /// What the canvas shows, each pixel over its base.
    pub fn colors(&self) -> Vec<u8> {
        self.pixels.iter().zip(&self.base).map(|(pixel, base)| pixel.unwrap_or(*base)).collect()
    }
}

/// `org.bukkit.map.MapView$Scale`.
/// bukkit: org/bukkit/map/MapView$Scale.getValue
pub fn map_scale_value(name: &str) -> u8 {
    ["CLOSEST", "CLOSE", "NORMAL", "FAR", "FARTHEST"].iter().position(|scale| *scale == name).unwrap_or(3) as u8
}

/// A plain `serialize` of what has one, for `ConfigurationSerializable`.
/// bukkit: org/bukkit/configuration/serialization/ConfigurationSerializable.serialize
pub fn serialize_location(location: &Location) -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("world".to_owned(), json!(location.get_world().dimension())),
        ("x".to_owned(), json!(location.get_x())),
        ("y".to_owned(), json!(location.get_y())),
        ("z".to_owned(), json!(location.get_z())),
        ("yaw".to_owned(), json!(location.get_yaw())),
        ("pitch".to_owned(), json!(location.get_pitch())),
    ])
}
