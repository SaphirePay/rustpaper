//! Family ev_player: see plugin-tools/tools/family.
//!
//! Events of real menus, read from the packets `paper_fam_ev_player.rs` holds until the hub is back,
//! so a listener here has the whole API. An event whose class also has a row in `events.rs` or a
//! struct in `event.rs` is written out again here with what those can't reach: use this one.
#![allow(missing_docs)]
use crate::entity::Entity;
use crate::event::Event;
use crate::persistence::NamespacedKey;
use crate::{Block, Inventory, ItemStack, Location, OPEN_INVENTORIES, Player, World};
use horizon_mod::{Answer, Value, query};
use serde_json::json;
use std::collections::BTreeMap;

fn player_of(value: &Value) -> Option<Player> {
    Player::read(&value["player"]).or_else(|| Player::read(value)).or_else(|| Player::read(&query(json!({ "op": "player", "uuid": value["uuid"] })))).or_else(|| {
        // Inside a guest's turn the hub is out of the game's hands: it can still say who's acting.
        let acting = query(json!({ "op": "ev_player_acting" }));
        (acting["uuid"] == value["uuid"]).then(|| Player::read(&acting)).flatten()
    })
}

fn world_of(value: &Value) -> World {
    World::of(value["level"].as_str().or(value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
}

fn block_at(value: &Value, pos: &Value) -> Option<Block> {
    let at = |i: usize| pos[i].as_i64().map(|v| v as i32);
    Some(world_of(value).get_block_at(at(0)?, at(1)?, at(2)?))
}

fn stacks_of(list: &Value) -> Vec<Option<ItemStack>> {
    list.as_array().into_iter().flatten().map(ItemStack::read).collect()
}

fn snbt_of(stack: Option<&ItemStack>) -> Value {
    stack.map_or(Value::Null, |stack| json!(stack.to_snbt()))
}

fn answer_of(cancelled: bool) -> Answer {
    if cancelled { Answer::Fail } else { Answer::Pass }
}

/// Work for the end of the tick, where a listener's game is whole.
static AT_TICK: std::sync::Mutex<Vec<Box<dyn FnOnce() + Send>>> = std::sync::Mutex::new(Vec::new());

fn at_tick(work: impl FnOnce() + Send + 'static) {
    AT_TICK.lock().unwrap_or_else(|e| e.into_inner()).push(Box::new(work));
}

/// The tick's own firing: `None` for an event's.
fn tick_answer(value: &Value) -> Option<Answer> {
    if !value.is_null() {
        return None;
    }
    let work = std::mem::take(&mut *AT_TICK.lock().unwrap_or_else(|e| e.into_inner()));
    for work in work {
        work();
    }
    Some(Answer::Pass)
}

const HELD_HOOKS: &[&str] = &["pl_held", "pl_tables", "pl_stations", "pl_trades"];

macro_rules! held_event {
    ($name:ident, |$value:ident| $wanted:expr) => {
        impl Event for $name {
            const HOOK: &'static str = "pl_packet";
            fn more_hooks() -> &'static [&'static str] {
                HELD_HOOKS
            }
            fn read($value: &Value) -> Option<Self> {
                ($wanted).then(|| Self { value: $value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
            }
            fn answer(&self) -> Answer {
                answer_of(self.cancelled)
            }
        }
    };
}

/// An event over one hook of `family/ev_player.py`, or several.
macro_rules! hooked {
    ($name:ident, $hook:literal $(, [$($more:literal),*])?, |$value:ident| $wanted:expr) => {
        impl Event for $name {
            const HOOK: &'static str = $hook;
            $(fn more_hooks() -> &'static [&'static str] {
                &[$($more),*]
            })?
            fn read($value: &Value) -> Option<Self> {
                ($wanted).then(|| Self { value: $value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
            }
            fn answer(&self) -> Answer {
                answer_of(self.cancelled)
            }
        }
    };
}

// ---- What's open ----

/// A menu as an event saw it. Its `set_item` goes to the real slot at once: a chest's block, the
/// anvil's own slots, the player's rows.
#[derive(Clone, Debug, Default)]
pub struct MenuInventory {
    uuid: String,
    window: i32,
    kind: String,
    size: usize,
    items: Vec<Option<ItemStack>>,
    rows: Vec<Option<ItemStack>>,
    location: Option<Location>,
    gui: Option<Inventory>,
}

impl MenuInventory {
    /// Whether this window is `inventory`, the GUI the plugin made with `Inventory::create` and opened: Java's
    /// `event.getInventory() == this.menu` (or `getHolder() instanceof MyMenu`).
    pub fn is(&self, inventory: &Inventory) -> bool {
        self.gui.as_ref().is_some_and(|gui| gui.same_as(inventory))
    }
    /// The plugin's own GUI this window shows, if it is one.
    pub fn gui(&self) -> Option<&Inventory> {
        self.gui.as_ref()
    }
    fn read(value: &Value) -> Self {
        Self {
            uuid: value["uuid"].as_str().unwrap_or_default().to_owned(),
            window: value["window"].as_i64().unwrap_or(0) as i32,
            kind: value["type"].as_str().unwrap_or("CHEST").to_owned(),
            size: value["size"].as_u64().unwrap_or(0) as usize,
            items: stacks_of(&value["items"]),
            rows: stacks_of(&value["rows"]),
            location: block_at(value, &value["pos"]).map(|block| block.get_location()),
            gui: None,
        }
    }
    /// `InventoryType`'s name.
    pub fn get_type(&self) -> &str {
        &self.kind
    }
    /// Its own slots; for the player's inventory menu, the result and grid.
    pub fn get_size(&self) -> usize {
        self.size
    }
    /// bukkit: org/bukkit/inventory/Inventory.getMaxStackSize
    ///
    /// A container's, as Bukkit gives it (99); an item's own limit applies on top.
    pub fn get_max_stack_size(&self) -> i32 {
        99
    }
    /// The stack in a slot of this menu.
    pub fn get_item(&self, slot: usize) -> Option<ItemStack> {
        match self.live() {
            Some(inventory) => inventory.get_item(slot),
            None => self.items.get(slot).cloned().flatten(),
        }
    }
    pub fn get_contents(&self) -> Vec<Option<ItemStack>> {
        match self.live() {
            Some(inventory) => inventory.get_contents(),
            None => self.items.iter().take(self.size).cloned().collect(),
        }
    }
    /// A block's menu reads the block, as Java's Inventory is live: what an event handed over and is read again
    /// later (CoreProtect compares a chest before a click with the same inventory after it) shows what is there now,
    /// not the event's copy. A plugin's GUI, the player's own and a menu bigger than its block (a double chest) keep
    /// the copy.
    fn live(&self) -> Option<crate::fam_items::BlockInventory> {
        if self.gui.is_some() {
            return None;
        }
        let inventory = crate::fam_items::BlockInventory::of(&self.location.as_ref()?.get_block());
        (inventory.get_size() == self.size && self.size > 0).then_some(inventory)
    }
    /// What the player holds below it, rows first and then the hotbar.
    pub fn get_player_rows(&self) -> &[Option<ItemStack>] {
        &self.rows
    }
    /// Where the block is, if it's a block's.
    pub fn get_location(&self) -> Option<&Location> {
        self.location.as_ref()
    }
    /// The real slot, now. `false` if the menu is gone.
    pub fn set_item(&mut self, slot: usize, stack: Option<&ItemStack>) -> bool {
        if let Some(place) = self.items.get_mut(slot) {
            *place = stack.cloned();
        }
        if let Some(gui) = &mut self.gui {
            gui.set_item(slot, stack.cloned());
            if let Some(player) = Player::read(&query(json!({ "op": "player", "uuid": self.uuid }))) {
                player.update_inventory(gui);
            }
            return true;
        }
        query(json!({ "op": "ev_player_slot_set", "uuid": self.uuid, "window": self.window, "slot": slot, "stack": snbt_of(stack) })).as_bool() == Some(true)
    }
    /// The result slot's number in this kind of menu, if it has one.
    fn result_slot(&self) -> Option<usize> {
        match self.kind.as_str() {
            "CRAFTING" | "WORKBENCH" => Some(0),
            "ANVIL" | "GRINDSTONE" | "CARTOGRAPHY" => Some(2),
            "SMITHING" => Some(3),
            "STONECUTTER" => Some(1),
            "LOOM" => Some(3),
            "MERCHANT" => Some(2),
            _ => None,
        }
    }
    /// bukkit: org/bukkit/inventory/CraftingInventory.getResult
    pub fn get_result(&self) -> Option<ItemStack> {
        self.result_slot().and_then(|slot| self.get_item(slot))
    }
    /// bukkit: org/bukkit/inventory/CraftingInventory.setResult
    pub fn set_result(&mut self, stack: Option<&ItemStack>) -> bool {
        self.result_slot().is_some_and(|slot| self.set_item(slot, stack))
    }
    /// bukkit: org/bukkit/inventory/CraftingInventory.getMatrix
    pub fn get_matrix(&self) -> Vec<Option<ItemStack>> {
        match self.kind.as_str() {
            "CRAFTING" => self.items.iter().skip(1).take(4).cloned().collect(),
            "WORKBENCH" => self.items.iter().skip(1).take(9).cloned().collect(),
            _ => Vec::new(),
        }
    }
}

/// What an event's `getView` gives: the menu and whose it is.
#[derive(Clone, Debug)]
pub struct MenuView {
    player: Option<Player>,
    top: MenuInventory,
}

impl MenuView {
    pub fn get_player(&self) -> Option<&Player> {
        self.player.as_ref()
    }
    pub fn get_top_inventory(&self) -> &MenuInventory {
        &self.top
    }
    pub fn get_top_inventory_mut(&mut self) -> &mut MenuInventory {
        &mut self.top
    }
    pub fn get_type(&self) -> &str {
        self.top.get_type()
    }
    pub fn get_cursor(&self) -> Option<ItemStack> {
        self.player.as_ref()?.get_item_on_cursor()
    }
    /// bukkit: org/bukkit/inventory/InventoryView.getTitle, org/bukkit/inventory/InventoryView.getOriginalTitle
    ///
    /// A plugin's menu: the title it was made with; one of the game's: its type's own.
    pub fn get_title(&self) -> String {
        if let Some(gui) = &self.top.gui {
            return gui.get_title().to_owned();
        }
        let open = self.player.as_ref().and_then(|player| OPEN_INVENTORIES.lock().unwrap_or_else(|e| e.into_inner()).get(player.uuid()).map(|gui| gui.get_title().to_owned()));
        open.unwrap_or_else(|| default_title(self.top.get_type()).to_owned())
    }
}

/// A raw slot's place in Bukkit's numbering, and the slot's type.
fn slot_facts(kind: &str, size: usize, raw: i32) -> (i32, &'static str) {
    if raw < 0 {
        return (raw, "OUTSIDE");
    }
    let raw_size = size as i32;
    if kind == "CRAFTING" {
        return match raw {
            0 => (0, "RESULT"),
            1..=4 => (raw, "CRAFTING"),
            5..=8 => (39 - (raw - 5), "ARMOR"),
            9..=35 => (raw, "CONTAINER"),
            36..=44 => (raw - 36, "QUICKBAR"),
            _ => (40, "QUICKBAR"),
        };
    }
    if raw >= raw_size {
        let row = raw - raw_size;
        return if row < 27 { (row + 9, "CONTAINER") } else { (row - 27, "QUICKBAR") };
    }
    let kind_of_slot = match (kind, raw) {
        ("WORKBENCH", 0) | ("ANVIL", 2) | ("GRINDSTONE", 2) | ("CARTOGRAPHY", 2) | ("SMITHING", 3) | ("STONECUTTER", 1) | ("LOOM", 3) | ("MERCHANT", 2) | ("FURNACE" | "BLAST_FURNACE" | "SMOKER", 2) => "RESULT",
        ("FURNACE" | "BLAST_FURNACE" | "SMOKER", 1) | ("BREWING", 4) => "FUEL",
        ("WORKBENCH", _) | ("ANVIL" | "GRINDSTONE" | "CARTOGRAPHY" | "SMITHING" | "STONECUTTER" | "LOOM" | "MERCHANT" | "FURNACE" | "BLAST_FURNACE" | "SMOKER" | "BREWING" | "ENCHANTING" | "BEACON", _) => "CRAFTING",
        _ => "CONTAINER",
    };
    (raw, kind_of_slot)
}

/// bukkit: org/bukkit/event/inventory/ClickType.isKeyboardClick
pub fn is_keyboard_click(click: &str) -> bool {
    matches!(click, "NUMBER_KEY" | "DROP" | "CONTROL_DROP" | "SWAP_OFFHAND")
}
/// bukkit: org/bukkit/event/inventory/ClickType.isMouseClick
pub fn is_mouse_click(click: &str) -> bool {
    matches!(click, "DOUBLE_CLICK" | "LEFT" | "RIGHT" | "MIDDLE" | "WINDOW_BORDER_LEFT" | "SHIFT_LEFT" | "SHIFT_RIGHT" | "WINDOW_BORDER_RIGHT")
}
/// bukkit: org/bukkit/event/inventory/ClickType.isCreativeAction
pub fn is_creative_action(click: &str) -> bool {
    matches!(click, "MIDDLE" | "CREATIVE")
}
/// bukkit: org/bukkit/event/inventory/ClickType.isRightClick
pub fn is_right_click(click: &str) -> bool {
    matches!(click, "RIGHT" | "SHIFT_RIGHT")
}
/// bukkit: org/bukkit/event/inventory/ClickType.isLeftClick
pub fn is_left_click(click: &str) -> bool {
    matches!(click, "LEFT" | "SHIFT_LEFT" | "DOUBLE_CLICK" | "CREATIVE")
}
/// bukkit: org/bukkit/event/inventory/ClickType.isShiftClick
pub fn is_shift_click(click: &str) -> bool {
    matches!(click, "SHIFT_LEFT" | "SHIFT_RIGHT")
}

/// bukkit: org/bukkit/event/inventory/InventoryType.getDefaultSize
pub fn default_size(kind: &str) -> usize {
    match kind {
        "CHEST" | "ENDER_CHEST" | "SHULKER_BOX" | "BARREL" => 27,
        "DISPENSER" | "DROPPER" | "CRAFTER" => 9,
        "FURNACE" | "BLAST_FURNACE" | "SMOKER" | "ANVIL" | "GRINDSTONE" | "CARTOGRAPHY" | "MERCHANT" => 3,
        "WORKBENCH" => 10,
        "CRAFTING" | "HOPPER" | "BREWING" => 5,
        "ENCHANTING" | "STONECUTTER" => 2,
        "BEACON" | "LECTERN" | "COMPOSTER" | "JUKEBOX" | "DECORATED_POT" => 1,
        "PLAYER" => 41,
        "CREATIVE" => 9,
        "SMITHING" | "LOOM" => 4,
        "CHISELED_BOOKSHELF" => 6,
        "SHELF" => 3,
        _ => 0,
    }
}
/// bukkit: org/bukkit/event/inventory/InventoryType.isCreatable
pub fn is_creatable(kind: &str) -> bool {
    !matches!(kind, "CREATIVE" | "CRAFTING" | "MERCHANT")
}
/// bukkit: org/bukkit/event/inventory/InventoryType.getDefaultTitle, org/bukkit/event/inventory/InventoryType.defaultTitle
pub fn default_title(kind: &str) -> &'static str {
    match kind {
        "CHEST" => "Chest",
        "DISPENSER" => "Dispenser",
        "DROPPER" => "Dropper",
        "FURNACE" => "Furnace",
        "WORKBENCH" | "CRAFTING" => "Crafting",
        "ENCHANTING" => "Enchanting",
        "BREWING" => "Brewing",
        "PLAYER" | "CREATIVE" => "Player",
        "MERCHANT" => "Villager",
        "ENDER_CHEST" => "Ender Chest",
        "ANVIL" => "Repairing",
        "SMITHING" | "SMITHING_NEW" => "Upgrade Gear",
        "BEACON" => "container.beacon",
        "HOPPER" => "Item Hopper",
        "SHULKER_BOX" => "Shulker Box",
        "BARREL" => "Barrel",
        "BLAST_FURNACE" => "Blast Furnace",
        "LECTERN" => "Lectern",
        "SMOKER" => "Smoker",
        "LOOM" => "Loom",
        "CARTOGRAPHY" => "Cartography Table",
        "GRINDSTONE" => "Repair & Disenchant",
        "STONECUTTER" => "Stonecutter",
        "COMPOSTER" => "Composter",
        "CHISELED_BOOKSHELF" => "Chiseled Bookshelf",
        "SHELF" => "Shelf",
        "JUKEBOX" => "Jukebox",
        "DECORATED_POT" => "Decorated Pot",
        "CRAFTER" => "Crafter",
        _ => "",
    }
}

// ---- Clicks ----

/// `getClick`: mode and button of `ServerboundContainerClickPacket`.
fn click_type(input: i64, button: i64, slot: i64) -> &'static str {
    match (input, button) {
        (0, 0) if slot == -999 => "WINDOW_BORDER_LEFT",
        (0, _) if slot == -999 => "WINDOW_BORDER_RIGHT",
        (0, 0) => "LEFT",
        (0, _) => "RIGHT",
        (1, 0) => "SHIFT_LEFT",
        (1, _) => "SHIFT_RIGHT",
        (2, 40) => "SWAP_OFFHAND",
        (2, _) => "NUMBER_KEY",
        (3, _) => "MIDDLE",
        (4, 0) => "DROP",
        (4, _) => "CONTROL_DROP",
        (6, _) => "DOUBLE_CLICK",
        _ => "UNKNOWN",
    }
}

/// `getAction`: what the click will do, from the slot and the cursor.
fn click_action(click: &str, kind: &str, size: usize, slot: i64, current: Option<&ItemStack>, cursor: Option<&ItemStack>, max: i64, creative: bool) -> &'static str {
    let result = slot_facts(kind, size, slot as i32).1 == "RESULT";
    match click {
        "WINDOW_BORDER_LEFT" if cursor.is_some() => "DROP_ALL_CURSOR",
        "WINDOW_BORDER_RIGHT" if cursor.is_some() => "DROP_ONE_CURSOR",
        "LEFT" | "RIGHT" => match (current, cursor) {
            (None, None) => "NOTHING",
            (None, Some(_)) if click == "LEFT" => "PLACE_ALL",
            (None, Some(_)) => "PLACE_ONE",
            (Some(_), None) if click == "LEFT" => "PICKUP_ALL",
            (Some(_), None) => "PICKUP_HALF",
            (Some(current), Some(cursor)) if result => {
                if current.is_similar(cursor) && i64::from(current.get_amount()) + i64::from(cursor.get_amount()) <= max { "PICKUP_ALL" } else { "NOTHING" }
            }
            (Some(current), Some(cursor)) if current.is_similar(cursor) => {
                let room = max - i64::from(current.get_amount());
                match click {
                    "RIGHT" if room > 0 => "PLACE_ONE",
                    "LEFT" if room >= i64::from(cursor.get_amount()) => "PLACE_ALL",
                    "LEFT" if room > 0 => "PLACE_SOME",
                    _ => "NOTHING",
                }
            }
            _ => "SWAP_WITH_CURSOR",
        },
        "SHIFT_LEFT" | "SHIFT_RIGHT" if current.is_some() => "MOVE_TO_OTHER_INVENTORY",
        "NUMBER_KEY" | "SWAP_OFFHAND" => "HOTBAR_SWAP",
        "MIDDLE" if creative && current.is_some() && cursor.is_none() => "CLONE_STACK",
        "DROP" if current.is_some() => "DROP_ONE_SLOT",
        "CONTROL_DROP" if current.is_some() => "DROP_ALL_SLOT",
        "DOUBLE_CLICK" if cursor.is_some() => "COLLECT_TO_CURSOR",
        _ => "NOTHING",
    }
}

/// bukkit: org/bukkit/event/inventory/InventoryClickEvent, org/bukkit/event/inventory/InventoryInteractEvent, org/bukkit/event/inventory/InventoryEvent
///
/// A player's click in a chest, a furnace, an anvil or another workstation, a villager's trade, their
/// own inventory, a GUI from `Bukkit.createInventory`; not yet a mount's inventory or a lectern. A cancel
/// leaves everything as it was, and the client is told so. `CraftItemEvent` and the like fire in its
/// place for the clicks that are theirs, as in Bukkit.
#[derive(Clone, Debug, Default)]
pub struct InventoryClickEvent {
    value: Value,
    cancelled: bool,
    current: Option<ItemStack>,
    inventory: MenuInventory,
}

/// The fields every click event reads, from a held click or a GUI's.
fn read_click(value: &Value) -> Option<(Value, MenuInventory, Option<ItemStack>)> {
    if value["kind"] == "click" {
        return Some((value.clone(), MenuInventory::read(value), ItemStack::read(&value["current"])));
    }
    // A GUI from `Bukkit.createInventory`: its stacks are the plugin's own.
    if value.get("mode").is_none() {
        return None;
    }
    let player = player_of(value)?;
    let gui = OPEN_INVENTORIES.lock().unwrap_or_else(|e| e.into_inner()).get(player.uuid()).cloned()?;
    let size = gui.get_size();
    let items = (0..size).map(|slot| gui.get_item(slot).cloned()).collect::<Vec<_>>();
    let slot = value["slot"].as_i64().unwrap_or(-1);
    let current = usize::try_from(slot).ok().and_then(|slot| items.get(slot).cloned().flatten());
    let mut full = value.clone();
    full["input"] = value["mode"].clone();
    full["name"] = json!(player.get_name());
    full["type"] = json!("CHEST");
    full["size"] = json!(size);
    let mut inventory = MenuInventory::read(&full);
    inventory.items = items;
    inventory.gui = Some(gui);
    Some((full, inventory, current))
}

impl Event for InventoryClickEvent {
    const HOOK: &'static str = "pl_packet";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_held", "pl_tables", "pl_stations", "pl_trades", "menu_click"]
    }
    fn read(value: &Value) -> Option<Self> {
        if own_click(value) {
            return None;
        }
        let (value, inventory, current) = read_click(value)?;
        Some(Self { value, cancelled: crate::event::prior_cancelled(), current, inventory })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

/// A click that's another event's in Bukkit.
fn own_click(value: &Value) -> bool {
    value["kind"] == "click" && (craft_click(value) || smith_click(value) || cartography_click(value))
}

/// A click on the result of a crafting grid.
fn craft_click(value: &Value) -> bool {
    value["kind"] == "click" && value["slot"] == 0 && (value["type"] == "WORKBENCH" || value["type"] == "CRAFTING") && !value["current"].is_null()
}

fn smith_click(value: &Value) -> bool {
    value["kind"] == "click" && value["slot"] == 3 && value["type"] == "SMITHING" && !value["current"].is_null()
}

fn cartography_click(value: &Value) -> bool {
    value["kind"] == "click" && value["slot"] == 2 && value["type"] == "CARTOGRAPHY" && !value["current"].is_null()
}

macro_rules! click_methods {
    ($name:ident) => {
        impl $name {
            fn whose(&self) -> Option<Player> {
                player_of(&self.value)
            }
            fn number(&self, key: &str) -> i64 {
                self.value[key].as_i64().unwrap_or(0)
            }
            fn raw_click(&self) -> &'static str {
                click_type(self.number("input"), self.number("button"), self.number("slot"))
            }
            fn write_slot(&mut self, raw: i64, stack: Option<&ItemStack>) {
                if let Some(gui) = &mut self.inventory.gui {
                    if let Ok(slot) = usize::try_from(raw) {
                        gui.set_item(slot, stack.cloned());
                    }
                    if let Some(player) = player_of(&self.value) {
                        player.update_inventory(gui);
                    }
                    return;
                }
                query(json!({ "op": "ev_player_slot_set", "uuid": self.value["uuid"], "window": self.value["window"], "slot": raw, "stack": snbt_of(stack) }));
            }
            fn set_the_cursor(&mut self, stack: Option<&ItemStack>) {
                if self.inventory.gui.is_some() {
                    return;
                }
                self.write_slot(-1, stack);
                self.value["cursor"] = stack.map_or(Value::Null, ItemStack::to_json);
            }
            fn the_action(&self) -> &'static str {
                let cursor = ItemStack::read(&self.value["cursor"]);
                let creative = self.whose().is_some_and(|player| player.get_game_mode() == "CREATIVE");
                click_action(self.raw_click(), self.value["type"].as_str().unwrap_or_default(), self.number("size") as usize, self.number("slot"), self.current.as_ref(), cursor.as_ref(), self.number("max").max(1), creative)
            }
        }
    };
}

click_methods!(InventoryClickEvent);

impl InventoryClickEvent {
    pub fn get_who_clicked(&self) -> Option<Player> {
        self.whose()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getInventory
    pub fn get_inventory(&self) -> &MenuInventory {
        &self.inventory
    }
    /// bukkit: org/bukkit/event/inventory/InventoryInteractEvent.getWhoClicked
    pub fn who_clicked(&self) -> Option<Player> {
        self.whose()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.getClickedInventory
    ///
    /// The menu's own inventory, or `None` for a click in the player's rows or outside.
    pub fn get_clicked_inventory(&self) -> Option<&MenuInventory> {
        usize::try_from(self.number("slot")).ok().filter(|slot| *slot < self.inventory.size).map(|_| &self.inventory)
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: self.whose(), top: self.inventory.clone() }
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getViewers
    pub fn get_viewers(&self) -> Vec<Player> {
        self.whose().into_iter().collect()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.getRawSlot
    pub fn get_raw_slot(&self) -> i32 {
        self.number("slot") as i32
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.getSlot
    pub fn get_slot(&self) -> i32 {
        slot_facts(self.inventory.get_type(), self.inventory.size, self.get_raw_slot()).0
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.getSlotType
    pub fn get_slot_type(&self) -> &'static str {
        slot_facts(self.inventory.get_type(), self.inventory.size, self.get_raw_slot()).1
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.getCurrentItem
    pub fn get_current_item(&self) -> Option<&ItemStack> {
        self.current.as_ref()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.setCurrentItem
    pub fn set_current_item(&mut self, stack: Option<&ItemStack>) {
        self.write_slot(self.number("slot"), stack);
        self.current = stack.cloned();
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.getCursor
    pub fn get_cursor(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["cursor"])
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.setCursor
    pub fn set_cursor(&mut self, stack: Option<&ItemStack>) {
        self.set_the_cursor(stack);
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.getClick
    pub fn get_click(&self) -> &'static str {
        self.raw_click()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.getAction
    pub fn get_action(&self) -> &'static str {
        self.the_action()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.getHotbarButton
    pub fn get_hotbar_button(&self) -> i32 {
        if self.number("input") == 2 { self.number("button") as i32 } else { -1 }
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.isLeftClick
    pub fn is_left_click(&self) -> bool {
        is_left_click(self.raw_click())
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.isRightClick
    pub fn is_right_click(&self) -> bool {
        is_right_click(self.raw_click())
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.isShiftClick
    pub fn is_shift_click(&self) -> bool {
        is_shift_click(self.raw_click())
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    /// `Event.Result`: `DENY` cancels.
    pub fn get_result(&self) -> &'static str {
        if self.cancelled { "DENY" } else { "DEFAULT" }
    }
    pub fn set_result(&mut self, result: &str) {
        self.cancelled = result == "DENY";
    }
}

/// bukkit: org/bukkit/event/inventory/CraftItemEvent
///
/// A click on the result of a crafting table or the player's grid, before anything is taken.
#[derive(Clone, Debug, Default)]
pub struct CraftItemEvent {
    value: Value,
    cancelled: bool,
    current: Option<ItemStack>,
    inventory: MenuInventory,
}

/// bukkit: org/bukkit/event/inventory/SmithItemEvent
#[derive(Clone, Debug, Default)]
pub struct SmithItemEvent {
    value: Value,
    cancelled: bool,
    current: Option<ItemStack>,
    inventory: MenuInventory,
}

/// bukkit: io/papermc/paper/event/player/CartographyItemEvent
#[derive(Clone, Debug, Default)]
pub struct CartographyItemEvent {
    value: Value,
    cancelled: bool,
    current: Option<ItemStack>,
    inventory: MenuInventory,
}

macro_rules! result_click {
    ($name:ident, $check:ident) => {
        click_methods!($name);
        impl Event for $name {
            const HOOK: &'static str = "pl_packet";
            fn more_hooks() -> &'static [&'static str] {
                HELD_HOOKS
            }
            fn read(value: &Value) -> Option<Self> {
                if !$check(value) {
                    return None;
                }
                let (value, inventory, current) = read_click(value)?;
                Some(Self { value, cancelled: crate::event::prior_cancelled(), current, inventory })
            }
            fn answer(&self) -> Answer {
                answer_of(self.cancelled)
            }
        }
    };
}

result_click!(CraftItemEvent, craft_click);
result_click!(SmithItemEvent, smith_click);
result_click!(CartographyItemEvent, cartography_click);

impl CraftItemEvent {
    pub fn get_who_clicked(&self) -> Option<Player> {
        self.whose()
    }
    /// bukkit: org/bukkit/event/inventory/CraftItemEvent.getInventory
    pub fn get_inventory(&self) -> &MenuInventory {
        &self.inventory
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: self.whose(), top: self.inventory.clone() }
    }
    pub fn get_current_item(&self) -> Option<&ItemStack> {
        self.current.as_ref()
    }
    pub fn set_current_item(&mut self, stack: Option<&ItemStack>) {
        self.write_slot(0, stack);
        self.current = stack.cloned();
    }
    pub fn get_cursor(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["cursor"])
    }
    pub fn set_cursor(&mut self, stack: Option<&ItemStack>) {
        self.set_the_cursor(stack);
    }
    pub fn get_click(&self) -> &'static str {
        self.raw_click()
    }
    pub fn get_action(&self) -> &'static str {
        self.the_action()
    }
    pub fn is_shift_click(&self) -> bool {
        is_shift_click(self.raw_click())
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.getSlotType
    ///
    /// A craft is a click on the result slot (Bukkit's CraftItemEvent is an InventoryClickEvent there).
    pub fn get_slot_type(&self) -> &'static str {
        "RESULT"
    }
    /// bukkit: org/bukkit/event/inventory/InventoryInteractEvent.getResult
    ///
    /// `DENY` once cancelled, else `DEFAULT` (CoreProtect skips a denied craft by it).
    pub fn get_result(&self) -> &'static str {
        if self.cancelled { "DENY" } else { "DEFAULT" }
    }
    /// bukkit: org/bukkit/event/inventory/InventoryClickEvent.getHotbarButton
    ///
    /// The number key pressed for a `NUMBER_KEY` click, else -1.
    pub fn get_hotbar_button(&self) -> i32 {
        if self.raw_click() == "NUMBER_KEY" { self.value["button"].as_i64().unwrap_or(-1) as i32 } else { -1 }
    }
    /// bukkit: org/bukkit/event/inventory/CraftItemEvent.getRecipe
    pub fn get_recipe(&self) -> Option<crate::core2::GameRecipe> {
        crate::core2::get_recipe(&NamespacedKey::from_string(self.value["recipe"].as_str()?)?)
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl SmithItemEvent {
    pub fn get_who_clicked(&self) -> Option<Player> {
        self.whose()
    }
    /// bukkit: org/bukkit/event/inventory/SmithItemEvent.getInventory
    pub fn get_inventory(&self) -> &MenuInventory {
        &self.inventory
    }
    pub fn get_current_item(&self) -> Option<&ItemStack> {
        self.current.as_ref()
    }
    pub fn get_cursor(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["cursor"])
    }
    pub fn get_click(&self) -> &'static str {
        self.raw_click()
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl CartographyItemEvent {
    pub fn get_who_clicked(&self) -> Option<Player> {
        self.whose()
    }
    /// bukkit: io/papermc/paper/event/player/CartographyItemEvent.getInventory
    pub fn get_inventory(&self) -> &MenuInventory {
        &self.inventory
    }
    pub fn get_current_item(&self) -> Option<&ItemStack> {
        self.current.as_ref()
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/inventory/InventoryDragEvent
///
/// A drag over two or more slots of any menu, as it ends. A drag of one slot is a click in the game,
/// and in Bukkit too.
#[derive(Clone, Debug, Default)]
pub struct InventoryDragEvent {
    value: Value,
    cancelled: bool,
    cursor: Option<Option<ItemStack>>,
}

/// The slots of a drag in a plugin's own menu still going, by player.
static GUI_DRAGS: std::sync::Mutex<BTreeMap<String, Vec<i32>>> = std::sync::Mutex::new(BTreeMap::new());

impl Event for InventoryDragEvent {
    const HOOK: &'static str = "pl_packet";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_held", "pl_tables", "pl_stations", "pl_trades", "menu_click"]
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "drag").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
    }
    fn read_all(value: &Value) -> Vec<Self> {
        if value.get("kind").is_some() {
            return Self::read(value).into_iter().collect();
        }
        // A plugin's menu: ServerboundContainerClickPacket's mode 5, the button (kind << 2) | stage.
        let (Some(5), Some(button), Some(player)) = (value["mode"].as_i64(), value["button"].as_i64(), player_of(value)) else {
            return Vec::new();
        };
        let Some(gui) = OPEN_INVENTORIES.lock().unwrap_or_else(|e| e.into_inner()).get(player.uuid()).cloned() else {
            return Vec::new();
        };
        let mut drags = GUI_DRAGS.lock().unwrap_or_else(|e| e.into_inner());
        let slots = drags.entry(player.uuid().to_owned()).or_default();
        match button & 3 {
            0 => slots.clear(),
            1 => slots.extend(value["slot"].as_i64().map(|slot| slot as i32)),
            _ => {
                let mut full = value.clone();
                full["kind"] = json!("drag");
                full["name"] = json!(player.get_name());
                full["slots"] = json!(std::mem::take(slots));
                full["drag"] = json!(button >> 2);
                full["type"] = json!("CHEST");
                full["size"] = json!(gui.get_size());
                return vec![Self { value: full, cancelled: crate::event::prior_cancelled(), ..Default::default() }];
            }
        }
        Vec::new()
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

impl InventoryDragEvent {
    pub fn get_who_clicked(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getInventory
    pub fn get_inventory(&self) -> MenuInventory {
        MenuInventory::read(&self.value)
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: player_of(&self.value), top: MenuInventory::read(&self.value) }
    }
    /// bukkit: org/bukkit/event/inventory/InventoryDragEvent.getRawSlots
    pub fn get_raw_slots(&self) -> Vec<i32> {
        self.value["slots"].as_array().into_iter().flatten().filter_map(Value::as_i64).map(|slot| slot as i32).collect()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryDragEvent.getInventorySlots
    ///
    /// The slots in the numbering of the inventory each is in.
    pub fn get_inventory_slots(&self) -> Vec<i32> {
        let (kind, size) = (self.value["type"].as_str().unwrap_or_default(), self.value["size"].as_u64().unwrap_or(0) as usize);
        self.get_raw_slots().into_iter().map(|raw| slot_facts(kind, size, raw).0).collect()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryDragEvent.getNewItems
    ///
    /// What each raw slot will hold: `Map<Integer, ItemStack>`.
    pub fn get_new_items(&self) -> BTreeMap<i32, ItemStack> {
        self.value["placed"].as_array().into_iter().flatten().filter_map(|row| Some((row[0].as_i64()? as i32, ItemStack::read(&row[1])?))).collect()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryDragEvent.getOldCursor
    ///
    /// The stack on the cursor as the drag began.
    pub fn get_old_cursor(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["cursor"])
    }
    /// bukkit: org/bukkit/event/inventory/InventoryDragEvent.getCursor
    ///
    /// The stack the cursor will be left with.
    pub fn get_cursor(&self) -> Option<ItemStack> {
        match &self.cursor {
            Some(stack) => stack.clone(),
            None => ItemStack::read(&self.value["cursor_after"]),
        }
    }
    /// bukkit: org/bukkit/event/inventory/InventoryDragEvent.setCursor
    pub fn set_cursor(&mut self, stack: Option<&ItemStack>) {
        self.cursor = Some(stack.cloned());
        query(json!({ "op": "ev_player_slot_set", "uuid": self.value["uuid"], "window": self.value["window"], "slot": -1, "stack": snbt_of(stack), "after": true }));
    }
    /// bukkit: org/bukkit/event/inventory/InventoryDragEvent.getType
    ///
    /// `SINGLE` for a right-button drag, else `EVEN`.
    pub fn get_type(&self) -> &'static str {
        if self.value["drag"] == 1 { "SINGLE" } else { "EVEN" }
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    pub fn get_result(&self) -> &'static str {
        if self.cancelled { "DENY" } else { "DEFAULT" }
    }
    pub fn set_result(&mut self, result: &str) {
        self.cancelled = result == "DENY";
    }
}

// ---- A menu's result, as its inputs change ----

macro_rules! prepare_event {
    ($name:ident, |$value:ident| $wanted:expr) => {
        held_event!($name, |$value| $value["kind"] == "prepare" && $wanted);
        impl $name {
            fn top(&self) -> MenuInventory {
                MenuInventory::read(&self.value)
            }
        }
    };
}

/// bukkit: org/bukkit/event/inventory/PrepareAnvilEvent
///
/// When an anvil's inputs or the name change.
#[derive(Clone, Debug, Default)]
pub struct PrepareAnvilEvent {
    value: Value,
    cancelled: bool,
}

prepare_event!(PrepareAnvilEvent, |value| value["type"] == "ANVIL");

impl PrepareAnvilEvent {
    /// bukkit: org/bukkit/event/inventory/PrepareAnvilEvent.getInventory
    pub fn get_inventory(&self) -> MenuInventory {
        self.top()
    }
    /// bukkit: org/bukkit/event/inventory/PrepareAnvilEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: player_of(&self.value), top: self.top() }
    }
    pub fn get_result(&self) -> Option<ItemStack> {
        self.top().get_result()
    }
    /// bukkit: org/bukkit/event/inventory/PrepareAnvilEvent.setResult
    pub fn set_result(&mut self, stack: Option<&ItemStack>) {
        self.top().set_result(stack);
    }
    /// What the player typed.
    pub fn get_rename_text(&self) -> Option<&str> {
        self.value["rename"].as_str()
    }
}

/// bukkit: org/bukkit/event/inventory/PrepareGrindstoneEvent, com/destroystokyo/paper/event/inventory/PrepareGrindstoneEvent
#[derive(Clone, Debug, Default)]
pub struct PrepareGrindstoneEvent {
    value: Value,
    cancelled: bool,
}

prepare_event!(PrepareGrindstoneEvent, |value| value["type"] == "GRINDSTONE");

impl PrepareGrindstoneEvent {
    /// bukkit: com/destroystokyo/paper/event/inventory/PrepareGrindstoneEvent.getInventory, org/bukkit/event/inventory/PrepareGrindstoneEvent.getInventory
    pub fn get_inventory(&self) -> MenuInventory {
        self.top()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: player_of(&self.value), top: self.top() }
    }
    pub fn get_result(&self) -> Option<ItemStack> {
        self.top().get_result()
    }
    pub fn set_result(&mut self, stack: Option<&ItemStack>) {
        self.top().set_result(stack);
    }
}

/// bukkit: org/bukkit/event/inventory/PrepareSmithingEvent
#[derive(Clone, Debug, Default)]
pub struct PrepareSmithingEvent {
    value: Value,
    cancelled: bool,
}

prepare_event!(PrepareSmithingEvent, |value| value["type"] == "SMITHING");

impl PrepareSmithingEvent {
    /// bukkit: org/bukkit/event/inventory/PrepareSmithingEvent.getInventory
    pub fn get_inventory(&self) -> MenuInventory {
        self.top()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: player_of(&self.value), top: self.top() }
    }
    pub fn get_result(&self) -> Option<ItemStack> {
        self.top().get_result()
    }
    pub fn set_result(&mut self, stack: Option<&ItemStack>) {
        self.top().set_result(stack);
    }
}

/// bukkit: org/bukkit/event/inventory/PrepareInventoryResultEvent, com/destroystokyo/paper/event/inventory/PrepareResultEvent
///
/// The anvil's, the grindstone's and the smithing table's, as in Bukkit.
#[derive(Clone, Debug, Default)]
pub struct PrepareInventoryResultEvent {
    value: Value,
    cancelled: bool,
}

prepare_event!(PrepareInventoryResultEvent, |value| matches!(value["type"].as_str(), Some("ANVIL" | "GRINDSTONE" | "SMITHING")));

impl PrepareInventoryResultEvent {
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getInventory
    pub fn get_inventory(&self) -> MenuInventory {
        self.top()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: player_of(&self.value), top: self.top() }
    }
    /// bukkit: org/bukkit/event/inventory/PrepareInventoryResultEvent.getResult
    pub fn get_result(&self) -> Option<ItemStack> {
        self.top().get_result()
    }
    /// bukkit: org/bukkit/event/inventory/PrepareInventoryResultEvent.setResult
    pub fn set_result(&mut self, stack: Option<&ItemStack>) {
        self.top().set_result(stack);
    }
}

/// bukkit: org/bukkit/event/inventory/PrepareItemCraftEvent
///
/// When a crafting grid changes: a table's, or the player's own 2x2.
#[derive(Clone, Debug, Default)]
pub struct PrepareItemCraftEvent {
    value: Value,
    cancelled: bool,
}

prepare_event!(PrepareItemCraftEvent, |value| matches!(value["type"].as_str(), Some("WORKBENCH" | "CRAFTING")));

impl PrepareItemCraftEvent {
    /// bukkit: org/bukkit/event/inventory/PrepareItemCraftEvent.getInventory
    pub fn get_inventory(&self) -> MenuInventory {
        self.top()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: player_of(&self.value), top: self.top() }
    }
    /// bukkit: org/bukkit/event/inventory/PrepareItemCraftEvent.getRecipe
    ///
    /// The recipe the grid makes, as the game's data has it.
    pub fn get_recipe(&self) -> Option<crate::core2::GameRecipe> {
        crate::core2::get_recipe(&NamespacedKey::from_string(self.value["recipe"].as_str()?)?)
    }
    /// bukkit: org/bukkit/event/inventory/PrepareItemCraftEvent.isRepair
    ///
    /// The grid holds two of one damaged tool: the game's own repair recipe.
    pub fn is_repair(&self) -> bool {
        self.value["recipe"] == "minecraft:repair_item"
    }
}

// ---- Enchanting ----

/// bukkit: org/bukkit/event/enchantment/PrepareItemEnchantEvent
///
/// When an enchanting table's item changes. Bukkit's offers are `Option<(enchantment, level, cost)>` here.
#[derive(Clone, Debug, Default)]
pub struct PrepareItemEnchantEvent {
    value: Value,
    cancelled: bool,
}

prepare_event!(PrepareItemEnchantEvent, |value| value["type"] == "ENCHANTING" && !value["items"][0].is_null());

impl PrepareItemEnchantEvent {
    /// bukkit: org/bukkit/event/enchantment/PrepareItemEnchantEvent.getEnchanter
    pub fn get_enchanter(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/enchantment/PrepareItemEnchantEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: player_of(&self.value), top: self.top() }
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getInventory
    pub fn get_inventory(&self) -> MenuInventory {
        self.top()
    }
    /// bukkit: org/bukkit/event/enchantment/PrepareItemEnchantEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["items"][0])
    }
    /// bukkit: org/bukkit/event/enchantment/PrepareItemEnchantEvent.getEnchantBlock
    pub fn get_enchant_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/enchantment/PrepareItemEnchantEvent.getEnchantmentBonus
    ///
    /// The bookshelves that count, up to 15.
    pub fn get_enchantment_bonus(&self) -> i32 {
        self.value["bookshelves"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/enchantment/PrepareItemEnchantEvent.getExpLevelCostsOffered
    pub fn get_exp_level_costs_offered(&self) -> Vec<i32> {
        self.value["costs"].as_array().into_iter().flatten().filter_map(Value::as_i64).map(|cost| cost as i32).collect()
    }
    /// The three offers: enchantment, level and cost.
    pub fn get_offers(&self) -> Vec<Option<(String, i32, i32)>> {
        let costs = self.get_exp_level_costs_offered();
        (0..3)
            .map(|i| {
                let clue = &self.value["clues"][i];
                Some((clue[0].as_str()?.to_owned(), clue[1].as_i64()? as i32, *costs.get(i)?))
            })
            .collect()
    }
    /// bukkit: org/bukkit/event/enchantment/PrepareItemEnchantEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/enchantment/PrepareItemEnchantEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/enchantment/EnchantItemEvent
///
/// A click on an offer the player can pay for, before anything is paid.
#[derive(Clone, Debug, Default)]
pub struct EnchantItemEvent {
    value: Value,
    cancelled: bool,
    item: Option<ItemStack>,
    enchants: Option<BTreeMap<String, i32>>,
}

held_event!(EnchantItemEvent, |value| value["kind"] == "button" && value["type"] == "ENCHANTING");

impl EnchantItemEvent {
    fn top(&self) -> MenuInventory {
        let mut value = self.value.clone();
        value["type"] = json!("ENCHANTING");
        value["size"] = json!(2);
        value["items"] = json!([self.value["item"]]);
        MenuInventory::read(&value)
    }
    /// bukkit: org/bukkit/event/enchantment/EnchantItemEvent.getEnchanter
    pub fn get_enchanter(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: player_of(&self.value), top: self.top() }
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getInventory
    pub fn get_inventory(&self) -> MenuInventory {
        self.top()
    }
    /// bukkit: org/bukkit/event/enchantment/EnchantItemEvent.getEnchantBlock
    pub fn get_enchant_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/enchantment/EnchantItemEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        self.item.clone().or_else(|| ItemStack::read(&self.value["item"]))
    }
    /// bukkit: org/bukkit/event/enchantment/EnchantItemEvent.setItem
    ///
    /// The item that gets the enchantments, in place of the one on the table.
    pub fn set_item(&mut self, stack: &ItemStack) {
        self.item = Some(stack.clone());
        self.result_changed();
    }
    /// bukkit: org/bukkit/event/enchantment/EnchantItemEvent.getExpLevelCost
    pub fn get_exp_level_cost(&self) -> i32 {
        self.value["cost"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/enchantment/EnchantItemEvent.setExpLevelCost
    pub fn set_exp_level_cost(&mut self, cost: i32) {
        self.value["cost"] = json!(cost);
        let creative = player_of(&self.value).is_some_and(|player| player.get_game_mode() == "CREATIVE");
        if !creative && i64::from(cost) > self.value["levels"].as_i64().unwrap_or(0) {
            query(json!({ "op": "ev_player_refuse" }));
        }
    }
    /// bukkit: org/bukkit/event/enchantment/EnchantItemEvent.whichButton
    pub fn which_button(&self) -> i32 {
        self.value["button"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/enchantment/EnchantItemEvent.getEnchantmentHint
    pub fn get_enchantment_hint(&self) -> Option<&str> {
        self.value["hint"]["enchantment"].as_str()
    }
    /// bukkit: org/bukkit/event/enchantment/EnchantItemEvent.getLevelHint
    pub fn get_level_hint(&self) -> i32 {
        self.value["hint"]["level"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/enchantment/EnchantItemEvent.getEnchantsToAdd
    ///
    /// What the item will get, by enchantment. Bukkit's is a live map: put a changed one back with `set_enchants_to_add`.
    pub fn get_enchants_to_add(&self) -> BTreeMap<String, i32> {
        if let Some(enchants) = &self.enchants {
            return enchants.clone();
        }
        let done = &self.value["done"]["components"];
        let list = done.get("minecraft:enchantments").or(done.get("minecraft:stored_enchantments"));
        let levels = list.and_then(|list| list.get("levels").or(Some(list)));
        levels.and_then(Value::as_object).into_iter().flatten().filter_map(|(id, level)| Some((id.clone(), level.as_i64()? as i32))).collect()
    }
    pub fn set_enchants_to_add(&mut self, enchants: &BTreeMap<String, i32>) {
        self.enchants = Some(enchants.clone());
        self.result_changed();
    }
    /// What the game would have made, with this event's changes in it, once it has made it.
    fn result_changed(&self) {
        let mut done = self.item.clone().or_else(|| ItemStack::read(&self.value["done"]));
        if let Some(done) = &mut done {
            let levels: serde_json::Map<String, Value> = self.get_enchants_to_add().into_iter().map(|(id, level)| (id, json!(level))).collect();
            done.set_component("minecraft:enchantments", Value::Object(levels));
        }
        query(json!({ "op": "ev_player_slot_set", "uuid": self.value["uuid"], "window": self.value["window"], "slot": 0, "stack": snbt_of(done.as_ref()), "after": true }));
    }
    /// bukkit: org/bukkit/event/enchantment/EnchantItemEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/enchantment/EnchantItemEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerStonecutterRecipeSelectEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerStonecutterRecipeSelectEvent {
    value: Value,
    cancelled: bool,
}

held_event!(PlayerStonecutterRecipeSelectEvent, |value| value["kind"] == "button" && value["type"] == "STONECUTTER");

impl PlayerStonecutterRecipeSelectEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerStonecutterRecipeSelectEvent.getStonecutterInventory
    pub fn get_stonecutter_inventory(&self) -> MenuInventory {
        let mut value = self.value.clone();
        value["size"] = json!(2);
        value["items"] = json!([self.value["input"]]);
        MenuInventory::read(&value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerStonecutterRecipeSelectEvent.getStonecuttingRecipe
    pub fn get_stonecutting_recipe(&self) -> Option<crate::core2::GameRecipe> {
        crate::core2::get_recipe(&NamespacedKey::from_string(self.value["recipe"].as_str()?)?)
    }
    /// The button: the recipe's place in the list.
    pub fn get_button(&self) -> i32 {
        self.value["button"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: io/papermc/paper/event/player/PlayerStonecutterRecipeSelectEvent.setStonecuttingRecipe
    ///
    /// By its key. One the stonecutter isn\'t offering is refused with `false`, and the click stays what it was, as in Paper.
    pub fn set_stonecutting_recipe(&mut self, recipe: &str) -> bool {
        let at = self.value["recipes"].as_array().into_iter().flatten().position(|known| known == recipe);
        if let Some(at) = at {
            self.value["recipe"] = json!(recipe);
            query(json!({ "op": "ev_player_button", "button": at }));
        }
        at.is_some()
    }
    /// bukkit: io/papermc/paper/event/player/PlayerStonecutterRecipeSelectEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerStonecutterRecipeSelectEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerLoomPatternSelectEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerLoomPatternSelectEvent {
    value: Value,
    cancelled: bool,
}

held_event!(PlayerLoomPatternSelectEvent, |value| value["kind"] == "button" && value["type"] == "LOOM");

impl PlayerLoomPatternSelectEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLoomPatternSelectEvent.getLoomInventory
    pub fn get_loom_inventory(&self) -> MenuInventory {
        let mut value = self.value.clone();
        value["size"] = json!(4);
        MenuInventory::read(&value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLoomPatternSelectEvent.getPatternType
    ///
    /// The pattern's key.
    pub fn get_pattern_type(&self) -> Option<&str> {
        self.value["pattern"].as_str()
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLoomPatternSelectEvent.setPatternType
    ///
    /// By its key. One the loom isn\'t offering is refused with `false`, and the click stays what it was, as in Paper.
    pub fn set_pattern_type(&mut self, pattern: &str) -> bool {
        let at = self.value["patterns"].as_array().into_iter().flatten().position(|known| known == pattern);
        if let Some(at) = at {
            self.value["pattern"] = json!(pattern);
            query(json!({ "op": "ev_player_button", "button": at }));
        }
        at.is_some()
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLoomPatternSelectEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLoomPatternSelectEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Packets held for a listener ----

/// bukkit: org/bukkit/event/inventory/InventoryCreativeEvent
///
/// A creative player's inventory screen setting a slot: `getCursor` is the stack it's setting.
#[derive(Clone, Debug, Default)]
pub struct InventoryCreativeEvent {
    value: Value,
    cancelled: bool,
}

held_event!(InventoryCreativeEvent, |value| value["kind"] == "creative");

impl InventoryCreativeEvent {
    pub fn get_who_clicked(&self) -> Option<Player> {
        player_of(&self.value)
    }
    pub fn get_raw_slot(&self) -> i32 {
        self.value["slot"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/inventory/InventoryCreativeEvent.getCursor
    pub fn get_cursor(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["cursor"])
    }
    /// bukkit: org/bukkit/event/inventory/InventoryCreativeEvent.setCursor
    pub fn set_cursor(&mut self, stack: Option<&ItemStack>) {
        query(json!({ "op": "ev_player_slot_set", "uuid": self.value["uuid"], "window": 0, "slot": self.value["slot"], "stack": snbt_of(stack), "after": true }));
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/inventory/TradeSelectEvent
#[derive(Clone, Debug, Default)]
pub struct TradeSelectEvent {
    value: Value,
    cancelled: bool,
}

held_event!(TradeSelectEvent, |value| value["kind"] == "trade_select");

impl TradeSelectEvent {
    pub fn get_who_clicked(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/inventory/TradeSelectEvent.getIndex
    pub fn get_index(&self) -> i32 {
        self.value["index"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/inventory/TradeSelectEvent.getInventory
    pub fn get_inventory(&self) -> MenuInventory {
        MenuInventory::read(&self.value)
    }
    /// bukkit: org/bukkit/event/inventory/TradeSelectEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: player_of(&self.value), top: MenuInventory::read(&self.value) }
    }
    /// bukkit: org/bukkit/event/inventory/TradeSelectEvent.getMerchant
    pub fn get_merchant(&self) -> Option<Entity> {
        Some(Entity::of(self.value["entity"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerPickBlockEvent, io/papermc/paper/event/player/PlayerPickItemEvent
///
/// The pick key on a block. A cancel gives nothing.
#[derive(Clone, Debug, Default)]
pub struct PlayerPickBlockEvent {
    value: Value,
    cancelled: bool,
    slots: Option<(i32, i32)>,
}

/// bukkit: io/papermc/paper/event/player/PlayerPickItemEvent.getItem, io/papermc/paper/event/player/PlayerPickItemEvent.getTargetSlot, io/papermc/paper/event/player/PlayerPickItemEvent.getSourceSlot, io/papermc/paper/event/player/PlayerPickItemEvent.setTargetSlot, io/papermc/paper/event/player/PlayerPickItemEvent.setSourceSlot
macro_rules! pick_methods {
    ($name:ident) => {
        impl $name {
            /// The stack that was asked for.
            pub fn get_item(&self) -> Option<ItemStack> {
                ItemStack::read(&self.value["item"])
            }
            /// The hotbar slot it ends up in, and the one selected.
            pub fn get_target_slot(&self) -> i32 {
                self.slots.map_or(self.value["target"].as_i64().unwrap_or(0) as i32, |slots| slots.1)
            }
            pub fn set_target_slot(&mut self, slot: i32) {
                self.set_slots(self.get_source_slot(), slot.clamp(0, 8));
            }
            /// The inventory slot it comes from: -1 if it's not there.
            pub fn get_source_slot(&self) -> i32 {
                self.slots.map_or(self.value["source"].as_i64().unwrap_or(-1) as i32, |slots| slots.0)
            }
            /// -1 makes the item for a creative player, and gives others nothing.
            pub fn set_source_slot(&mut self, slot: i32) {
                self.set_slots(slot.clamp(-1, 35), self.get_target_slot());
            }
            fn set_slots(&mut self, source: i32, target: i32) {
                self.slots = Some((source, target));
                crate::event::carry("source", json!(source));
                crate::event::carry("target", json!(target));
                query(json!({ "op": "ev_player_pick_to", "source": source, "target": target }));
            }
        }
    };
}

pick_methods!(PlayerPickBlockEvent);
pick_methods!(PlayerPickEntityEvent);

held_event!(PlayerPickBlockEvent, |value| value["kind"] == "pick_block");

impl PlayerPickBlockEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerPickBlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    pub fn is_include_data(&self) -> bool {
        self.value["include_data"] == true
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerPickEntityEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerPickEntityEvent {
    value: Value,
    cancelled: bool,
    slots: Option<(i32, i32)>,
}

held_event!(PlayerPickEntityEvent, |value| value["kind"] == "pick_entity");

impl PlayerPickEntityEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerPickEntityEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        Some(Entity::of(self.value["entity"].as_u64()?, &world_of(&self.value)))
    }
    pub fn is_include_data(&self) -> bool {
        self.value["include_data"] == true
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerUseUnknownEntityEvent
///
/// A click on an entity id nothing in the world has: a plugin's fake entity. The game drops such a
/// click anyway, so a cancel changes nothing, as in Paper.
#[derive(Clone, Debug, Default)]
pub struct PlayerUseUnknownEntityEvent {
    value: Value,
    cancelled: bool,
}

held_event!(PlayerUseUnknownEntityEvent, |value| value["kind"] == "unknown_entity");

impl PlayerUseUnknownEntityEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerUseUnknownEntityEvent.getEntityId
    pub fn get_entity_id(&self) -> i32 {
        self.value["entity"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerUseUnknownEntityEvent.isAttack
    pub fn is_attack(&self) -> bool {
        self.value["attack"] == true
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerUseUnknownEntityEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerUseUnknownEntityEvent.getClickedRelativePosition
    ///
    /// Where on the entity: `None` for an attack, as in Paper.
    pub fn get_clicked_relative_position(&self) -> Option<crate::util::Vector> {
        let at = &self.value["at"];
        Some(crate::util::Vector::new(at[0].as_f64()?, at[1].as_f64()?, at[2].as_f64()?))
    }
}

// ---- A player's hands ----

/// bukkit: org/bukkit/event/player/PlayerFishEvent
///
/// Each moment as it happens: a cast, a bite, a catch for each item caught, a reel.
#[derive(Clone, Debug, Default)]
pub struct PlayerFishEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerFishEvent, "pl_fish", ["pl_fish_reel", "pl_fish_tick", "pl_fish_xp"], |value| value["state"].is_string());

impl PlayerFishEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerFishEvent.getState
    ///
    /// `PlayerFishEvent.State`'s name.
    pub fn get_state(&self) -> &str {
        self.value["state"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/player/PlayerFishEvent.getCaught
    pub fn get_caught(&self) -> Option<Entity> {
        Some(Entity::of(self.value["caught"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/player/PlayerFishEvent.getHook
    pub fn get_hook(&self) -> Option<Entity> {
        Some(Entity::of(self.value["hook"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/player/PlayerFishEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: org/bukkit/event/player/PlayerFishEvent.getExpToDrop
    ///
    /// The experience a catch gives: nothing for any other state, as in Bukkit.
    pub fn get_exp_to_drop(&self) -> i32 {
        self.value["exp"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/player/PlayerFishEvent.setExpToDrop
    ///
    /// The size of the catch's orb.
    pub fn set_exp_to_drop(&mut self, exp: i32) {
        self.value["exp"] = json!(exp.max(0));
        query(json!({ "op": "ev_player_fish_exp", "exp": exp.max(0) }));
    }
    /// bukkit: org/bukkit/event/player/PlayerFishEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerFishEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerItemConsumeEvent
///
/// As a food or potion finishes. A cancel keeps the item, and the client is told.
#[derive(Clone, Debug, Default)]
pub struct PlayerItemConsumeEvent {
    value: Value,
    cancelled: bool,
    replacement: Option<ItemStack>,
    item: Option<ItemStack>,
}

impl Event for PlayerItemConsumeEvent {
    const HOOK: &'static str = "pl_consume";
    fn more_hooks() -> &'static [&'static str] {
        &["tick"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        tick_answer(value)
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
    fn finish(&self) {
        if self.cancelled {
            let uuid = self.value["uuid"].clone();
            at_tick(move || {
                query(json!({ "op": "ev_player_resend", "uuid": uuid }));
            });
        }
    }
}

impl PlayerItemConsumeEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerItemConsumeEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        self.item.clone().or_else(|| ItemStack::read(&self.value["stack"]))
    }
    /// bukkit: org/bukkit/event/player/PlayerItemConsumeEvent.setItem
    ///
    /// What's eaten in its place: its food and effects count, and the hand's stack goes down as usual.
    pub fn set_item(&mut self, stack: &ItemStack) {
        self.item = Some(stack.clone());
        query(json!({ "op": "ev_player_eat", "stack": snbt_of(Some(stack)) }));
    }
    /// bukkit: org/bukkit/event/player/PlayerItemConsumeEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: org/bukkit/event/player/PlayerItemConsumeEvent.getReplacement
    pub fn get_replacement(&self) -> Option<&ItemStack> {
        self.replacement.as_ref()
    }
    /// bukkit: org/bukkit/event/player/PlayerItemConsumeEvent.setReplacement
    ///
    /// What the hand holds after the meal, in place of what the game would leave there.
    pub fn set_replacement(&mut self, stack: Option<&ItemStack>) {
        self.replacement = stack.cloned();
        query(json!({ "op": "ev_player_replacement", "stack": snbt_of(stack) }));
    }
    /// bukkit: org/bukkit/event/player/PlayerItemConsumeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerItemConsumeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerItemDamageEvent
///
/// Wear on a player's held tool or worn armour.
#[derive(Clone, Debug, Default)]
pub struct PlayerItemDamageEvent {
    value: Value,
    cancelled: bool,
    damage: Option<i32>,
}

impl Event for PlayerItemDamageEvent {
    const HOOK: &'static str = "pl_item_damage";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
    fn finish(&self) {
        if let Some(damage) = self.damage {
            query(json!({ "op": "ev_player_loss", "loss": damage.max(0) }));
        }
    }
}

impl PlayerItemDamageEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerItemDamageEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: org/bukkit/event/player/PlayerItemDamageEvent.getOriginalDamage
    pub fn get_original_damage(&self) -> i32 {
        self.value["amount"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/player/PlayerItemDamageEvent.getDamage
    ///
    /// The damage after Unbreaking.
    pub fn get_damage(&self) -> i32 {
        self.damage.unwrap_or(self.value["loss"].as_i64().unwrap_or(0) as i32)
    }
    /// bukkit: org/bukkit/event/player/PlayerItemDamageEvent.setDamage
    pub fn set_damage(&mut self, damage: i32) {
        self.damage = Some(damage);
    }
    /// bukkit: org/bukkit/event/player/PlayerItemDamageEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerItemDamageEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerItemBreakEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerItemBreakEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerItemBreakEvent, "pl_item_break", |value| value["broken"] == true);

impl PlayerItemBreakEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerItemBreakEvent.getBrokenItem
    pub fn get_broken_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
}

/// bukkit: org/bukkit/event/player/PlayerArmorStandManipulateEvent
///
/// A click that would swap an item with an armor stand. A cancel leaves both as they were.
#[derive(Clone, Debug, Default)]
pub struct PlayerArmorStandManipulateEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerArmorStandManipulateEvent, "pl_stand_use", |value| value["use"].is_object());

impl PlayerArmorStandManipulateEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerArmorStandManipulateEvent.getRightClicked
    pub fn get_right_clicked(&self) -> Option<Entity> {
        Some(Entity::of(self.value["use"]["entity"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/player/PlayerArmorStandManipulateEvent.getPlayerItem
    pub fn get_player_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["use"]["held"])
    }
    /// bukkit: org/bukkit/event/player/PlayerArmorStandManipulateEvent.getArmorStandItem
    pub fn get_armor_stand_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["use"]["worn"])
    }
    /// bukkit: org/bukkit/event/player/PlayerArmorStandManipulateEvent.getSlot
    ///
    /// `EquipmentSlot`'s name.
    pub fn get_slot(&self) -> &str {
        self.value["use"]["slot"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/player/PlayerArmorStandManipulateEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["use"]["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerShearEntityEvent
///
/// Sheep, mooshrooms, snow golems and bogged, as the shears are used.
#[derive(Clone, Debug, Default)]
pub struct PlayerShearEntityEvent {
    value: Value,
    cancelled: bool,
    drops: Option<Vec<ItemStack>>,
}

hooked!(PlayerShearEntityEvent, "pl_shear_mob", ["pl_shear"], |value| value["item"] == "minecraft:shears");

impl PlayerShearEntityEvent {
    /// bukkit: org/bukkit/event/player/PlayerShearEntityEvent.getDrops
    ///
    /// What comes off: already rolled.
    pub fn get_drops(&self) -> Vec<ItemStack> {
        self.drops.clone().unwrap_or_else(|| stacks_of(&self.value["drops"]).into_iter().flatten().collect())
    }
    /// bukkit: org/bukkit/event/player/PlayerShearEntityEvent.setDrops
    pub fn set_drops(&mut self, drops: &[ItemStack]) {
        self.drops = Some(drops.to_vec());
        let rows: Vec<Value> = drops.iter().map(|stack| json!(stack.to_snbt())).collect();
        query(json!({ "op": "ev_player_shear_set", "drops": rows }));
    }
    /// bukkit: org/bukkit/event/player/PlayerShearEntityEvent.getItem
    ///
    /// The shears.
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerShearEntityEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        Some(Entity::of(self.value["entity"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/player/PlayerShearEntityEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: org/bukkit/event/player/PlayerShearEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerShearEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerGameModeChangeEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerGameModeChangeEvent {
    value: Value,
    cancelled: bool,
    message: Option<crate::Component>,
}

impl Event for PlayerGameModeChangeEvent {
    const HOOK: &'static str = "pl_game_mode";
    fn read(value: &Value) -> Option<Self> {
        (value["changes"] == true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), message: None })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
    fn finish(&self) {
        // The one who typed `/gamemode` hears it: no other cause has anyone to tell.
        if let (true, Some(message), "COMMAND") = (self.cancelled, &self.message, self.get_cause()) {
            query(json!({ "op": "ev_player_mode_message", "text": message.0 }));
        }
    }
}

impl PlayerGameModeChangeEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerGameModeChangeEvent.getNewGameMode
    ///
    /// `GameMode`'s name.
    pub fn get_new_game_mode(&self) -> String {
        self.value["mode"].as_str().unwrap_or_default().to_uppercase()
    }
    /// bukkit: org/bukkit/event/player/PlayerGameModeChangeEvent.cancelMessage
    ///
    /// What a cancelled change tells the player.
    pub fn cancel_message(&self) -> Option<&crate::Component> {
        self.message.as_ref()
    }
    pub fn set_cancel_message(&mut self, message: Option<&crate::Component>) {
        self.message = message.cloned();
    }
    /// bukkit: org/bukkit/event/player/PlayerGameModeChangeEvent.getCause
    ///
    /// `Cause`'s name.
    pub fn get_cause(&self) -> &str {
        self.value["cause"].as_str().unwrap_or("UNKNOWN")
    }
    /// bukkit: org/bukkit/event/player/PlayerGameModeChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerGameModeChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/inventory/InventoryMoveItemEvent
///
/// A hopper pushing or pulling. A cancel keeps the stack where it was.
#[derive(Clone, Debug, Default)]
pub struct InventoryMoveItemEvent {
    value: Value,
    cancelled: bool,
    item: Option<ItemStack>,
}

impl Event for InventoryMoveItemEvent {
    const HOOK: &'static str = "pl_move_item";
    /// A hopper's pull says where it pulls from only while these are heard (they mark the source, nothing more):
    /// without them a pull came with no source, `read` dropped it, and CoreProtect never logged a hopper taking items.
    fn more_hooks() -> &'static [&'static str] {
        &["pl_pulling", "pl_not_pulling", "pl_hopper_turn"]
    }
    fn read(value: &Value) -> Option<Self> {
        ((!value["from"].is_null() || !value["inventory"].is_null()) && value["again"] != true).then(|| Self { item: ItemStack::read(&value["stack"]), value: value.clone(), ..Default::default() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

/// A block's container as it stands: its `Items` in the saved data.
fn block_inventory(value: &Value, pos: &Value) -> Option<MenuInventory> {
    let block = block_at(value, pos)?;
    let data = query(json!({ "op": "block_data", "dimension": block.get_world().dimension(), "pos": pos }));
    let list = data.get("data").unwrap_or(&data).get("Items").cloned().unwrap_or(Value::Null);
    let mut slots: Vec<Option<ItemStack>> = Vec::new();
    for item in list.as_array().into_iter().flatten() {
        let slot = item["Slot"].as_i64().unwrap_or(0) as usize;
        if slots.len() <= slot {
            slots.resize(slot + 1, None);
        }
        slots[slot] = ItemStack::read(item);
    }
    let kind = container_type(block.get_type().key());
    Some(MenuInventory { kind: kind.to_owned(), size: slots.len().max(default_size(kind)), items: slots, location: Some(block.get_location()), ..Default::default() })
}

/// `InventoryType`'s name for a block that holds items.
pub fn container_type(block: &str) -> &'static str {
    match block.trim_start_matches("minecraft:").to_lowercase().as_str() {
        "barrel" => "BARREL",
        "hopper" => "HOPPER",
        "dispenser" => "DISPENSER",
        "dropper" => "DROPPER",
        "furnace" => "FURNACE",
        "blast_furnace" => "BLAST_FURNACE",
        "smoker" => "SMOKER",
        "brewing_stand" => "BREWING",
        "crafter" => "CRAFTER",
        name if name.ends_with("shulker_box") => "SHULKER_BOX",
        _ => "CHEST",
    }
}

impl InventoryMoveItemEvent {
    /// bukkit: org/bukkit/event/inventory/InventoryMoveItemEvent.getItem
    pub fn get_item(&self) -> Option<&ItemStack> {
        self.item.as_ref()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryMoveItemEvent.setItem
    ///
    /// What moves, in place of the game's stack.
    pub fn set_item(&mut self, stack: &ItemStack) {
        self.item = Some(stack.clone());
        self.cancelled = true;
        query(json!({ "op": "ev_player_moved", "stack": snbt_of(Some(stack)) }));
    }
    /// bukkit: org/bukkit/event/inventory/InventoryMoveItemEvent.getSource
    pub fn get_source(&self) -> Option<MenuInventory> {
        if self.value["from"].is_null() {
            return Some(MenuInventory::read(&self.value["inventory"]));
        }
        block_inventory(&self.value, &self.value["from"])
    }
    /// bukkit: org/bukkit/event/inventory/InventoryMoveItemEvent.getDestination
    pub fn get_destination(&self) -> Option<MenuInventory> {
        if self.value["to"].is_null() {
            return Some(MenuInventory::read(&self.value["inventory"]));
        }
        block_inventory(&self.value, &self.value["to"])
    }
    /// bukkit: org/bukkit/event/inventory/InventoryMoveItemEvent.getInitiator
    ///
    /// The hopper that does it: the source's when it pushes, the destination's when it pulls.
    pub fn get_initiator(&self) -> Option<MenuInventory> {
        if self.value["pull"] == true { self.get_destination() } else { self.get_source() }
    }
    /// bukkit: org/bukkit/event/inventory/InventoryMoveItemEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/inventory/InventoryMoveItemEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/inventory/InventoryPickupItemEvent
///
/// An item entity going into a hopper.
#[derive(Clone, Debug, Default)]
pub struct InventoryPickupItemEvent {
    value: Value,
    cancelled: bool,
}

hooked!(InventoryPickupItemEvent, "pl_move_item", |value| value["item"].is_u64() && value["again"] != true);

impl InventoryPickupItemEvent {
    /// bukkit: org/bukkit/event/inventory/InventoryPickupItemEvent.getItem
    ///
    /// The item entity on the hopper.
    pub fn get_item(&self) -> Option<Entity> {
        Some(Entity::of(self.value["item"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/inventory/InventoryPickupItemEvent.getInventory
    pub fn get_inventory(&self) -> Option<MenuInventory> {
        block_inventory(&self.value, &self.value["to"])
    }
    /// bukkit: org/bukkit/event/inventory/InventoryPickupItemEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/inventory/InventoryPickupItemEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Worlds, logins ----

/// bukkit: org/bukkit/event/player/PlayerChangedWorldEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerChangedWorldEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerChangedWorldEvent, "change_level", |_value| true);

impl PlayerChangedWorldEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerChangedWorldEvent.getFrom
    ///
    /// The world they left.
    pub fn get_from(&self) -> World {
        World::of(self.value["from"].as_str().unwrap_or("minecraft:overworld"))
    }
}

/// The player's socket as the game has it: address and port.
fn socket_of(player: &Player) -> Option<String> {
    query(json!({ "op": "ev_player_socket", "uuid": player.uuid() })).as_str().map(str::to_owned)
}

impl crate::event::PlayerLoginEvent {
    /// bukkit: org/bukkit/event/player/PlayerLoginEvent.getRealSocketAddress
    pub fn get_real_socket_address(&self) -> Option<std::net::SocketAddr> {
        self.value["socket"].as_str()?.parse().ok()
    }
    /// bukkit: org/bukkit/event/player/PlayerLoginEvent.getAddress, org/bukkit/event/player/PlayerLoginEvent.getRealAddress
    pub fn get_address(&self) -> Option<std::net::IpAddr> {
        self.value["address"].as_str()?.parse().ok()
    }
    /// bukkit: org/bukkit/event/player/PlayerLoginEvent.kickMessage
    pub fn kick_message(&self) -> crate::Component {
        crate::adventure::legacy(self.get_kick_message())
    }
    /// bukkit: org/bukkit/event/player/PlayerLoginEvent.getHostname
    ///
    /// The host and port the client says it connected to.
    pub fn get_hostname(&self) -> &str {
        self.value["hostname"].as_str().unwrap_or_default()
    }
}

// ---- Signs, books, names ----

// ---- What clients say ----

/// bukkit: org/bukkit/event/player/PlayerResourcePackStatusEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerResourcePackStatusEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerResourcePackStatusEvent, "packet_pack", |_value| true);

impl PlayerResourcePackStatusEvent {
    /// bukkit: org/bukkit/event/player/PlayerResourcePackStatusEvent.getHash
    ///
    /// Always `None`: clients stopped sending it, as Paper's own note says.
    pub fn get_hash(&self) -> Option<&str> {
        None
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerResourcePackStatusEvent.getID
    pub fn get_id(&self) -> &str {
        self.value["id"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/player/PlayerResourcePackStatusEvent.getStatus
    ///
    /// `Status`'s name, from the action the client sent.
    pub fn get_status(&self) -> &'static str {
        match self.value["action"].as_i64().unwrap_or(-1) {
            0 => "SUCCESSFULLY_LOADED",
            1 => "DECLINED",
            2 => "FAILED_DOWNLOAD",
            3 => "ACCEPTED",
            4 => "DOWNLOADED",
            5 => "INVALID_URL",
            6 => "FAILED_RELOAD",
            _ => "DISCARDED",
        }
    }
}

/// bukkit: org/bukkit/event/player/PlayerLocaleChangeEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerLocaleChangeEvent {
    value: Value,
    cancelled: bool,
}

/// Whether `key` of a client's options differs from what it last sent; the first time says no.
fn option_changed(seen: &'static std::sync::Mutex<BTreeMap<String, Value>>, value: &Value, key: &str) -> bool {
    let Some(uuid) = value["uuid"].as_str() else {
        return false;
    };
    let before = seen.lock().unwrap_or_else(|e| e.into_inner()).insert(uuid.to_owned(), value[key].clone());
    before.is_some_and(|before| before != value[key])
}

static LOCALES: std::sync::Mutex<BTreeMap<String, Value>> = std::sync::Mutex::new(BTreeMap::new());
static MAIN_HANDS: std::sync::Mutex<BTreeMap<String, Value>> = std::sync::Mutex::new(BTreeMap::new());

hooked!(PlayerLocaleChangeEvent, "packet_options", |value| option_changed(&LOCALES, value, "locale"));

impl PlayerLocaleChangeEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerLocaleChangeEvent.getLocale
    pub fn get_locale(&self) -> &str {
        self.value["locale"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/player/PlayerLocaleChangeEvent.locale
    pub fn locale(&self) -> &str {
        self.get_locale()
    }
}

/// bukkit: org/bukkit/event/player/PlayerChangedMainHandEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerChangedMainHandEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerChangedMainHandEvent, "packet_options", |value| option_changed(&MAIN_HANDS, value, "main_hand"));

impl PlayerChangedMainHandEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerChangedMainHandEvent.getNewMainHand
    ///
    /// `MainHand`'s name.
    pub fn get_new_main_hand(&self) -> &'static str {
        if self.value["main_hand"] == 0 { "LEFT" } else { "RIGHT" }
    }
    /// bukkit: org/bukkit/event/player/PlayerChangedMainHandEvent.getMainHand
    pub fn get_main_hand(&self) -> &'static str {
        self.get_new_main_hand()
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent
///
/// Every `has...Changed` compares with what the player last sent.
#[derive(Clone, Debug, Default)]
pub struct PlayerClientOptionsChangeEvent {
    value: Value,
    cancelled: bool,
    before: Value,
}

/// What each player's client last said of itself.
static OPTIONS: std::sync::Mutex<BTreeMap<String, Value>> = std::sync::Mutex::new(BTreeMap::new());

impl Event for PlayerClientOptionsChangeEvent {
    const HOOK: &'static str = "packet_options";
    fn read(value: &Value) -> Option<Self> {
        let uuid = value["uuid"].as_str()?.to_owned();
        let before = OPTIONS.lock().unwrap_or_else(|e| e.into_inner()).insert(uuid, value.clone()).unwrap_or(Value::Null);
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), before })
    }
}

impl PlayerClientOptionsChangeEvent {
    fn changed(&self, key: &str) -> bool {
        self.before[key] != self.value[key]
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.getLocale
    pub fn get_locale(&self) -> &str {
        self.value["locale"].as_str().unwrap_or_default()
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.getViewDistance
    pub fn get_view_distance(&self) -> i64 {
        self.value["view_distance"].as_i64().unwrap_or(0)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.hasChatColorsEnabled
    pub fn has_chat_colors_enabled(&self) -> bool {
        self.value["chat_colors"] == true
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.hasLocaleChanged
    pub fn has_locale_changed(&self) -> bool {
        self.changed("locale")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.hasViewDistanceChanged
    pub fn has_view_distance_changed(&self) -> bool {
        self.changed("view_distance")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.hasChatVisibilityChanged
    pub fn has_chat_visibility_changed(&self) -> bool {
        self.changed("chat_visibility")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.hasChatColorsEnabledChanged
    pub fn has_chat_colors_enabled_changed(&self) -> bool {
        self.changed("chat_colors")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.hasSkinPartsChanged
    pub fn has_skin_parts_changed(&self) -> bool {
        self.changed("skin_parts")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.hasMainHandChanged
    pub fn has_main_hand_changed(&self) -> bool {
        self.changed("main_hand")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.hasAllowServerListingsChanged
    pub fn has_allow_server_listings_changed(&self) -> bool {
        self.changed("allows_listing")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.hasTextFilteringChanged
    pub fn has_text_filtering_changed(&self) -> bool {
        self.changed("text_filtering")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.hasParticleVisibilityChanged
    pub fn has_particle_visibility_changed(&self) -> bool {
        self.changed("particle_status")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.hasTextFilteringEnabled
    pub fn has_text_filtering_enabled(&self) -> bool {
        self.value["text_filtering"] == true
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.allowsServerListings
    pub fn allows_server_listings(&self) -> bool {
        self.value["allows_listing"] == true
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.getMainHand
    ///
    /// `MainHand`'s name.
    pub fn get_main_hand(&self) -> &'static str {
        if self.value["main_hand"] == 0 { "LEFT" } else { "RIGHT" }
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.getChatVisibility
    ///
    /// `ClientOption.ChatVisibility`'s name.
    pub fn get_chat_visibility(&self) -> &'static str {
        match self.value["chat_visibility"].as_i64() {
            Some(0) => "FULL",
            Some(1) => "SYSTEM",
            Some(2) => "HIDDEN",
            _ => "UNKNOWN",
        }
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.getParticleVisibility
    ///
    /// `ClientOption.ParticleVisibility`'s name.
    pub fn get_particle_visibility(&self) -> &'static str {
        match self.value["particle_status"].as_i64() {
            Some(0) => "ALL",
            Some(1) => "DECREASED",
            _ => "MINIMAL",
        }
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerClientOptionsChangeEvent.getSkinParts
    ///
    /// The skin parts' bits, as the client sends them.
    pub fn get_skin_parts(&self) -> i64 {
        self.value["skin_parts"].as_i64().unwrap_or(0)
    }
}

/// What a player's client holds down.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayerKeys(u8);

impl PlayerKeys {
    pub fn is_forward(&self) -> bool {
        self.0 & 1 != 0
    }
    pub fn is_backward(&self) -> bool {
        self.0 & 2 != 0
    }
    pub fn is_left(&self) -> bool {
        self.0 & 4 != 0
    }
    pub fn is_right(&self) -> bool {
        self.0 & 8 != 0
    }
    pub fn is_jump(&self) -> bool {
        self.0 & 16 != 0
    }
    pub fn is_sneak(&self) -> bool {
        self.0 & 32 != 0
    }
    pub fn is_sprint(&self) -> bool {
        self.0 & 64 != 0
    }
}

/// bukkit: org/bukkit/event/player/PlayerInputEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerInputEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerInputEvent, "packet_input", |_value| true);

impl PlayerInputEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerInputEvent.getInput
    pub fn get_input(&self) -> PlayerKeys {
        PlayerKeys(self.value["flags"].as_u64().unwrap_or(0) as u8)
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerJumpEvent
///
/// A move that leaves the ground going up, read as the moves are: before the game acts on it. A
/// cancel sends them back to `getFrom`.
#[derive(Clone, Debug)]
pub struct PlayerJumpEvent {
    player: Player,
    from: Location,
    to: Location,
    cancelled: bool,
}

impl Event for PlayerJumpEvent {
    const HOOK: &'static str = "pl_jump";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_moves", "pl_move_turn"]
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { player: player_of(value)?, from: place_of(value, "from")?, to: place_of(value, "to")?, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
    fn finish(&self) {
        if self.cancelled {
            query(json!({ "op": "ev_player_move_to", "to": place_json(&self.from) }));
        }
    }
}

impl PlayerJumpEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerJumpEvent.getFrom
    pub fn get_from(&self) -> &Location {
        &self.from
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerJumpEvent.setFrom
    pub fn set_from(&mut self, from: &Location) {
        self.from = from.clone();
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerJumpEvent.getTo
    pub fn get_to(&self) -> &Location {
        &self.to
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerJumpEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerJumpEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Chat ----

/// Everyone on as the line came in.
fn chat_online(value: &Value) -> Vec<Player> {
    value["online"].as_array().into_iter().flatten().filter_map(Player::read).collect()
}

/// Bukkit's chat format: `%1$s` the name, `%2$s` the message.
fn chat_line(format: &str, name: &str, message: &str) -> String {
    format.replacen("%1$s", name, 1).replacen("%s", name, 1).replacen("%2$s", message, 1).replacen("%s", message, 1)
}

/// Who the listeners before this one left the line to.
fn chat_recipients(value: &Value) -> Option<Vec<Player>> {
    let wanted = value["chat_recipients"].as_array()?;
    Some(chat_online(value).into_iter().filter(|player| wanted.iter().any(|uuid| uuid.as_str() == Some(player.uuid()))).collect())
}

/// `ChatRenderer`: source, their name, the message and the viewer, for the line that viewer gets.
pub type ChatRenderer = std::sync::Arc<dyn Fn(&Player, &crate::Component, &crate::Component, &Player) -> crate::Component + Send + Sync>;

/// The renderer the last `AsyncChatEvent` of this plugin set, for `chat_final`.
static RENDERER: std::sync::Mutex<Option<ChatRenderer>> = std::sync::Mutex::new(None);

/// The line as the listeners left it.
fn chat_message(value: &Value) -> crate::Component {
    if value["chat_component"].is_object() || value["chat_component"].is_string() {
        return crate::Component(value["chat_component"].clone());
    }
    crate::adventure::legacy(value["chat_message"].as_str().or(value["message"].as_str()).unwrap_or_default())
}

/// `ChatProcessor.complete`, after the last listener: what goes out in place of the signed text. Left
/// alone, the game sends the message as it came. A changed message is the same signed message with
/// unsigned content, in the chat type's own decoration. A renderer or a format of its own takes the
/// `raw` type, with each viewer's line as the unsigned content.
pub(crate) fn chat_final(value: &Value) -> Answer {
    if value.get("chat_lines").is_some() {
        return Answer::Pass;
    }
    let name = value["name"].as_str().unwrap_or_default();
    let message = chat_message(value);
    if value["chat_renderer"] == true {
        let Some(render) = RENDERER.lock().unwrap_or_else(|e| e.into_inner()).take() else {
            return Answer::Pass;
        };
        let online = chat_online(value);
        let Some(source) = online.iter().find(|player| Some(player.uuid()) == value["uuid"].as_str()).cloned() else {
            return Answer::Pass;
        };
        let viewers = chat_recipients(value).unwrap_or_else(|| online.clone());
        let shown = crate::Component::text(name);
        let mut lines = serde_json::Map::new();
        for viewer in &viewers {
            lines.insert(viewer.uuid().to_owned(), render(&source, &shown, &message, viewer).0);
        }
        crate::event::carry("chat_line", json!(render(&source, &shown, &message, &source).plain()));
        crate::event::carry("chat_lines", Value::Object(lines));
        return Answer::Pass;
    }
    let format = value["chat_format"].as_str().unwrap_or("<%1$s> %2$s");
    if format != "<%1$s> %2$s" {
        let line = crate::adventure::legacy(&chat_line(format, name, &message.to_legacy()));
        crate::event::carry("chat_line", json!(line.plain()));
        crate::event::carry("chat_lines", json!({ "*": line.0 }));
    } else if value["chat_message"].is_string() || !value["chat_component"].is_null() {
        crate::event::carry("chat_line", json!(format!("<{name}> {}", message.plain())));
        crate::event::carry("chat_unsigned", message.0);
    }
    Answer::Pass
}

/// bukkit: org/bukkit/event/player/AsyncPlayerChatEvent.isAsynchronous, org/bukkit/event/player/PlayerChatEvent.isAsynchronous
macro_rules! chat_event {
    ($name:ident, $hook:literal, $thread:literal) => {
        impl Event for $name {
            const HOOK: &'static str = $hook;
            fn read(value: &Value) -> Option<Self> {
                Some(Self { player: player_of(value)?, original: value["message"].as_str()?.to_owned(), message: value["chat_message"].as_str().map(str::to_owned), format: value["chat_format"].as_str().map(str::to_owned), recipients: chat_recipients(value), online: chat_online(value), value: value.clone(), cancelled: crate::event::prior_cancelled() || value["chat_cancelled"] == true })
            }
            fn answer(&self) -> Answer {
                if self.cancelled { Answer::Fail } else { Answer::Pass }
            }
            fn finish(&self) {
                crate::event::carry("chat_cancelled", json!(self.cancelled));
                if let Some(message) = &self.message {
                    crate::event::carry("chat_message", json!(message));
                }
                if let Some(format) = &self.format {
                    crate::event::carry("chat_format", json!(format));
                }
                if let Some(recipients) = &self.recipients {
                    crate::event::carry("chat_recipients", json!(recipients.iter().map(|player| player.uuid().to_owned()).collect::<Vec<_>>()));
                }
            }
            fn is_asynchronous(&self) -> bool {
                // A plugin's `Player.chat` runs where the plugin is.
                $thread && self.value["compelled"] != true
            }
        }
    };
}

/// bukkit: org/bukkit/event/player/AsyncPlayerChatEvent
///
/// On the player's own chat thread, as Paper's is: most of the game can't be touched from here, and
/// a question to it waits for the next tick.
#[derive(Clone, Debug)]
pub struct AsyncPlayerChatEvent {
    player: Player,
    original: String,
    message: Option<String>,
    format: Option<String>,
    recipients: Option<Vec<Player>>,
    online: Vec<Player>,
    value: Value,
    cancelled: bool,
}

/// bukkit: org/bukkit/event/player/PlayerChatEvent
///
/// After `AsyncPlayerChatEvent`, on the game's thread, with chat waiting for it.
#[derive(Clone, Debug)]
pub struct PlayerChatEvent {
    player: Player,
    original: String,
    message: Option<String>,
    format: Option<String>,
    recipients: Option<Vec<Player>>,
    online: Vec<Player>,
    value: Value,
    cancelled: bool,
}

chat_event!(AsyncPlayerChatEvent, "pl_chat_old", true);
chat_event!(PlayerChatEvent, "pl_chat_main", false);

impl AsyncPlayerChatEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerChatEvent.getMessage
    pub fn get_message(&self) -> &str {
        self.message.as_deref().unwrap_or(&self.original)
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerChatEvent.setMessage
    pub fn set_message(&mut self, message: &str) {
        self.message = Some(message.to_owned());
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerChatEvent.getFormat
    pub fn get_format(&self) -> &str {
        self.format.as_deref().unwrap_or("<%1$s> %2$s")
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerChatEvent.setFormat
    pub fn set_format(&mut self, format: &str) {
        self.format = Some(format.to_owned());
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerChatEvent.getRecipients
    ///
    /// Everyone on as the line came in, unless changed with `set_recipients`.
    pub fn get_recipients(&self) -> Vec<Player> {
        self.recipients.clone().unwrap_or_else(|| self.online.clone())
    }
    /// Java's set is live: put the changed list back with this.
    pub fn set_recipients(&mut self, recipients: Vec<Player>) {
        self.recipients = Some(recipients);
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerChatEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerChatEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl PlayerChatEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/event/player/PlayerChatEvent.setPlayer
    ///
    /// Who the line reads as from here on.
    pub fn set_player(&mut self, player: &Player) {
        self.player = player.clone();
        crate::event::carry("uuid", json!(player.uuid()));
        crate::event::carry("name", json!(player.get_name()));
    }
    /// bukkit: org/bukkit/event/player/PlayerChatEvent.getMessage
    pub fn get_message(&self) -> &str {
        self.message.as_deref().unwrap_or(&self.original)
    }
    /// bukkit: org/bukkit/event/player/PlayerChatEvent.setMessage
    pub fn set_message(&mut self, message: &str) {
        self.message = Some(message.to_owned());
    }
    /// bukkit: org/bukkit/event/player/PlayerChatEvent.getFormat
    pub fn get_format(&self) -> &str {
        self.format.as_deref().unwrap_or("<%1$s> %2$s")
    }
    /// bukkit: org/bukkit/event/player/PlayerChatEvent.setFormat
    pub fn set_format(&mut self, format: &str) {
        self.format = Some(format.to_owned());
    }
    /// bukkit: org/bukkit/event/player/PlayerChatEvent.getRecipients
    ///
    /// Everyone on as the line came in, unless changed with `set_recipients`.
    pub fn get_recipients(&self) -> Vec<Player> {
        self.recipients.clone().unwrap_or_else(|| self.online.clone())
    }
    pub fn set_recipients(&mut self, recipients: Vec<Player>) {
        self.recipients = Some(recipients);
    }
    /// bukkit: org/bukkit/event/player/PlayerChatEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerChatEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Beds ----

/// bukkit: org/bukkit/event/player/PlayerBedEnterEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerBedEnterEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerBedEnterEvent, "pl_bed", ["pl_packet", "pl_bed_blast"], |value| value["result"].is_string());

/// `BedEnterAction`: what a bed attempt is about to do.
#[derive(Clone, Debug)]
pub struct BedAction {
    problem: Option<String>,
}

impl BedAction {
    /// `BedRuleResult`'s name.
    pub fn can_sleep(&self) -> &'static str {
        if self.problem.is_none() { "ALLOWED" } else { "NEVER" }
    }
    /// The `BedEnterProblem` by `BedEnterResult`'s name.
    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }
    /// `BedRuleResult`'s name: only the overworld's beds set the spawn.
    pub fn can_set_spawn(&self) -> &'static str {
        if self.problem.as_deref() == Some("NOT_POSSIBLE_HERE") { "NEVER" } else { "ALLOWED" }
    }
    /// What the game says of it.
    pub fn error_message(&self) -> Option<crate::Component> {
        let key = match self.problem.as_deref()? {
            "NOT_SAFE" => "block.minecraft.bed.not_safe",
            "NOT_POSSIBLE_NOW" => "block.minecraft.bed.no_sleep",
            "TOO_FAR_AWAY" => "block.minecraft.bed.too_far_away",
            _ => return None,
        };
        Some(crate::Component(json!({ "translate": key })))
    }
}

fn bed_action(value: &Value) -> BedAction {
    BedAction { problem: value["result"].as_str().filter(|result| *result != "OK").map(str::to_owned) }
}

impl PlayerBedEnterEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerBedEnterEvent.getBed
    pub fn get_bed(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/player/PlayerBedEnterEvent.getBedEnterResult
    ///
    /// `BedEnterResult`'s name.
    pub fn get_bed_enter_result(&self) -> &str {
        self.value["result"].as_str().unwrap_or("OK")
    }
    /// bukkit: org/bukkit/event/player/PlayerBedEnterEvent.enterAction
    pub fn enter_action(&self) -> BedAction {
        bed_action(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerBedEnterEvent.useBed
    ///
    /// `Event.Result`: `DENY` once cancelled.
    pub fn use_bed(&self) -> &'static str {
        if self.cancelled { "DENY" } else { "DEFAULT" }
    }
    /// bukkit: org/bukkit/event/player/PlayerBedEnterEvent.setUseBed
    pub fn set_use_bed(&mut self, result: &str) {
        self.cancelled = result == "DENY";
        query(json!({ "op": "ev_player_bed_allow", "allow": result == "ALLOW" }));
    }
    /// bukkit: org/bukkit/event/player/PlayerBedEnterEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerBedEnterEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerBedFailEnterEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerBedFailEnterEvent {
    value: Value,
    cancelled: bool,
    line: Option<crate::Component>,
    explode: Option<bool>,
}

hooked!(PlayerBedFailEnterEvent, "pl_bed", ["pl_packet", "pl_bed_blast"], |value| value["result"].is_string() && value["result"] != "OK");

impl PlayerBedFailEnterEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerBedFailEnterEvent.getBed
    pub fn get_bed(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerBedFailEnterEvent.getFailReason
    ///
    /// `FailReason`'s name.
    pub fn get_fail_reason(&self) -> &str {
        self.value["result"].as_str().unwrap_or_default()
    }
    /// bukkit: io/papermc/paper/event/player/PlayerBedFailEnterEvent.getWillExplode
    ///
    /// A bed outside the overworld blows up.
    pub fn get_will_explode(&self) -> bool {
        self.explode.unwrap_or(self.value["result"] == "NOT_POSSIBLE_HERE")
    }
    /// bukkit: io/papermc/paper/event/player/PlayerBedFailEnterEvent.setWillExplode
    pub fn set_will_explode(&mut self, explode: bool) {
        self.explode = Some(explode);
        query(json!({ "op": "ev_player_bed_blast", "explode": explode }));
    }
    /// bukkit: io/papermc/paper/event/player/PlayerBedFailEnterEvent.getMessage
    ///
    /// What the game will tell them, unless `set_message` changes it.
    pub fn get_message(&self) -> Option<crate::Component> {
        self.line.clone().or_else(|| bed_action(&self.value).error_message())
    }
    /// bukkit: io/papermc/paper/event/player/PlayerBedFailEnterEvent.setMessage
    pub fn set_message(&mut self, message: &crate::Component) {
        self.line = Some(message.clone());
        query(json!({ "op": "ev_player_bed_line", "line": message.to_legacy() }));
    }
    /// bukkit: io/papermc/paper/event/player/PlayerBedFailEnterEvent.enterAction
    pub fn enter_action(&self) -> BedAction {
        bed_action(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerBedFailEnterEvent.setCancelled
    ///
    /// They hear nothing about it.
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerBedFailEnterEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Furnaces, brewing ----

/// bukkit: org/bukkit/event/inventory/FurnaceBurnEvent
#[derive(Clone, Debug, Default)]
pub struct FurnaceBurnEvent {
    value: Value,
    cancelled: bool,
    burn: Option<i32>,
    keep: bool,
    not_burning: bool,
}

impl Event for FurnaceBurnEvent {
    const HOOK: &'static str = "pl_furnace";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "burn").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
    fn finish(&self) {
        if self.burn.is_some() || self.keep {
            query(json!({ "op": "ev_player_burn", "burn": self.burn, "keep": self.keep }));
        }
    }
}

impl FurnaceBurnEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceBurnEvent.getFuel
    pub fn get_fuel(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["fuel"])
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceBurnEvent.getBurnTime
    pub fn get_burn_time(&self) -> i32 {
        self.burn.unwrap_or(self.value["burn_time"].as_i64().unwrap_or(0) as i32)
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceBurnEvent.setBurnTime
    pub fn set_burn_time(&mut self, time: i32) {
        self.burn = Some(time);
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceBurnEvent.willConsumeFuel
    pub fn will_consume_fuel(&self) -> bool {
        !self.keep
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceBurnEvent.setConsumeFuel
    pub fn set_consume_fuel(&mut self, consume: bool) {
        self.keep = !consume;
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceBurnEvent.isBurning
    pub fn is_burning(&self) -> bool {
        self.get_burn_time() > 0 && !self.not_burning
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceBurnEvent.setBurning
    ///
    /// `false` keeps the fuel: the furnace lights without using it, as in Bukkit.
    pub fn set_burning(&mut self, burning: bool) {
        self.not_burning = !burning;
        self.keep = !burning;
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceBurnEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceBurnEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/inventory/FurnaceStartSmeltEvent
#[derive(Clone, Debug, Default)]
pub struct FurnaceStartSmeltEvent {
    value: Value,
    cancelled: bool,
    total: Option<i32>,
}

impl Event for FurnaceStartSmeltEvent {
    const HOOK: &'static str = "pl_furnace";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "start").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
    }
    fn finish(&self) {
        if let Some(total) = self.total {
            query(json!({ "op": "ev_player_burn", "cook": total }));
        }
    }
}

impl FurnaceStartSmeltEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    pub fn get_source(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["source"])
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceStartSmeltEvent.getRecipe
    pub fn get_recipe(&self) -> Option<crate::core2::GameRecipe> {
        crate::core2::get_recipe(&NamespacedKey::from_string(self.value["recipe"].as_str()?)?)
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceStartSmeltEvent.getTotalCookTime
    pub fn get_total_cook_time(&self) -> i32 {
        self.total.unwrap_or(self.value["cook_time"].as_i64().unwrap_or(0) as i32)
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceStartSmeltEvent.setTotalCookTime
    pub fn set_total_cook_time(&mut self, total: i32) {
        self.total = Some(total);
    }
}

/// bukkit: org/bukkit/event/inventory/BrewEvent
///
/// As the time runs out. A cancel starts the brew over.
#[derive(Clone, Debug, Default)]
pub struct BrewEvent {
    value: Value,
    cancelled: bool,
}

hooked!(BrewEvent, "pl_brew", |value| value["kind"] == "brew");

impl BrewEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/inventory/BrewEvent.getContents
    pub fn get_contents(&self) -> MenuInventory {
        MenuInventory::read(&self.value)
    }
    /// bukkit: org/bukkit/event/inventory/BrewEvent.getFuelLevel
    pub fn get_fuel_level(&self) -> i32 {
        self.value["fuel_level"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/inventory/BrewEvent.getResults
    ///
    /// What the three bottles become.
    pub fn get_results(&self) -> Vec<Option<ItemStack>> {
        stacks_of(&self.value["results"])
    }
    /// Java's list is live: put a changed one back with this.
    pub fn set_results(&mut self, results: &[Option<ItemStack>]) {
        self.value["results"] = results.iter().map(|stack| stack.as_ref().map_or(Value::Null, ItemStack::to_json)).collect();
        query(json!({ "op": "ev_player_brewed", "stacks": results.iter().map(|stack| snbt_of(stack.as_ref())).collect::<Vec<_>>() }));
    }
    /// bukkit: org/bukkit/event/inventory/BrewEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/inventory/BrewEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/inventory/BrewingStandFuelEvent
#[derive(Clone, Debug, Default)]
pub struct BrewingStandFuelEvent {
    value: Value,
    cancelled: bool,
    power: Option<i32>,
    keep: bool,
}

impl Event for BrewingStandFuelEvent {
    const HOOK: &'static str = "pl_brew";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "fuel").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
    fn finish(&self) {
        if self.power.is_some() || self.keep {
            query(json!({ "op": "ev_player_burn", "burn": self.power, "keep": self.keep }));
        }
    }
}

impl BrewingStandFuelEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/inventory/BrewingStandFuelEvent.getFuel
    pub fn get_fuel(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["fuel"])
    }
    /// bukkit: org/bukkit/event/inventory/BrewingStandFuelEvent.getFuelPower
    pub fn get_fuel_power(&self) -> i32 {
        self.power.unwrap_or(self.value["power"].as_i64().unwrap_or(20) as i32)
    }
    /// bukkit: org/bukkit/event/inventory/BrewingStandFuelEvent.setFuelPower
    pub fn set_fuel_power(&mut self, power: i32) {
        self.power = Some(power);
    }
    /// bukkit: org/bukkit/event/inventory/BrewingStandFuelEvent.isConsuming
    pub fn is_consuming(&self) -> bool {
        !self.keep
    }
    /// bukkit: org/bukkit/event/inventory/BrewingStandFuelEvent.setConsuming
    pub fn set_consuming(&mut self, consuming: bool) {
        self.keep = !consuming;
    }
    /// bukkit: org/bukkit/event/inventory/BrewingStandFuelEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/inventory/BrewingStandFuelEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/inventory/FurnaceExtractEvent
///
/// A click that takes from a furnace's result.
#[derive(Clone, Debug, Default)]
pub struct FurnaceExtractEvent {
    value: Value,
    cancelled: bool,
}

held_event!(FurnaceExtractEvent, |value| value["kind"] == "click"
    && value["slot"] == 2
    && matches!(value["type"].as_str(), Some("FURNACE" | "BLAST_FURNACE" | "SMOKER"))
    && !value["current"].is_null()
    && matches!(value["input"].as_i64(), Some(0 | 1)));

impl FurnaceExtractEvent {
    /// bukkit: org/bukkit/event/inventory/FurnaceExtractEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceExtractEvent.getItemStack
    pub fn get_item_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["current"])
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceExtractEvent.getItemType
    pub fn get_item_type(&self) -> Option<crate::Material> {
        self.get_item_stack().map(|stack| stack.get_type().clone())
    }
    /// bukkit: org/bukkit/event/inventory/FurnaceExtractEvent.getItemAmount
    pub fn get_item_amount(&self) -> i32 {
        self.get_item_stack().map_or(0, |stack| stack.get_amount())
    }
}

/// bukkit: org/bukkit/event/inventory/InventoryCloseEvent
///
/// A player closing a menu of any kind, or a GUI from `Bukkit.createInventory`.
#[derive(Clone, Debug, Default)]
pub struct InventoryCloseEvent {
    value: Value,
    cancelled: bool,
    gui: Option<Inventory>,
}

impl Event for InventoryCloseEvent {
    const HOOK: &'static str = "pl_packet";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_held", "pl_tables", "pl_stations", "pl_trades", "menu_close", "pl_closing", "pl_closing_table", "pl_closing_station"]
    }
    fn read(value: &Value) -> Option<Self> {
        if value["kind"] == "close" {
            return Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() });
        }
        if value.get("kind").is_some() || value.get("window").is_none() {
            return None;
        }
        let gui = OPEN_INVENTORIES.lock().unwrap_or_else(|e| e.into_inner()).remove(value["uuid"].as_str()?)?;
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), gui: Some(gui) })
    }
}

impl InventoryCloseEvent {
    /// bukkit: org/bukkit/event/inventory/InventoryCloseEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getInventory
    pub fn get_inventory(&self) -> MenuInventory {
        let mut inventory = MenuInventory::read(&self.value);
        if let Some(gui) = &self.gui {
            inventory.size = gui.get_size();
            inventory.kind = "CHEST".to_owned();
            inventory.items = (0..gui.get_size()).map(|slot| gui.get_item(slot).cloned()).collect();
            inventory.gui = Some(gui.clone());
        }
        inventory
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: player_of(&self.value), top: self.get_inventory() }
    }
    /// bukkit: org/bukkit/event/inventory/InventoryCloseEvent.getReason
    ///
    /// `Reason`'s name.
    pub fn get_reason(&self) -> &str {
        self.value["reason"].as_str().unwrap_or("PLAYER")
    }
}

// ---- More packets ----

/// bukkit: org/bukkit/event/player/PlayerEditBookEvent
///
/// The book and quill's Done or Sign. Pages are plain text here, Bukkit's `BookMeta` as the two lists.
#[derive(Clone, Debug, Default)]
pub struct PlayerEditBookEvent {
    value: Value,
    cancelled: bool,
    meta: Option<crate::meta::BookMeta>,
}

held_event!(PlayerEditBookEvent, |value| value["kind"] == "edit_book");

impl PlayerEditBookEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerEditBookEvent.getSlot
    pub fn get_slot(&self) -> i64 {
        self.value["slot"].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/event/player/PlayerEditBookEvent.getPreviousBookMeta
    pub fn get_previous_book_meta(&self) -> Option<crate::meta::BookMeta> {
        ItemStack::read(&self.value["previous"]).map(crate::meta::BookMeta)
    }
    fn new_pages(&self) -> Vec<String> {
        self.value["pages"].as_array().into_iter().flatten().filter_map(|page| page.as_str().map(str::to_owned)).collect()
    }
    /// bukkit: org/bukkit/event/player/PlayerEditBookEvent.getNewBookMeta
    ///
    /// The book as it will be: the old one with these pages and, signed, the title and its author.
    pub fn get_new_book_meta(&self) -> Option<crate::meta::BookMeta> {
        if self.meta.is_some() {
            return self.meta.clone();
        }
        let mut meta = self.get_previous_book_meta()?;
        let pages = self.new_pages();
        meta.set_pages(&pages.iter().map(String::as_str).collect::<Vec<_>>());
        if let Some(title) = self.value["title"].as_str() {
            meta.set_title(title);
            meta.set_author(self.value["name"].as_str().unwrap_or_default());
            meta.set_generation_number(0);
        }
        Some(meta)
    }
    /// bukkit: org/bukkit/event/player/PlayerEditBookEvent.setNewBookMeta
    pub fn set_new_book_meta(&mut self, meta: &crate::meta::BookMeta) {
        let pages = meta.get_pages();
        let title = if self.is_signing() { meta.get_title().or_else(|| self.value["title"].as_str().map(str::to_owned)) } else { None };
        self.set_book(&pages, title.as_deref());
        // Everything past the pages and the title goes on the book the game writes.
        let mut rest = meta.0.clone();
        rest.remove_component("minecraft:writable_book_content");
        rest.remove_component("minecraft:written_book_content");
        let tags: Vec<Value> = meta.pages().into_iter().map(|page| page.0).collect();
        query(json!({ "op": "ev_player_book_meta", "stack": snbt_of(Some(&rest)), "author": meta.get_author(), "generation": meta.get_generation_number(), "tags": tags }));
        self.meta = Some(meta.clone());
    }
    fn set_book(&mut self, pages: &[String], title: Option<&str>) {
        self.value["pages"] = json!(pages);
        self.value["title"] = json!(title);
        query(json!({ "op": "ev_player_book", "pages": pages, "title": title }));
    }
    /// bukkit: org/bukkit/event/player/PlayerEditBookEvent.isSigning
    pub fn is_signing(&self) -> bool {
        self.value["title"].is_string()
    }
    /// bukkit: org/bukkit/event/player/PlayerEditBookEvent.setSigning
    pub fn set_signing(&mut self, signing: bool) {
        if signing != self.is_signing() {
            let pages = self.new_pages();
            self.set_book(&pages, signing.then_some(""));
        }
    }
    /// bukkit: org/bukkit/event/player/PlayerEditBookEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerEditBookEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/packet/UncheckedSignChangeEvent
///
/// What a client wrote on a sign that isn't in the world: a plugin's fake one.
#[derive(Clone, Debug, Default)]
pub struct UncheckedSignChangeEvent {
    value: Value,
    cancelled: bool,
}

held_event!(UncheckedSignChangeEvent, |value| value["kind"] == "unchecked_sign");

impl UncheckedSignChangeEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/packet/UncheckedSignChangeEvent.getEditedBlockPosition
    pub fn get_edited_block_position(&self) -> Option<crate::pure::Position> {
        let at = |i: usize| self.value["pos"][i].as_i64().map(|v| v as i32);
        Some(crate::pure::Position::block(at(0)?, at(1)?, at(2)?))
    }
    /// bukkit: io/papermc/paper/event/packet/UncheckedSignChangeEvent.lines
    pub fn lines(&self) -> Vec<crate::Component> {
        self.value["lines"].as_array().into_iter().flatten().filter_map(|line| line.as_str().map(crate::Component::text)).collect()
    }
    /// bukkit: io/papermc/paper/event/packet/UncheckedSignChangeEvent.getSide
    ///
    /// `Side`'s name.
    pub fn get_side(&self) -> &'static str {
        if self.value["front"] == true { "FRONT" } else { "BACK" }
    }
    /// bukkit: io/papermc/paper/event/packet/UncheckedSignChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/packet/UncheckedSignChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerCustomClickEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerCustomClickEvent {
    value: Value,
    cancelled: bool,
}

held_event!(PlayerCustomClickEvent, |value| value["kind"] == "custom_click");

impl PlayerCustomClickEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    ///
    /// `None` for someone a dialog holds in configuration, who isn't a player yet.
    pub fn get_player(&self) -> Option<Player> {
        if self.value["configuring"] == true {
            return None;
        }
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerCustomClickEvent.getIdentifier
    pub fn get_identifier(&self) -> Option<NamespacedKey> {
        NamespacedKey::from_string(self.value["id"].as_str()?)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerCustomClickEvent.getTag
    ///
    /// As SNBT, as Java's `BinaryTagHolder` holds it.
    pub fn get_tag(&self) -> Option<String> {
        (!self.value["tag"].is_null()).then(|| crate::data::snbt(&self.value["tag"]))
    }
    /// bukkit: io/papermc/paper/event/player/PlayerCustomClickEvent.getDialogResponseView
    ///
    /// What a dialog sent.
    pub fn get_dialog_response_view(&self) -> Option<crate::fam_paper::DialogResponseView> {
        self.value["tag"].is_object().then(|| crate::fam_paper::DialogResponseView(self.value["tag"].clone()))
    }
    /// bukkit: io/papermc/paper/event/player/PlayerCustomClickEvent.getCommonConnection
    pub fn get_common_connection(&self) -> Option<crate::fam_paper::PlayerCommonConnection> {
        if self.value["configuring"] == true {
            return Some(crate::fam_paper::PlayerCommonConnection::Configuration(crate::fam_paper::PlayerConfigurationConnection(self.value.clone())));
        }
        self.get_player().map(|player| crate::fam_paper::PlayerCommonConnection::Game(crate::fam_paper::PlayerGameConnection(player)))
    }
}

/// bukkit: org/bukkit/event/player/PlayerRecipeDiscoverEvent
///
/// One recipe the book has just taken: a cancel takes it out again before the player hears of it.
#[derive(Clone, Debug, Default)]
pub struct PlayerRecipeDiscoverEvent {
    player: Option<Player>,
    recipe: String,
    cancelled: bool,
}

impl Event for PlayerRecipeDiscoverEvent {
    const HOOK: &'static str = "pl_recipes";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { player: player_of(value), recipe: value["recipe"].as_str()?.to_owned(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

impl PlayerRecipeDiscoverEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<&Player> {
        self.player.as_ref()
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeDiscoverEvent.getRecipe
    pub fn get_recipe(&self) -> Option<NamespacedKey> {
        NamespacedKey::from_string(&self.recipe)
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeDiscoverEvent.shouldShowNotification
    pub fn should_show_notification(&self) -> bool {
        true
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeDiscoverEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeDiscoverEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerKickEvent
///
/// Every kick, at the turn after the hub or a command asked for it. A cancel keeps them on.
#[derive(Clone, Debug, Default)]
pub struct PlayerKickEvent {
    value: Value,
    cancelled: bool,
    reason: Option<String>,
    leave: Option<Option<String>>,
}

impl Event for PlayerKickEvent {
    const HOOK: &'static str = "pl_kick";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_kick_note", "pl_moves", "pl_move_turn"]
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

impl PlayerKickEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerKickEvent.getReason
    pub fn get_reason(&self) -> &str {
        self.reason.as_deref().unwrap_or(self.value["reason"].as_str().unwrap_or_default())
    }
    /// bukkit: org/bukkit/event/player/PlayerKickEvent.getCause
    ///
    /// `Cause`'s name.
    pub fn get_cause(&self) -> &str {
        self.value["cause"].as_str().unwrap_or("UNKNOWN")
    }
    /// bukkit: org/bukkit/event/player/PlayerKickEvent.getLeaveMessage
    pub fn get_leave_message(&self) -> String {
        match &self.leave {
            Some(text) => text.clone().unwrap_or_default(),
            None => format!("{} left the game", self.value["name"].as_str().unwrap_or_default()),
        }
    }
    /// bukkit: org/bukkit/event/player/PlayerKickEvent.setLeaveMessage
    ///
    /// What's said as they go: `None` says nothing.
    pub fn set_leave_message(&mut self, text: Option<&str>) {
        self.leave = Some(text.map(str::to_owned));
        crate::event::line_set(1, text);
    }
    /// bukkit: org/bukkit/event/player/PlayerKickEvent.leaveMessage
    pub fn leave_message(&self) -> crate::Component {
        crate::adventure::legacy(&self.get_leave_message())
    }
    /// bukkit: org/bukkit/event/player/PlayerKickEvent.setReason
    pub fn set_reason(&mut self, reason: &str) {
        self.reason = Some(reason.to_owned());
        query(json!({ "op": "ev_player_kick_reason", "line": reason }));
    }
    /// bukkit: org/bukkit/event/player/PlayerKickEvent.reason
    pub fn reason(&self) -> crate::Component {
        crate::adventure::legacy(self.get_reason())
    }
    /// bukkit: org/bukkit/event/player/PlayerKickEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerKickEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerPostRespawnEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerPostRespawnEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerPostRespawnEvent, "respawn", |_value| true);

impl PlayerPostRespawnEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerPostRespawnEvent.getRespawnedLocation
    pub fn get_respawned_location(&self) -> Option<Location> {
        self.get_player().map(|player| player.get_location())
    }
}

/// What the game found for this player's respawn.
fn respawn_facts(player: &Player) -> Value {
    query(json!({ "op": "ev_player_respawn", "uuid": player.uuid() }))
}

impl crate::event::PlayerQuitEvent {
    /// bukkit: org/bukkit/event/player/PlayerQuitEvent.getReason
    ///
    /// `QuitReason`'s name, from what the hub said as they went.
    pub fn get_reason(&self) -> &'static str {
        let reason = query(json!({ "op": "ev_player_left", "uuid": self.player.uuid() }));
        match reason.as_str().unwrap_or_default() {
            "" | "Disconnected" => "DISCONNECTED",
            "Timed out" => "TIMED_OUT",
            text if text.starts_with("Internal Exception") || text.contains("nvalid") => "ERRONEOUS_STATE",
            _ => "KICKED",
        }
    }
}

/// bukkit: org/bukkit/event/player/PlayerRespawnEvent
///
/// Before the respawn, with where they'd go worked out. A player who leaves the End comes back
/// through the game's own respawn, and a changed location then teleports them.
pub struct PlayerRespawnEvent {
    pub player: Player,
    value: Value,
    to: Option<Location>,
}

impl Event for PlayerRespawnEvent {
    const HOOK: &'static str = "pl_respawn_ask";
    fn more_hooks() -> &'static [&'static str] {
        &["respawn"]
    }
    fn read(value: &Value) -> Option<Self> {
        let player = player_of(value)?;
        if value["to"].is_null() && query(json!({ "op": "ev_player_respawn_asked", "uuid": player.uuid() })) == true {
            return None;
        }
        Some(Self { player, value: value.clone(), to: None })
    }
    fn finish(&self) {
        let Some(to) = &self.to else {
            return;
        };
        if self.value["to"].is_null() {
            self.player.teleport(to);
        } else {
            query(json!({ "op": "ev_player_respawn_to", "to": place_json(to), "level": to.world.dimension() }));
        }
    }
}

impl PlayerRespawnEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    pub fn get_respawn_location(&self) -> Location {
        self.to.clone().or_else(|| place_of(&self.value, "to")).unwrap_or_else(|| self.player.get_location())
    }
    /// bukkit~: org/bukkit/event/player/PlayerRespawnEvent.setRespawnLocation | from the End the player respawns and is then teleported, and a place in another level has no trip holding them
    pub fn set_respawn_location(&mut self, to: &Location) {
        self.to = Some(to.clone());
    }
    /// bukkit: io/papermc/paper/event/player/AbstractRespawnEvent.isBedSpawn
    pub fn is_bed_spawn(&self) -> bool {
        let facts = respawn_facts(&self.player);
        facts["block"] == "BED" && facts["missing"] != true
    }
    /// bukkit: io/papermc/paper/event/player/AbstractRespawnEvent.isAnchorSpawn
    pub fn is_anchor_spawn(&self) -> bool {
        let facts = respawn_facts(&self.player);
        facts["block"] == "ANCHOR" && facts["missing"] != true
    }
    /// bukkit: io/papermc/paper/event/player/AbstractRespawnEvent.isMissingRespawnBlock
    ///
    /// They had a bed or anchor, and it's gone or blocked.
    pub fn is_missing_respawn_block(&self) -> bool {
        respawn_facts(&self.player)["missing"] == true
    }
    /// bukkit: io/papermc/paper/event/player/AbstractRespawnEvent.getRespawnFlags
    ///
    /// `RespawnFlag`'s names.
    pub fn get_respawn_flags(&self) -> Vec<&'static str> {
        let facts = respawn_facts(&self.player);
        let mut flags = Vec::new();
        if facts["missing"] != true && facts["block"] == "BED" {
            flags.push("BED_SPAWN");
        }
        if facts["missing"] != true && facts["block"] == "ANCHOR" {
            flags.push("ANCHOR_SPAWN");
        }
        if facts["end"] == true {
            flags.push("END_PORTAL");
        }
        flags
    }
    /// bukkit: io/papermc/paper/event/player/AbstractRespawnEvent.getRespawnReason
    pub fn get_respawn_reason(&self) -> &'static str {
        let facts = respawn_facts(&self.player);
        if facts["plugin"] == true {
            "PLUGIN"
        } else if facts["end"] == true {
            "END_PORTAL"
        } else {
            "DEATH"
        }
    }
    /// bukkit: io/papermc/paper/event/player/AbstractRespawnEvent.getRespawnLocation
    pub fn respawn_location(&self) -> Location {
        self.get_respawn_location()
    }
}

impl crate::event::PlayerCommandPreprocessEvent {
    /// bukkit: org/bukkit/event/player/PlayerCommandPreprocessEvent.setMessage
    ///
    /// The command that runs, unsigned, in place of what they sent.
    pub fn set_message(&mut self, message: &str) {
        query(json!({ "op": "ev_player_command_to", "from": self.message, "to": message }));
        self.message = message.to_owned();
        crate::event::carry("message", json!(message));
    }
    /// bukkit: org/bukkit/event/player/PlayerCommandPreprocessEvent.setPlayer
    pub fn set_player(&mut self, player: &Player) {
        query(json!({ "op": "ev_player_command_as", "by": self.player.uuid(), "as": player.uuid() }));
        self.player = player.clone();
        crate::event::carry("uuid", json!(player.uuid()));
        crate::event::carry("name", json!(player.get_name()));
    }
    /// bukkit: org/bukkit/event/player/PlayerCommandPreprocessEvent.getRecipients
    pub fn get_recipients(&self) -> Vec<Player> {
        crate::get_online_players()
    }
}

/// A block's plain name, on a click.
fn clicked_name(value: &Value) -> Option<(Block, String)> {
    let player = player_of(value)?;
    let pos = &value["pos"];
    let at = |i: usize| pos[i].as_i64().map(|v| v as i32);
    let block = player.get_world().get_block_at(at(0)?, at(1)?, at(2)?);
    let name = block.get_type().key().trim_start_matches("minecraft:").to_owned();
    Some((block, name))
}

/// bukkit: io/papermc/paper/event/player/PlayerFlowerPotManipulateEvent
///
/// A flower going in or coming out of a pot, by a player's click.
#[derive(Clone, Debug, Default)]
pub struct PlayerFlowerPotManipulateEvent {
    value: Value,
    cancelled: bool,
    block: Option<Block>,
    placing: bool,
}

impl Event for PlayerFlowerPotManipulateEvent {
    const HOOK: &'static str = "block_use_item";
    fn more_hooks() -> &'static [&'static str] {
        &["block_use"]
    }
    fn read(value: &Value) -> Option<Self> {
        let (block, name) = clicked_name(value)?;
        let held = ItemStack::read(&value["stack"]);
        let placing = name == "flower_pot";
        if placing {
            let held = held?;
            let potted = format!("minecraft:potted_{}", held.get_type().key().trim_start_matches("minecraft:"));
            if query(json!({ "op": "ev_player_is_block", "name": potted })) != true {
                return None;
            }
        } else if !name.starts_with("potted_") {
            return None;
        }
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), block: Some(block), placing })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

impl PlayerFlowerPotManipulateEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFlowerPotManipulateEvent.getFlowerpot
    pub fn get_flowerpot(&self) -> Option<&Block> {
        self.block.as_ref()
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFlowerPotManipulateEvent.isPlacing
    pub fn is_placing(&self) -> bool {
        self.placing
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFlowerPotManipulateEvent.getItem
    ///
    /// The flower: in the hand, or in the pot.
    pub fn get_item(&self) -> Option<ItemStack> {
        if self.placing {
            return ItemStack::read(&self.value["stack"]);
        }
        let name = self.block.as_ref()?.get_type().key().replacen("potted_", "", 1);
        Some(ItemStack::new(crate::Material::of(&name), 1))
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFlowerPotManipulateEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFlowerPotManipulateEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerHarvestBlockEvent
///
/// Sweet berries and glow berries by a click.
#[derive(Clone, Debug, Default)]
pub struct PlayerHarvestBlockEvent {
    value: Value,
    cancelled: bool,
    block: Option<Block>,
    items: Vec<ItemStack>,
    rolled: Vec<ItemStack>,
}

impl Event for PlayerHarvestBlockEvent {
    const HOOK: &'static str = "block_use";
    fn more_hooks() -> &'static [&'static str] {
        &["block_use_item"]
    }
    fn read(value: &Value) -> Option<Self> {
        let (block, name) = clicked_name(value)?;
        let ripe = match name.as_str() {
            "sweet_berry_bush" => block.get_property("age").is_some_and(|age| age == "2" || age == "3"),
            "cave_vines" | "cave_vines_plant" => block.get_property("berries").as_deref() == Some("true"),
            _ => false,
        };
        if !ripe {
            return None;
        }
        let rolled = stacks_of(&query(json!({ "op": "ev_player_harvest", "pos": value["pos"] }))).into_iter().flatten().collect::<Vec<_>>();
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), block: Some(block), items: rolled.clone(), rolled })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
    fn finish(&self) {
        if self.cancelled {
            query(json!({ "op": "ev_player_harvest_set", "pos": self.value["pos"], "drops": Value::Null }));
        } else if self.items != self.rolled {
            let drops: Vec<Value> = self.items.iter().map(|stack| json!(stack.to_snbt())).collect();
            query(json!({ "op": "ev_player_harvest_set", "pos": self.value["pos"], "drops": drops }));
        }
    }
}

impl PlayerHarvestBlockEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerHarvestBlockEvent.getHarvestedBlock
    pub fn get_harvested_block(&self) -> Option<&Block> {
        self.block.as_ref()
    }
    /// bukkit: org/bukkit/event/player/PlayerHarvestBlockEvent.getItemsHarvested
    ///
    /// What the click will give: change it here.
    pub fn get_items_harvested(&mut self) -> &mut Vec<ItemStack> {
        &mut self.items
    }
    /// bukkit: org/bukkit/event/player/PlayerHarvestBlockEvent.getHand
    ///
    /// The hand the game is on.
    pub fn get_hand(&self) -> String {
        query(json!({ "op": "ev_player_acting" }))["hand"].as_str().unwrap_or("HAND").to_owned()
    }
    /// bukkit: org/bukkit/event/player/PlayerHarvestBlockEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerHarvestBlockEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerInventorySlotChangeEvent
///
/// The 36 slots, compared as the game looks for inventory triggers: as Paper's menu compares its
/// slots when it broadcasts, two changes between looks read as one.
#[derive(Clone, Debug, Default)]
pub struct PlayerInventorySlotChangeEvent {
    value: Value,
    trigger: Option<bool>,
}

impl Event for PlayerInventorySlotChangeEvent {
    const HOOK: &'static str = "pl_slots";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), trigger: None })
    }
}

impl PlayerInventorySlotChangeEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerInventorySlotChangeEvent.getSlot
    pub fn get_slot(&self) -> i32 {
        self.value["slot"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: io/papermc/paper/event/player/PlayerInventorySlotChangeEvent.getRawSlot
    ///
    /// In the player's inventory menu's numbering.
    pub fn get_raw_slot(&self) -> i32 {
        let slot = self.get_slot();
        if slot < 9 { slot + 36 } else { slot }
    }
    /// bukkit: io/papermc/paper/event/player/PlayerInventorySlotChangeEvent.getOldItemStack
    pub fn get_old_item_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["old"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerInventorySlotChangeEvent.getNewItemStack
    pub fn get_new_item_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["new"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerInventorySlotChangeEvent.shouldTriggerAdvancements
    pub fn should_trigger_advancements(&self) -> bool {
        self.trigger.unwrap_or(true)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerInventorySlotChangeEvent.setShouldTriggerAdvancements
    ///
    /// With `false` for every slot that changed, the inventory trigger doesn't run for this change.
    pub fn set_should_trigger_advancements(&mut self, trigger: bool) {
        self.trigger = Some(trigger);
        query(json!({ "op": "ev_player_slot_trigger", "trigger": trigger }));
    }
}

// ---- More of the same ----

/// bukkit: org/bukkit/event/player/PlayerAnimationEvent, io/papermc/paper/event/player/PlayerArmSwingEvent
///
/// A client swinging the main arm. A cancel means the others don't see it.
#[derive(Clone, Debug, Default)]
pub struct PlayerAnimationEvent {
    value: Value,
    cancelled: bool,
}

held_event!(PlayerAnimationEvent, |value| value["kind"] == "swing");

impl PlayerAnimationEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerAnimationEvent.getAnimationType
    ///
    /// `PlayerAnimationType`'s name.
    pub fn get_animation_type(&self) -> &'static str {
        "ARM_SWING"
    }
    /// `EquipmentSlot`'s name: the game swings the main hand here.
    pub fn get_hand(&self) -> &'static str {
        "HAND"
    }
    /// bukkit: org/bukkit/event/player/PlayerAnimationEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerAnimationEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

fn trade_click(value: &Value) -> bool {
    value["kind"] == "click" && value["type"] == "MERCHANT" && value["slot"] == 2 && !value["current"].is_null() && matches!(value["input"].as_i64(), Some(0 | 1))
}

/// bukkit: io/papermc/paper/event/player/PlayerPurchaseEvent.getTrade, io/papermc/paper/event/player/PlayerPurchaseEvent.setTrade, io/papermc/paper/event/player/PlayerPurchaseEvent.isRewardingExp, io/papermc/paper/event/player/PlayerPurchaseEvent.setRewardExp, io/papermc/paper/event/player/PlayerPurchaseEvent.willIncreaseTradeUses, io/papermc/paper/event/player/PlayerPurchaseEvent.setIncreaseTradeUses, io/papermc/paper/event/player/PlayerPurchaseEvent.isCancelled, io/papermc/paper/event/player/PlayerPurchaseEvent.setCancelled
macro_rules! trade_event {
    ($name:ident, |$value:ident| $wanted:expr) => {
        impl Event for $name {
            const HOOK: &'static str = "pl_packet";
            fn more_hooks() -> &'static [&'static str] {
                HELD_HOOKS
            }
            fn read($value: &Value) -> Option<Self> {
                (trade_click($value) && $wanted).then(|| Self { value: $value.clone(), cancelled: crate::event::prior_cancelled(), uses: true })
            }
            fn answer(&self) -> Answer {
                answer_of(self.cancelled)
            }
        }
        impl $name {
            pub fn get_player(&self) -> Option<Player> {
                player_of(&self.value)
            }
            /// The offer being bought.
            pub fn get_trade(&self) -> crate::core2::MerchantRecipe {
                crate::core2::MerchantRecipe(self.value["offer"].clone())
            }
            pub fn set_trade(&mut self, trade: &crate::core2::MerchantRecipe) {
                self.value["offer"] = trade.0.clone();
                self.send();
            }
            pub fn is_rewarding_exp(&self) -> bool {
                self.value["offer"]["rewardExp"] == true
            }
            pub fn set_reward_exp(&mut self, reward: bool) {
                self.value["offer"]["rewardExp"] = json!(reward);
                self.send();
            }
            pub fn will_increase_trade_uses(&self) -> bool {
                self.uses
            }
            pub fn set_increase_trade_uses(&mut self, uses: bool) {
                self.uses = uses;
                self.send();
            }
            fn send(&self) {
                query(json!({ "op": "ev_player_trade", "offer": self.value["offer"], "reward": self.value["offer"]["rewardExp"], "uses": self.uses }));
            }
            pub fn set_cancelled(&mut self, cancelled: bool) {
                self.cancelled = cancelled;
            }
            pub fn is_cancelled(&self) -> bool {
                self.cancelled
            }
        }
    };
}

/// bukkit: io/papermc/paper/event/player/PlayerTradeEvent
///
/// A click on a villager's result, before it's paid for. A cancel keeps both sides as they were.
#[derive(Clone, Debug)]
pub struct PlayerTradeEvent {
    value: Value,
    cancelled: bool,
    uses: bool,
}

trade_event!(PlayerTradeEvent, |value| !value["merchant"].is_null());

impl PlayerTradeEvent {
    /// bukkit: io/papermc/paper/event/player/PlayerTradeEvent.getVillager
    pub fn get_villager(&self) -> Option<Entity> {
        Some(Entity::of(self.value["merchant"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: io/papermc/paper/event/player/PlayerTradeEvent.getMerchant
    pub fn get_merchant(&self) -> Option<Entity> {
        self.get_villager()
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerPurchaseEvent
///
/// The same click on a plugin's own merchant, which no villager stands behind.
#[derive(Clone, Debug)]
pub struct PlayerPurchaseEvent {
    value: Value,
    cancelled: bool,
    uses: bool,
}

trade_event!(PlayerPurchaseEvent, |value| value["merchant"].is_null());

impl PlayerPurchaseEvent {
    /// bukkit: io/papermc/paper/event/player/PlayerPurchaseEvent.getMerchant
    ///
    /// The merchant's offers as the window has them.
    pub fn get_merchant(&self) -> crate::fam_core::VirtualMerchant {
        let mut merchant = crate::fam_core::create_merchant(None);
        merchant.set_recipes(self.value["offers"].as_array().into_iter().flatten().cloned().map(crate::core2::MerchantRecipe).collect());
        merchant
    }
}

/// bukkit: org/bukkit/event/player/PlayerDropItemEvent
///
/// Whatever a player drops, once it's in the world. A cancel takes it back.
#[derive(Clone, Debug, Default)]
pub struct PlayerDropItemEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerDropItemEvent, "pl_drop", |_value| true);

impl PlayerDropItemEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// The stack that was dropped. (Java's `getItemDrop()` is the dropped Item entity, `get_item_entity`; this one
    /// carried that tag too, and a converted plugin's `getItemDrop().getItemStack()` got this ItemStack.)
    pub fn get_item_drop(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: org/bukkit/event/player/PlayerDropItemEvent.getItemDrop
    pub fn get_item_entity(&self) -> Option<crate::entities::Item> {
        Some(crate::entities::Item(Entity::of(self.value["entity"].as_u64()?, &world_of(&self.value))))
    }
    /// bukkit: org/bukkit/event/player/PlayerDropItemEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerDropItemEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// Each player's bundle selection, as last told.
static BUNDLES: std::sync::Mutex<BTreeMap<String, i64>> = std::sync::Mutex::new(BTreeMap::new());

/// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent, io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent$Direction
#[derive(Clone, Debug, Default)]
pub struct PlayerBundleItemSelectEvent {
    value: Value,
    cancelled: bool,
    previous: i32,
}

impl Event for PlayerBundleItemSelectEvent {
    const HOOK: &'static str = "pl_packet";
    fn more_hooks() -> &'static [&'static str] {
        HELD_HOOKS
    }
    fn read(value: &Value) -> Option<Self> {
        if value["kind"] != "bundle" {
            return None;
        }
        let key = format!("{}#{}", value["uuid"].as_str()?, value["slot"]);
        let previous = BUNDLES.lock().unwrap_or_else(|e| e.into_inner()).insert(key, value["index"].as_i64()?).unwrap_or(-1);
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), previous: previous as i32 })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

impl PlayerBundleItemSelectEvent {
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent.getRawSlot
    pub fn get_raw_slot(&self) -> i32 {
        self.value["slot"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent.getSelectedIndex
    pub fn get_selected_index(&self) -> i32 {
        self.value["index"].as_i64().unwrap_or(-1) as i32
    }
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent.getPreviousIndex
    pub fn get_previous_index(&self) -> i32 {
        self.previous
    }
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent.getSlot
    ///
    /// The bundle's slot in the inventory it's in.
    pub fn get_slot(&self) -> i32 {
        let raw = self.get_raw_slot();
        if (36..45).contains(&raw) { raw - 36 } else { raw }
    }
    fn held(&self, index: i32) -> Option<ItemStack> {
        ItemStack::read(self.value["bundle"]["components"]["minecraft:bundle_contents"].get(usize::try_from(index).ok()?)?)
    }
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent.getSelectedItem
    pub fn get_selected_item(&self) -> Option<ItemStack> {
        self.held(self.get_selected_index())
    }
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent.getPreviousItem
    pub fn get_previous_item(&self) -> Option<ItemStack> {
        self.held(self.previous)
    }
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent.getDirection
    ///
    /// `Direction`'s name, from the old selection to the new: `FORWARD`, `BACKWARD` or `NONE`.
    pub fn get_direction(&self) -> &'static str {
        match self.get_selected_index() - self.previous {
            0 => "NONE",
            delta if delta > 0 => "FORWARD",
            _ => "BACKWARD",
        }
    }
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent$Direction.flip
    pub fn flip(direction: &str) -> &'static str {
        match direction {
            "FORWARD" => "BACKWARD",
            "BACKWARD" => "FORWARD",
            _ => "NONE",
        }
    }
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent$Direction.getDelta
    pub fn get_delta(direction: &str) -> Option<i32> {
        match direction {
            "FORWARD" => Some(1),
            "BACKWARD" => Some(-1),
            _ => None,
        }
    }
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent.getBundle
    pub fn get_bundle(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["bundle"])
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerSwapWithEquipmentSlotEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerSwapWithEquipmentSlotEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerSwapWithEquipmentSlotEvent, "pl_equip", |value| value["slot"].is_string());

impl PlayerSwapWithEquipmentSlotEvent {
    /// bukkit: io/papermc/paper/event/player/PlayerSwapWithEquipmentSlotEvent.getItemToSwap
    ///
    /// What's worn there now.
    pub fn get_item_to_swap(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["worn"])
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerSwapWithEquipmentSlotEvent.getItemInHand
    pub fn get_item_in_hand(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerSwapWithEquipmentSlotEvent.getSlot
    ///
    /// `EquipmentSlot`'s name.
    pub fn get_slot(&self) -> String {
        self.value["slot"].as_str().unwrap_or_default().to_uppercase()
    }
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: io/papermc/paper/event/player/PlayerSwapWithEquipmentSlotEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerSwapWithEquipmentSlotEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// `Statistic`'s name of a game statistic: a counter by its own name, the rest by what they count.
fn statistic_name(kind: &str, key: &str) -> String {
    let key = key.trim_start_matches("minecraft:");
    match kind.trim_start_matches("minecraft:") {
        "mined" => "MINE_BLOCK".to_owned(),
        "crafted" => "CRAFT_ITEM".to_owned(),
        "used" => "USE_ITEM".to_owned(),
        "broken" => "BREAK_ITEM".to_owned(),
        "picked_up" => "PICKUP".to_owned(),
        "dropped" => "DROP".to_owned(),
        "killed" => "KILL_ENTITY".to_owned(),
        "killed_by" => "ENTITY_KILLED_BY".to_owned(),
        _ => match key {
            "play_time" => "PLAY_ONE_MINUTE",
            "drop" => "DROP_COUNT",
            "eat_cake_slice" => "CAKE_SLICES_EATEN",
            "fill_cauldron" => "CAULDRON_FILLED",
            "use_cauldron" => "CAULDRON_USED",
            "clean_armor" => "ARMOR_CLEANED",
            "clean_banner" => "BANNER_CLEANED",
            "interact_with_brewingstand" => "BREWINGSTAND_INTERACTION",
            "interact_with_beacon" => "BEACON_INTERACTION",
            "inspect_dropper" => "DROPPER_INSPECTED",
            "inspect_hopper" => "HOPPER_INSPECTED",
            "inspect_dispenser" => "DISPENSER_INSPECTED",
            "play_noteblock" => "NOTEBLOCK_PLAYED",
            "tune_noteblock" => "NOTEBLOCK_TUNED",
            "pot_flower" => "FLOWER_POTTED",
            "trigger_trapped_chest" => "TRAPPED_CHEST_TRIGGERED",
            "open_enderchest" => "ENDERCHEST_OPENED",
            "enchant_item" => "ITEM_ENCHANTED",
            "play_record" => "RECORD_PLAYED",
            "interact_with_furnace" => "FURNACE_INTERACTION",
            "interact_with_crafting_table" => "CRAFTING_TABLE_INTERACTION",
            "open_chest" => "CHEST_OPENED",
            "open_shulker_box" => "SHULKER_BOX_OPENED",
            other => return other.to_uppercase(),
        }
        .to_owned(),
    }
}

/// bukkit: org/bukkit/event/player/PlayerStatisticIncrementEvent
///
/// Before a count goes in, as the player's turn ends: a cancel means it never does. The counters
/// that go up every tick aren't asked about, as in Bukkit.
#[derive(Clone, Debug, Default)]
pub struct PlayerStatisticIncrementEvent {
    value: Value,
    cancelled: bool,
    kind: String,
    key: String,
    amount: i32,
    total: i32,
}

impl Event for PlayerStatisticIncrementEvent {
    const HOOK: &'static str = "pl_stats";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_stats_on"]
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self {
            value: value.clone(),
            cancelled: crate::event::prior_cancelled(),
            kind: value["kind"].as_str()?.to_owned(),
            key: value["key"].as_str().unwrap_or_default().to_owned(),
            amount: value["amount"].as_i64().unwrap_or(0) as i32,
            total: value["total"].as_i64().unwrap_or(0) as i32,
        })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

impl PlayerStatisticIncrementEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerStatisticIncrementEvent.getStatistic
    ///
    /// `Statistic`'s name.
    pub fn get_statistic(&self) -> String {
        statistic_name(&self.kind, &self.key)
    }
    /// bukkit: org/bukkit/event/player/PlayerStatisticIncrementEvent.setCancelled
    ///
    /// The count never goes in.
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerStatisticIncrementEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    /// bukkit: org/bukkit/event/player/PlayerStatisticIncrementEvent.getNewValue
    pub fn get_new_value(&self) -> i32 {
        self.total
    }
    /// bukkit: org/bukkit/event/player/PlayerStatisticIncrementEvent.getPreviousValue
    pub fn get_previous_value(&self) -> i32 {
        self.total - self.amount
    }
    /// bukkit: org/bukkit/event/player/PlayerStatisticIncrementEvent.getMaterial
    ///
    /// What's counted, for a block or an item.
    pub fn get_material(&self) -> Option<crate::Material> {
        matches!(self.kind.trim_start_matches("minecraft:"), "mined" | "crafted" | "used" | "broken" | "picked_up" | "dropped").then(|| crate::Material::of(&self.key))
    }
    /// bukkit: org/bukkit/event/player/PlayerStatisticIncrementEvent.getEntityType
    ///
    /// `EntityType`'s key, for `KILL_ENTITY` and `ENTITY_KILLED_BY`.
    pub fn get_entity_type(&self) -> Option<&str> {
        matches!(self.kind.trim_start_matches("minecraft:"), "killed" | "killed_by").then_some(self.key.as_str())
    }
}

/// `net.kyori.adventure.chat.SignedMessage`: the message as it came, whatever the event does to it.
#[derive(Clone, Debug)]
pub struct SignedChatMessage {
    value: Value,
}

impl SignedChatMessage {
    pub(crate) fn of(value: Value) -> Self {
        Self { value }
    }
    /// The text the player signed.
    pub fn message(&self) -> &str {
        self.value["message"].as_str().unwrap_or_default()
    }
    /// Epoch milliseconds.
    pub fn timestamp(&self) -> i64 {
        self.value["timestamp"].as_i64().unwrap_or(0)
    }
    pub fn salt(&self) -> i64 {
        self.value["salt"].as_i64().unwrap_or(0)
    }
    /// The signature's bytes: an unsigned message has none.
    pub fn signature(&self) -> Option<Vec<u8>> {
        self.value["signature"].as_array().map(|bytes| bytes.iter().filter_map(|byte| byte.as_u64().map(|byte| byte as u8)).collect())
    }
    pub fn is_system(&self) -> bool {
        self.signature().is_none() && self.value["sender"].is_null()
    }
    /// Who signed it.
    pub fn identity(&self) -> &str {
        self.value["uuid"].as_str().unwrap_or_default()
    }
}

/// bukkit: io/papermc/paper/event/player/AsyncChatEvent, io/papermc/paper/event/player/AbstractChatEvent
///
/// After the legacy events, on the same thread, with what they left.
#[derive(Clone, Debug)]
pub struct AsyncChatEvent {
    player: Player,
    original: String,
    message: Option<String>,
    format: Option<String>,
    recipients: Option<Vec<Player>>,
    online: Vec<Player>,
    value: Value,
    cancelled: bool,
}

chat_event!(AsyncChatEvent, "pl_chat", true);

impl AsyncChatEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// The message as it stands.
    pub fn message(&self) -> crate::Component {
        match &self.message {
            Some(message) if !self.value["chat_component"].is_null() && self.value["chat_message"].as_str() == Some(message) => chat_message(&self.value),
            Some(message) => crate::adventure::legacy(message),
            None => chat_message(&self.value),
        }
    }
    /// bukkit: io/papermc/paper/event/player/AbstractChatEvent.message
    pub fn set_message(&mut self, message: &crate::Component) {
        self.message = Some(message.to_legacy());
        self.value["chat_component"] = message.0.clone();
        self.value["chat_message"] = json!(self.message);
        crate::event::carry("chat_component", message.0.clone());
    }
    /// bukkit: io/papermc/paper/event/player/AbstractChatEvent.renderer
    ///
    /// The line each viewer gets.
    pub fn renderer(&mut self, renderer: impl Fn(&Player, &crate::Component, &crate::Component, &Player) -> crate::Component + Send + Sync + 'static) {
        *RENDERER.lock().unwrap_or_else(|e| e.into_inner()) = Some(std::sync::Arc::new(renderer));
        crate::event::carry("chat_renderer", json!(true));
    }
    /// bukkit: io/papermc/paper/event/player/AbstractChatEvent.renderer
    ///
    /// The renderer set by this plugin, or the default's `<name> message`.
    pub fn get_renderer(&self) -> ChatRenderer {
        RENDERER.lock().unwrap_or_else(|e| e.into_inner()).clone().unwrap_or_else(|| {
            std::sync::Arc::new(|_source, name, message, _viewer| crate::Component(json!({ "translate": "chat.type.text", "with": [name.0, message.0] })))
        })
    }
    /// The message as the player typed it.
    pub fn original_message(&self) -> crate::Component {
        crate::adventure::legacy(&self.original)
    }
    /// bukkit: io/papermc/paper/event/player/AbstractChatEvent.signedMessage
    pub fn signed_message(&self) -> SignedChatMessage {
        SignedChatMessage { value: self.value.clone() }
    }
    /// bukkit: io/papermc/paper/event/player/AbstractChatEvent.viewers
    ///
    /// The players among them: `console_views` is the rest of the set. Java's set is live: put a
    /// changed list back with `set_viewers`.
    pub fn viewers(&self) -> Vec<Player> {
        self.recipients.clone().unwrap_or_else(|| self.online.clone())
    }
    pub fn set_viewers(&mut self, viewers: Vec<Player>) {
        self.recipients = Some(viewers);
    }
    /// Whether the console is among the viewers: then the line goes to the log.
    pub fn console_views(&self) -> bool {
        self.value["chat_console"] != false
    }
    pub fn set_console_views(&mut self, views: bool) {
        self.value["chat_console"] = json!(views);
        crate::event::carry("chat_console", json!(views));
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerDeepSleepEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerDeepSleepEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerDeepSleepEvent, "pl_deep_sleep", |_value| true);

impl PlayerDeepSleepEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerDeepSleepEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerDeepSleepEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Second pass: events with hooks of their own ----

/// A durability's experience, by default: the rate the event's own numbers show.
type XpOperation = std::sync::Arc<dyn Fn(i32) -> i32 + Send + Sync>;

/// bukkit: org/bukkit/event/player/PlayerItemMendEvent
///
/// An orb's experience about to repair an item with Mending. A cancel gives it all as experience.
#[derive(Clone, Default)]
pub struct PlayerItemMendEvent {
    value: Value,
    cancelled: bool,
    repair: Option<i32>,
    operation: Option<XpOperation>,
}

impl Event for PlayerItemMendEvent {
    const HOOK: &'static str = "pl_mend";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
    fn finish(&self) {
        if self.repair.is_some() || self.operation.is_some() {
            query(json!({ "op": "ev_player_mend", "repair": self.get_repair_amount().max(0), "used": self.get_consumed_experience().max(0) }));
        }
    }
}

impl PlayerItemMendEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerItemMendEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: org/bukkit/event/player/PlayerItemMendEvent.getSlot
    ///
    /// `EquipmentSlot`'s name.
    pub fn get_slot(&self) -> &str {
        self.value["slot"].as_str().unwrap_or("HAND")
    }
    /// bukkit: org/bukkit/event/player/PlayerItemMendEvent.getExperienceOrb
    ///
    /// The orb being taken.
    pub fn get_experience_orb(&self) -> Option<Entity> {
        Some(Entity::of(self.value["orb"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/player/PlayerItemMendEvent.getRepairAmount
    ///
    /// The durability it gives back.
    pub fn get_repair_amount(&self) -> i32 {
        self.repair.unwrap_or(self.value["repair"].as_i64().unwrap_or(0) as i32)
    }
    /// bukkit: org/bukkit/event/player/PlayerItemMendEvent.setRepairAmount
    pub fn set_repair_amount(&mut self, amount: i32) {
        self.repair = Some(amount);
    }
    /// bukkit: org/bukkit/event/player/PlayerItemMendEvent.getConsumedExperience
    ///
    /// The experience the repair uses.
    pub fn get_consumed_experience(&self) -> i32 {
        (self.get_durability_to_xp_operation())(self.get_repair_amount())
    }
    /// bukkit: org/bukkit/event/player/PlayerItemMendEvent.getDurabilityToXpOperation
    ///
    /// The game's rate: what this repair uses, for each point of it.
    pub fn get_durability_to_xp_operation(&self) -> XpOperation {
        if let Some(operation) = &self.operation {
            return operation.clone();
        }
        let (used, repair) = (self.value["used"].as_i64().unwrap_or(0), self.value["repair"].as_i64().unwrap_or(0));
        std::sync::Arc::new(move |durability| if repair == 0 { 0 } else { (i64::from(durability) * used / repair) as i32 })
    }
    /// bukkit: org/bukkit/event/player/PlayerItemMendEvent.setDurabilityToXpOperation
    pub fn set_durability_to_xp_operation(&mut self, operation: impl Fn(i32) -> i32 + Send + Sync + 'static) {
        self.operation = Some(std::sync::Arc::new(operation));
    }
    /// bukkit: org/bukkit/event/player/PlayerItemMendEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerItemMendEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerExpChangeEvent
///
/// Experience from an orb, after Mending has taken its part.
#[derive(Clone, Debug, Default)]
pub struct PlayerExpChangeEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerExpChangeEvent, "pl_exp", |_value| true);

impl PlayerExpChangeEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerExpChangeEvent.getAmount
    pub fn get_amount(&self) -> i32 {
        self.value["amount"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/player/PlayerExpChangeEvent.setAmount
    pub fn set_amount(&mut self, amount: i32) {
        self.value["amount"] = json!(amount.max(0));
        query(json!({ "op": "ev_player_exp", "amount": amount.max(0) }));
    }
    /// bukkit: org/bukkit/event/player/PlayerExpChangeEvent.getSource
    ///
    /// The orb.
    pub fn get_source(&self) -> Option<Entity> {
        Some(Entity::of(self.value["orb"].as_u64()?, &world_of(&self.value)))
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerPickupExperienceEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerPickupExperienceEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerPickupExperienceEvent, "pl_orb", |value| value["kind"] == "pickup");

impl PlayerPickupExperienceEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerPickupExperienceEvent.getExperienceOrb
    pub fn get_experience_orb(&self) -> Option<Entity> {
        Some(Entity::of(self.value["orb"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerPickupExperienceEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerPickupExperienceEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerExpCooldownChangeEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerExpCooldownChangeEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerExpCooldownChangeEvent, "pl_orb", |value| value["kind"] == "cooldown");

impl PlayerExpCooldownChangeEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerExpCooldownChangeEvent.getNewCooldown
    pub fn get_new_cooldown(&self) -> i32 {
        self.value["new"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/player/PlayerExpCooldownChangeEvent.setNewCooldown
    pub fn set_new_cooldown(&mut self, ticks: i32) {
        self.value["new"] = json!(ticks.max(0));
        crate::event::carry("new", json!(ticks.max(0)));
        query(json!({ "op": "ev_player_orb_cooldown", "ticks": ticks.max(0) }));
    }
    /// bukkit: org/bukkit/event/player/PlayerExpCooldownChangeEvent.getReason
    ///
    /// `ChangeReason`'s name.
    pub fn get_reason(&self) -> &str {
        self.value["reason"].as_str().unwrap_or("PICKUP_ORB")
    }
}

/// `Player.setExpCooldown`: `PlayerExpCooldownChangeEvent` with `PLUGIN`, and the ticks it left.
pub(crate) fn exp_cooldown_set(player: &Player, ticks: i32) -> i32 {
    query(json!({ "op": "ev_player_exp_cooldown", "uuid": player.uuid(), "name": player.get_name(), "ticks": ticks })).as_i64().map_or(ticks, |ticks| ticks as i32)
}

/// bukkit: io/papermc/paper/event/player/PlayerItemGroupCooldownEvent, io/papermc/paper/event/player/PlayerItemCooldownEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerItemGroupCooldownEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerItemGroupCooldownEvent, "pl_cooldown", |_value| true);

impl PlayerItemGroupCooldownEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerItemGroupCooldownEvent.getCooldownGroup
    pub fn get_cooldown_group(&self) -> Option<NamespacedKey> {
        NamespacedKey::from_string(self.value["group"].as_str()?)
    }
    /// The item of the group: a group has its item's name, unless the item names another.
    pub fn get_type(&self) -> crate::Material {
        crate::Material::of(self.value["group"].as_str().unwrap_or("minecraft:air"))
    }
    /// bukkit: io/papermc/paper/event/player/PlayerItemGroupCooldownEvent.getCooldown
    pub fn get_cooldown(&self) -> i32 {
        self.value["ticks"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: io/papermc/paper/event/player/PlayerItemGroupCooldownEvent.setCooldown
    pub fn set_cooldown(&mut self, ticks: i32) {
        self.value["ticks"] = json!(ticks);
        query(json!({ "op": "ev_player_cooldown", "ticks": ticks }));
    }
    /// bukkit: io/papermc/paper/event/player/PlayerItemGroupCooldownEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerItemGroupCooldownEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerShieldDisableEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerShieldDisableEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerShieldDisableEvent, "pl_shield", ["pl_shield_hit"], |_value| true);

impl PlayerShieldDisableEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerShieldDisableEvent.getDamager
    pub fn get_damager(&self) -> Option<Entity> {
        Some(Entity::of(self.value["attacker"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: io/papermc/paper/event/player/PlayerShieldDisableEvent.getCooldown
    pub fn get_cooldown(&self) -> i32 {
        self.value["ticks"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: io/papermc/paper/event/player/PlayerShieldDisableEvent.setCooldown
    pub fn set_cooldown(&mut self, ticks: i32) {
        self.value["ticks"] = json!(ticks.max(0));
        query(json!({ "op": "ev_player_shield", "ticks": ticks.max(0) }));
    }
    /// bukkit: io/papermc/paper/event/player/PlayerShieldDisableEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerShieldDisableEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerReadyArrowEvent
///
/// Each time a bow or crossbow looks for something to shoot: as it's drawn and as it lets go. A
/// cancel moves on to the next arrow.
#[derive(Clone, Debug, Default)]
pub struct PlayerReadyArrowEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerReadyArrowEvent, "pl_ready_arrow", |value| !value["arrow"].is_null());

impl PlayerReadyArrowEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerReadyArrowEvent.getBow
    pub fn get_bow(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["bow"])
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerReadyArrowEvent.getArrow
    pub fn get_arrow(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["arrow"])
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerReadyArrowEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerReadyArrowEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerLaunchProjectileEvent
///
/// Snowballs, eggs, ender pearls, potions and bottles of experience.
#[derive(Clone, Debug, Default)]
pub struct PlayerLaunchProjectileEvent {
    value: Value,
    cancelled: bool,
    keep: bool,
}

hooked!(PlayerLaunchProjectileEvent, "pl_launch", ["pl_throw", "pl_throw_bottle"], |_value| true);

impl PlayerLaunchProjectileEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerLaunchProjectileEvent.getItemStack
    pub fn get_item_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerLaunchProjectileEvent.getProjectile
    pub fn get_projectile(&self) -> Option<Entity> {
        Some(Entity::of(self.value["entity"].as_u64()?, &world_of(&self.value)))
    }
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerLaunchProjectileEvent.shouldConsume
    pub fn should_consume(&self) -> bool {
        !self.keep
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerLaunchProjectileEvent.setShouldConsume
    pub fn set_should_consume(&mut self, consume: bool) {
        self.keep = !consume;
        query(json!({ "op": "ev_player_keep_throw", "keep": !consume }));
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerLaunchProjectileEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerLaunchProjectileEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerEggThrowEvent
///
/// A player's egg as it breaks: its rolls are the game's, and an answer takes their place.
#[derive(Clone, Debug, Default)]
pub struct PlayerEggThrowEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerEggThrowEvent, "pl_egg", |_value| true);

impl PlayerEggThrowEvent {
    fn send(&self) {
        query(json!({ "op": "ev_player_hatch", "hatching": self.value["hatching"], "hatches": self.value["hatches"], "type": self.value["type"] }));
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerEggThrowEvent.getEgg
    pub fn get_egg(&self) -> Option<Entity> {
        Some(Entity::of(self.value["entity"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/player/PlayerEggThrowEvent.isHatching
    pub fn is_hatching(&self) -> bool {
        self.value["hatching"] == true
    }
    /// bukkit: org/bukkit/event/player/PlayerEggThrowEvent.setHatching
    pub fn set_hatching(&mut self, hatching: bool) {
        self.value["hatching"] = json!(hatching);
        if hatching && self.value["hatches"] == 0 {
            self.value["hatches"] = json!(1);
        }
        self.send();
    }
    /// bukkit: org/bukkit/event/player/PlayerEggThrowEvent.getNumHatches
    pub fn get_num_hatches(&self) -> i32 {
        self.value["hatches"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/player/PlayerEggThrowEvent.setNumHatches
    pub fn set_num_hatches(&mut self, hatches: i32) {
        self.value["hatches"] = json!(hatches);
        self.send();
    }
    /// bukkit: org/bukkit/event/player/PlayerEggThrowEvent.getHatchingType
    ///
    /// `EntityType`'s key.
    pub fn get_hatching_type(&self) -> &str {
        self.value["type"].as_str().unwrap_or("minecraft:chicken")
    }
    /// bukkit: org/bukkit/event/player/PlayerEggThrowEvent.setHatchingType
    pub fn set_hatching_type(&mut self, kind: &str) {
        self.value["type"] = json!(kind);
        self.send();
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerStartSpectatingEntityEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerStartSpectatingEntityEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerStartSpectatingEntityEvent, "pl_camera", |value| !value["new"].is_null());

impl PlayerStartSpectatingEntityEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerStartSpectatingEntityEvent.getCurrentSpectatorTarget
    ///
    /// The player's own entity while nothing else is.
    pub fn get_current_spectator_target(&self) -> Option<Entity> {
        let id = self.value["old"].as_u64().or(self.value["entity_id"].as_u64())?;
        Some(Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerStartSpectatingEntityEvent.getNewSpectatorTarget
    pub fn get_new_spectator_target(&self) -> Option<Entity> {
        Some(Entity::of(self.value["new"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerStartSpectatingEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerStartSpectatingEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerStopSpectatingEntityEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerStopSpectatingEntityEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerStopSpectatingEntityEvent, "pl_camera", |value| value["new"].is_null() && !value["old"].is_null());

impl PlayerStopSpectatingEntityEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerStopSpectatingEntityEvent.getSpectatorTarget
    pub fn get_spectator_target(&self) -> Option<Entity> {
        Some(Entity::of(self.value["old"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerStopSpectatingEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerStopSpectatingEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerAdvancementCriterionGrantEvent
///
/// Before the criterion goes in.
#[derive(Clone, Debug, Default)]
pub struct PlayerAdvancementCriterionGrantEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerAdvancementCriterionGrantEvent, "pl_criterion", |_value| true);

impl PlayerAdvancementCriterionGrantEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerAdvancementCriterionGrantEvent.getCriterion
    pub fn get_criterion(&self) -> &str {
        self.value["criterion"].as_str().unwrap_or_default()
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerAdvancementCriterionGrantEvent.getAdvancement
    pub fn get_advancement(&self) -> Option<crate::core2::Advancement> {
        crate::core2::get_advancement(self.value["advancement"].as_str()?)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerAdvancementCriterionGrantEvent.getAdvancementProgress
    pub fn get_advancement_progress(&self) -> Option<crate::core2::AdvancementProgress> {
        Some(self.get_player()?.get_advancement_progress(&self.get_advancement()?))
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerAdvancementCriterionGrantEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerAdvancementCriterionGrantEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerToggleEntityAgeLockEvent
///
/// A golden dandelion on a mob. A cancel leaves the mob and the flower as they were.
#[derive(Clone, Debug, Default)]
pub struct PlayerToggleEntityAgeLockEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerToggleEntityAgeLockEvent, "pl_age_lock", |_value| true);

impl PlayerToggleEntityAgeLockEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerToggleEntityAgeLockEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        Some(Entity::of(self.value["entity"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: io/papermc/paper/event/player/PlayerToggleEntityAgeLockEvent.isAgeLocked
    pub fn is_age_locked(&self) -> bool {
        self.value["locked"] == true
    }
    /// bukkit: io/papermc/paper/event/player/PlayerToggleEntityAgeLockEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerToggleEntityAgeLockEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: io/papermc/paper/event/player/PlayerToggleEntityAgeLockEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerToggleEntityAgeLockEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerUnleashEntityEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerUnleashEntityEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerUnleashEntityEvent, "pl_unleash", |value| value["held"] == true);

impl PlayerUnleashEntityEvent {
    /// bukkit: org/bukkit/event/player/PlayerUnleashEntityEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        Some(Entity::of(self.value["entity"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/player/PlayerUnleashEntityEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// `EntityUnleashEvent`'s: this event is one.
    pub fn get_reason(&self) -> &'static str {
        "PLAYER_UNLEASH"
    }
    pub fn is_drop_leash(&self) -> bool {
        self.value["drop"] == true
    }
    pub fn set_drop_leash(&mut self, drop: bool) {
        self.value["drop"] = json!(drop);
        crate::event::carry("drop", json!(drop));
        query(json!({ "op": "ev_player_unleash_drop", "drop": drop }));
    }
    /// bukkit: org/bukkit/event/player/PlayerUnleashEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerUnleashEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerBucketEntityEvent, org/bukkit/event/player/PlayerBucketFishEvent
///
/// A water bucket about to scoop up a fish, a tadpole or an axolotl.
#[derive(Clone, Debug, Default)]
pub struct PlayerBucketEntityEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerBucketEntityEvent, "pl_bucket_mob", |value| value["scoop"].is_object());

impl PlayerBucketEntityEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerBucketEntityEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        Some(Entity::of(self.value["scoop"]["entity"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/player/PlayerBucketEntityEvent.getOriginalBucket
    pub fn get_original_bucket(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: org/bukkit/event/player/PlayerBucketEntityEvent.getEntityBucket
    pub fn get_entity_bucket(&self) -> Option<ItemStack> {
        Some(ItemStack::new(crate::Material::of(self.value["scoop"]["bucket"].as_str()?), 1))
    }
    pub fn get_water_bucket(&self) -> Option<ItemStack> {
        self.get_original_bucket()
    }
    pub fn get_fish_bucket(&self) -> Option<ItemStack> {
        self.get_entity_bucket()
    }
    /// bukkit: org/bukkit/event/player/PlayerBucketEntityEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: org/bukkit/event/player/PlayerBucketEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerBucketEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

macro_rules! bucket_event {
    ($name:ident, |$value:ident| $wanted:expr) => {
        impl Event for $name {
            const HOOK: &'static str = "pl_bucket";
            fn more_hooks() -> &'static [&'static str] {
                &["tick"]
            }
            fn intercept(value: &Value) -> Option<Answer> {
                tick_answer(value)
            }
            fn read($value: &Value) -> Option<Self> {
                ($wanted).then(|| Self { value: $value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
            }
            fn answer(&self) -> Answer {
                answer_of(self.cancelled)
            }
        }
    };
}

/// `setItemStack`: what the hand gets as the bucket's done.
fn bucket_result(stack: Option<&ItemStack>) {
    query(json!({ "op": "ev_player_bucket_result", "stack": snbt_of(stack) }));
}

/// bukkit: org/bukkit/event/player/PlayerBucketEmptyEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerBucketEmptyEvent {
    value: Value,
    cancelled: bool,
    result: Option<ItemStack>,
}

bucket_event!(PlayerBucketEmptyEvent, |value| value["stack"]["item"] != "minecraft:bucket");

/// bukkit: org/bukkit/event/player/PlayerBucketFillEvent, org/bukkit/event/player/PlayerBucketEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerBucketFillEvent {
    value: Value,
    cancelled: bool,
    result: Option<ItemStack>,
}

bucket_event!(PlayerBucketFillEvent, |value| value["stack"]["item"] == "minecraft:bucket");

fn block_face_of(face: &Value) -> Option<&'static str> {
    let at = |i: usize| face[i].as_i64();
    Some(match (at(0)?, at(1)?, at(2)?) {
        (1, _, _) => "EAST",
        (-1, _, _) => "WEST",
        (_, 1, _) => "UP",
        (_, -1, _) => "DOWN",
        (_, _, 1) => "SOUTH",
        _ => "NORTH",
    })
}

impl PlayerBucketFillEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// What's in the hand after it.
    pub fn get_item_stack(&self) -> Option<ItemStack> {
        self.result.clone().or_else(|| ItemStack::read(&self.value["result"]))
    }
    /// bukkit: org/bukkit/event/player/PlayerBucketEvent.setItemStack
    pub fn set_item_stack(&mut self, stack: &ItemStack) {
        self.result = Some(stack.clone());
        bucket_result(Some(stack));
    }
    /// The bucket's item.
    pub fn get_bucket(&self) -> Option<crate::Material> {
        ItemStack::read(&self.value["stack"]).map(|stack| stack.get_type().clone())
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    pub fn get_block_clicked(&self) -> Option<Block> {
        self.get_block()
    }
    pub fn get_block_face(&self) -> Option<&'static str> {
        block_face_of(&self.value["face"])
    }
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl PlayerBucketEmptyEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// What's in the hand after it.
    pub fn get_item_stack(&self) -> Option<ItemStack> {
        self.result.clone().or_else(|| ItemStack::read(&self.value["result"]))
    }
    /// bukkit: org/bukkit/event/player/PlayerBucketEmptyEvent.setItemStack
    pub fn set_item_stack(&mut self, stack: &ItemStack) {
        self.result = Some(stack.clone());
        bucket_result(Some(stack));
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    pub fn get_block_face(&self) -> Option<&'static str> {
        block_face_of(&self.value["face"])
    }
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: org/bukkit/event/player/PlayerBucketEvent.getBucket
    ///
    /// The bucket emptied (`WATER_BUCKET`, `LAVA_BUCKET`...): CoreProtect logs water or lava by it.
    pub fn get_bucket(&self) -> Option<crate::Material> {
        ItemStack::read(&self.value["stack"]).map(|stack| stack.get_type().clone())
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerPortalEvent
///
/// Nether and End portals, before the trip: `getTo` is where the exit is looked for, as on Paper.
#[derive(Clone, Debug, Default)]
pub struct PlayerPortalEvent {
    value: Value,
    cancelled: bool,
    to: Option<Location>,
    search: Option<i32>,
    can_create: Option<bool>,
    creation: Option<i32>,
}

impl Event for PlayerPortalEvent {
    const HOOK: &'static str = "pl_portal";
    fn more_hooks() -> &'static [&'static str] {
        &["tick", "pl_portal_exit"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        tick_answer(value)
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled || self.to.is_some())
    }
    fn finish(&self) {
        if let (false, Some(to), Some(player)) = (self.cancelled, self.to.clone(), self.get_player()) {
            at_tick(move || {
                player.teleport(&to);
            });
        }
    }
}

impl PlayerPortalEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// `TeleportCause`'s name.
    pub fn get_cause(&self) -> &str {
        self.value["cause"].as_str().unwrap_or("NETHER_PORTAL")
    }
    /// bukkit: org/bukkit/event/player/PlayerPortalEvent.getTo
    ///
    /// Where the portal leads, as the game's first guess of the exit.
    pub fn get_to(&self) -> Option<Location> {
        if let Some(to) = &self.to {
            return Some(to.clone());
        }
        let pos = &self.value["to"];
        Some(Location::new(world_of(&self.value), pos[0].as_f64()?, pos[1].as_f64()?, pos[2].as_f64()?, 0.0, 0.0))
    }
    /// bukkit: org/bukkit/event/player/PlayerPortalEvent.setTo
    ///
    /// They go there in place of through the portal.
    pub fn set_to(&mut self, to: &Location) {
        self.to = Some(to.clone());
    }
    /// bukkit: org/bukkit/event/player/PlayerPortalEvent.getSearchRadius
    ///
    /// How far the game looks for a portal to come out of.
    pub fn get_search_radius(&self) -> i32 {
        self.search.unwrap_or(if self.value["level"].as_str().unwrap_or_default().ends_with("the_nether") { 16 } else { 128 })
    }
    /// bukkit: org/bukkit/event/player/PlayerPortalEvent.setSearchRadius
    pub fn set_search_radius(&mut self, radius: i32) {
        self.search = Some(radius);
        self.rules();
    }
    fn rules(&self) {
        query(json!({ "op": "ev_player_portal", "at": self.value["to"], "search": self.search, "create": self.can_create != Some(false), "creation": self.creation, "back": self.value["back"], "from": self.value["from"] }));
    }
    /// bukkit: org/bukkit/event/player/PlayerPortalEvent.getCreationRadius
    ///
    /// How far from the exit the game may build a portal.
    pub fn get_creation_radius(&self) -> i32 {
        self.creation.unwrap_or(16)
    }
    /// bukkit: org/bukkit/event/player/PlayerPortalEvent.setCreationRadius
    pub fn set_creation_radius(&mut self, radius: i32) {
        self.creation = Some(radius);
        self.rules();
    }
    /// bukkit: org/bukkit/event/player/PlayerPortalEvent.getCanCreatePortal
    ///
    /// The game makes one when it finds none.
    pub fn get_can_create_portal(&self) -> bool {
        self.can_create != Some(false)
    }
    /// bukkit: org/bukkit/event/player/PlayerPortalEvent.setCanCreatePortal
    pub fn set_can_create_portal(&mut self, can_create: bool) {
        self.can_create = Some(can_create);
        self.rules();
    }
    /// bukkit: org/bukkit/event/player/PlayerPortalEvent.willDismountPlayer
    ///
    /// A portal takes them off what they ride.
    pub fn will_dismount_player(&self) -> bool {
        true
    }
    /// bukkit: org/bukkit/event/player/PlayerPortalEvent.getRelativeTeleportationFlags
    ///
    /// A portal's exit is a whole position.
    pub fn get_relative_teleportation_flags(&self) -> Vec<String> {
        Vec::new()
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerTeleportEndGatewayEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerTeleportEndGatewayEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerTeleportEndGatewayEvent, "pl_gateway", |_value| true);

impl PlayerTeleportEndGatewayEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerTeleportEndGatewayEvent.getGateway
    ///
    /// The gateway's block.
    pub fn get_gateway(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    pub fn get_from(&self) -> Option<Location> {
        self.get_player().map(|player| player.get_location())
    }
    /// Where it leads. On a gateway's first trip that's the area: Paper's exact spot needs the chunks
    /// there, which load as they arrive, so the game finds it then.
    pub fn get_to(&self) -> Option<Location> {
        let at = &self.value["to"];
        let from = self.get_from()?;
        Some(Location::new(from.world.clone(), at[0].as_f64()?, at[1].as_f64()?, at[2].as_f64()?, from.yaw, from.pitch))
    }
    /// Takes them there in place of the exit.
    pub fn set_to(&mut self, to: &Location) {
        self.value["to"] = json!([to.x, to.y, to.z]);
        crate::event::carry("to", self.value["to"].clone());
        query(json!({ "op": "ev_player_gateway_to", "to": [to.x, to.y, to.z] }));
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Held packets, second pass ----

/// bukkit: org/bukkit/event/player/PlayerTakeLecternBookEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerTakeLecternBookEvent {
    value: Value,
    cancelled: bool,
}

held_event!(PlayerTakeLecternBookEvent, |value| value["kind"] == "lectern_take");

impl PlayerTakeLecternBookEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerTakeLecternBookEvent.getLectern
    pub fn get_lectern(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/player/PlayerTakeLecternBookEvent.getBook
    pub fn get_book(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["book"])
    }
    /// bukkit: org/bukkit/event/player/PlayerTakeLecternBookEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerTakeLecternBookEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerLecternPageChangeEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerLecternPageChangeEvent {
    value: Value,
    cancelled: bool,
}

held_event!(PlayerLecternPageChangeEvent, |value| value["kind"] == "lectern_page");

impl PlayerLecternPageChangeEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLecternPageChangeEvent.getLectern
    pub fn get_lectern(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLecternPageChangeEvent.getBook
    pub fn get_book(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["book"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLecternPageChangeEvent.getOldPage
    pub fn get_old_page(&self) -> i32 {
        self.value["page"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLecternPageChangeEvent.getNewPage
    ///
    /// The page the button asks for, within the book.
    pub fn get_new_page(&self) -> i32 {
        if let Some(page) = self.value["new_page"].as_i64() {
            return page as i32;
        }
        let old = self.get_old_page();
        let page = match self.value["button"].as_i64().unwrap_or(0) {
            1 => old - 1,
            2 => old + 1,
            button => button as i32 - 100,
        };
        let components = &self.value["book"]["components"];
        let pages = components["minecraft:written_book_content"]["pages"].as_array().or(components["minecraft:writable_book_content"]["pages"].as_array()).map_or(1, |pages| pages.len() as i32);
        page.clamp(0, (pages - 1).max(0))
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLecternPageChangeEvent.setNewPage
    pub fn set_new_page(&mut self, page: i32) {
        self.value["new_page"] = json!(page);
        query(json!({ "op": "ev_player_page", "page": page }));
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLecternPageChangeEvent.getPageChangeDirection
    ///
    /// `PageChangeDirection`'s name.
    pub fn get_page_change_direction(&self) -> &'static str {
        if self.get_new_page() >= self.get_old_page() { "RIGHT" } else { "LEFT" }
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLecternPageChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLecternPageChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerRecipeBookClickEvent, org/bukkit/event/player/PlayerRecipeBookClickEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerRecipeBookClickEvent {
    value: Value,
    cancelled: bool,
}

held_event!(PlayerRecipeBookClickEvent, |value| value["kind"] == "recipe_click");

impl PlayerRecipeBookClickEvent {
    fn send(&self, recipe: Option<&str>) {
        query(json!({ "op": "ev_player_recipe", "recipe": recipe, "shift": self.value["shift"] }));
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerRecipeBookClickEvent.getRecipe
    pub fn get_recipe(&self) -> Option<NamespacedKey> {
        NamespacedKey::from_string(self.value["recipe"].as_str()?)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerRecipeBookClickEvent.setRecipe
    pub fn set_recipe(&mut self, recipe: &str) {
        if self.value.get("original").is_none() {
            let first = self.value["recipe"].clone();
            crate::event::carry("original", first.clone());
            self.value["original"] = first;
        }
        self.value["recipe"] = json!(recipe);
        crate::event::carry("recipe", json!(recipe));
        self.send(Some(recipe));
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeBookClickEvent.getRecipe
    pub fn get_game_recipe(&self) -> Option<crate::core2::GameRecipe> {
        crate::core2::get_recipe(&self.get_recipe()?)
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeBookClickEvent.getOriginalRecipe
    pub fn get_original_recipe(&self) -> Option<crate::core2::GameRecipe> {
        let key = self.value.get("original").unwrap_or(&self.value["recipe"]);
        crate::core2::get_recipe(&NamespacedKey::from_string(key.as_str()?)?)
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeBookClickEvent.setRecipe
    pub fn set_game_recipe(&mut self, recipe: &crate::core2::GameRecipe) {
        self.set_recipe(&recipe.key);
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerRecipeBookClickEvent.isMakeAll
    pub fn is_make_all(&self) -> bool {
        self.value["shift"] == true
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerRecipeBookClickEvent.setMakeAll
    pub fn set_make_all(&mut self, all: bool) {
        self.value["shift"] = json!(all);
        self.send(None);
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeBookClickEvent.isShiftClick
    pub fn is_shift_click(&self) -> bool {
        self.is_make_all()
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeBookClickEvent.setShiftClick
    pub fn set_shift_click(&mut self, shift: bool) {
        self.set_make_all(shift);
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerRecipeBookClickEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerRecipeBookClickEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerChangeBeaconEffectEvent
///
/// A beacon's confirm button. A cancel keeps the beacon as it was and the payment where it is.
#[derive(Clone, Debug, Default)]
pub struct PlayerChangeBeaconEffectEvent {
    value: Value,
    cancelled: bool,
    keep: bool,
}

held_event!(PlayerChangeBeaconEffectEvent, |value| value["kind"] == "beacon");

impl PlayerChangeBeaconEffectEvent {
    fn send(&self) {
        query(json!({ "op": "ev_player_beacon", "primary": self.value["primary"], "secondary": self.value["secondary"], "keep": self.keep }));
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerChangeBeaconEffectEvent.getBeacon
    pub fn get_beacon(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerChangeBeaconEffectEvent.getPrimary
    pub fn get_primary(&self) -> Option<&str> {
        self.value["primary"].as_str()
    }
    /// bukkit: io/papermc/paper/event/player/PlayerChangeBeaconEffectEvent.setPrimary
    pub fn set_primary(&mut self, effect: &str) {
        self.value["primary"] = json!(effect);
        self.send();
    }
    /// bukkit: io/papermc/paper/event/player/PlayerChangeBeaconEffectEvent.getSecondary
    pub fn get_secondary(&self) -> Option<&str> {
        self.value["secondary"].as_str()
    }
    /// bukkit: io/papermc/paper/event/player/PlayerChangeBeaconEffectEvent.setSecondary
    pub fn set_secondary(&mut self, effect: &str) {
        self.value["secondary"] = json!(effect);
        self.send();
    }
    /// bukkit: io/papermc/paper/event/player/PlayerChangeBeaconEffectEvent.willConsumeItem
    pub fn will_consume_item(&self) -> bool {
        self.value["paid"] == true && !self.keep
    }
    /// bukkit: io/papermc/paper/event/player/PlayerChangeBeaconEffectEvent.setConsumeItem
    pub fn set_consume_item(&mut self, consume: bool) {
        self.keep = !consume;
        self.send();
    }
    /// bukkit: io/papermc/paper/event/player/PlayerChangeBeaconEffectEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerChangeBeaconEffectEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PrePlayerAttackEntityEvent
///
/// A player's attack packet, before the game looks at it. A cancel means no attack.
#[derive(Clone, Debug, Default)]
pub struct PrePlayerAttackEntityEvent {
    value: Value,
    cancelled: bool,
}

held_event!(PrePlayerAttackEntityEvent, |value| value["kind"] == "attack");

impl PrePlayerAttackEntityEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PrePlayerAttackEntityEvent.getAttacked
    pub fn get_attacked(&self) -> Option<Entity> {
        Some(Entity::of(self.value["entity"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: io/papermc/paper/event/player/PrePlayerAttackEntityEvent.willAttack
    ///
    /// Whether the game hits such a thing at all: not items, orbs and arrows.
    pub fn will_attack(&self) -> bool {
        !matches!(self.value["target"].as_str().map(|kind| kind.trim_start_matches("minecraft:")), Some("item" | "experience_orb" | "arrow" | "spectral_arrow" | ""))
    }
    /// bukkit: io/papermc/paper/event/player/PrePlayerAttackEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PrePlayerAttackEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerAttackEntityCooldownResetEvent
///
/// A player's attack, about to reset the cooldown. A cancel keeps the charge they had.
#[derive(Clone, Debug, Default)]
pub struct PlayerAttackEntityCooldownResetEvent {
    value: Value,
    cancelled: bool,
}

impl Event for PlayerAttackEntityCooldownResetEvent {
    const HOOK: &'static str = "pl_packet";
    fn more_hooks() -> &'static [&'static str] {
        HELD_HOOKS
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "attack").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), ..Default::default() })
    }
    fn finish(&self) {
        if self.cancelled {
            query(json!({ "op": "ev_player_keep_attack", "keep": true }));
        }
    }
}

impl PlayerAttackEntityCooldownResetEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerAttackEntityCooldownResetEvent.getAttackedEntity
    pub fn get_attacked_entity(&self) -> Option<Entity> {
        Some(Entity::of(self.value["entity"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerAttackEntityCooldownResetEvent.getCooledAttackStrength
    pub fn get_cooled_attack_strength(&self) -> f32 {
        self.value["strength"].as_f64().unwrap_or(1.0) as f32
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerAttackEntityCooldownResetEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerAttackEntityCooldownResetEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerRegisterChannelEvent, org/bukkit/event/player/PlayerChannelEvent
#[derive(Clone, Debug)]
pub struct PlayerRegisterChannelEvent {
    player: Option<Player>,
    channel: String,
}

/// bukkit: org/bukkit/event/player/PlayerUnregisterChannelEvent
#[derive(Clone, Debug)]
pub struct PlayerUnregisterChannelEvent {
    player: Option<Player>,
    channel: String,
}

macro_rules! channel_event {
    ($name:ident, $register:literal) => {
        impl Event for $name {
            const HOOK: &'static str = "pl_packet";
            fn more_hooks() -> &'static [&'static str] {
                HELD_HOOKS
            }
            fn read(_: &Value) -> Option<Self> {
                None
            }
            fn read_all(value: &Value) -> Vec<Self> {
                if value["kind"] != "channel" || value["register"] != $register {
                    return Vec::new();
                }
                let player = player_of(value);
                value["channels"].as_array().into_iter().flatten().filter_map(Value::as_str).map(|channel| Self { player: player.clone(), channel: channel.to_owned() }).collect()
            }
        }
    };
}

channel_event!(PlayerRegisterChannelEvent, true);
channel_event!(PlayerUnregisterChannelEvent, false);

impl PlayerRegisterChannelEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<&Player> {
        self.player.as_ref()
    }
    pub fn get_channel(&self) -> &str {
        &self.channel
    }
}

impl PlayerUnregisterChannelEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<&Player> {
        self.player.as_ref()
    }
    pub fn get_channel(&self) -> &str {
        &self.channel
    }
}

// ---- Seen once a tick ----

/// bukkit: org/bukkit/event/player/PlayerChatTabCompleteEvent
///
/// Paper no longer fires this: clients don't ask for it. So it never fires here.
#[derive(Clone, Debug, Default)]
pub struct PlayerChatTabCompleteEvent {
    message: String,
    completions: Vec<String>,
}

impl Event for PlayerChatTabCompleteEvent {
    const HOOK: &'static str = "pl_never";
    fn read(_: &Value) -> Option<Self> {
        None
    }
}

impl PlayerChatTabCompleteEvent {
    /// bukkit: org/bukkit/event/player/PlayerChatTabCompleteEvent.getChatMessage
    pub fn get_chat_message(&self) -> &str {
        &self.message
    }
    /// bukkit: org/bukkit/event/player/PlayerChatTabCompleteEvent.getLastToken
    pub fn get_last_token(&self) -> &str {
        self.message.rsplit(' ').next().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/player/PlayerChatTabCompleteEvent.getTabCompletions
    pub fn get_tab_completions(&self) -> &[String] {
        &self.completions
    }
}

/// bukkit: org/bukkit/event/player/AsyncPlayerChatPreviewEvent
///
/// Paper no longer fires this: chat previews were removed. So it never fires here.
#[derive(Clone, Debug, Default)]
pub struct AsyncPlayerChatPreviewEvent;

impl Event for AsyncPlayerChatPreviewEvent {
    const HOOK: &'static str = "pl_never";
    fn read(_: &Value) -> Option<Self> {
        None
    }
}

/// bukkit: org/bukkit/event/player/PlayerToggleSneakEvent.isSneaking, org/bukkit/event/player/PlayerToggleSprintEvent.isSprinting, org/bukkit/event/player/PlayerToggleFlightEvent.isFlying, org/bukkit/event/player/PlayerToggleSneakEvent.setCancelled, org/bukkit/event/player/PlayerToggleSneakEvent.isCancelled, org/bukkit/event/player/PlayerToggleSprintEvent.setCancelled, org/bukkit/event/player/PlayerToggleSprintEvent.isCancelled, org/bukkit/event/player/PlayerToggleFlightEvent.setCancelled, org/bukkit/event/player/PlayerToggleFlightEvent.isCancelled
macro_rules! toggle_event {
    ($name:ident, $what:literal, $getter:ident) => {
        impl Event for $name {
            const HOOK: &'static str = "pl_packet";
            fn read(value: &Value) -> Option<Self> {
                if value["kind"] != "toggle" || value["what"] != $what {
                    return None;
                }
                Some(Self { player: player_of(value)?, now: value["now"] == true, cancelled: crate::event::prior_cancelled() })
            }
            fn answer(&self) -> Answer {
                answer_of(self.cancelled)
            }
        }
        impl $name {
            pub fn get_player(&self) -> &Player {
                &self.player
            }
            pub fn $getter(&self) -> bool {
                self.now
            }
            pub fn set_cancelled(&mut self, cancelled: bool) {
                self.cancelled = cancelled;
            }
            pub fn is_cancelled(&self) -> bool {
                self.cancelled
            }
        }
    };
}

/// bukkit: org/bukkit/event/player/PlayerToggleSneakEvent
///
/// The client's input as it comes in. A cancel drops the packet, as Paper does.
#[derive(Clone, Debug)]
pub struct PlayerToggleSneakEvent {
    player: Player,
    now: bool,
    cancelled: bool,
}

/// bukkit: org/bukkit/event/player/PlayerToggleSprintEvent
#[derive(Clone, Debug)]
pub struct PlayerToggleSprintEvent {
    player: Player,
    now: bool,
    cancelled: bool,
}

/// bukkit: org/bukkit/event/player/PlayerToggleFlightEvent
///
/// A client asking to fly or to stop. A cancel tells it where it stands.
#[derive(Clone, Debug)]
pub struct PlayerToggleFlightEvent {
    player: Player,
    now: bool,
    cancelled: bool,
}

toggle_event!(PlayerToggleSneakEvent, "sneak", is_sneaking);
toggle_event!(PlayerToggleSprintEvent, "sprint", is_sprinting);
toggle_event!(PlayerToggleFlightEvent, "flight", is_flying);

/// bukkit: org/bukkit/event/player/PlayerItemHeldEvent
///
/// A hotbar key or the wheel. A cancel sends them back to the slot they're on.
#[derive(Clone, Debug)]
pub struct PlayerItemHeldEvent {
    player: Player,
    before: i64,
    now: i64,
    cancelled: bool,
}

impl Event for PlayerItemHeldEvent {
    const HOOK: &'static str = "pl_packet";
    fn read(value: &Value) -> Option<Self> {
        if value["kind"] != "held_slot" {
            return None;
        }
        Some(Self { player: player_of(value)?, before: value["old"].as_i64()?, now: value["new"].as_i64()?, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

impl PlayerItemHeldEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/event/player/PlayerItemHeldEvent.getNewSlot
    pub fn get_new_slot(&self) -> i64 {
        self.now
    }
    /// bukkit: org/bukkit/event/player/PlayerItemHeldEvent.getPreviousSlot
    pub fn get_previous_slot(&self) -> i64 {
        self.before
    }
    /// bukkit: org/bukkit/event/player/PlayerItemHeldEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerItemHeldEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerInteractEntityEvent, org/bukkit/event/player/PlayerInteractAtEntityEvent
///
/// A right click on an entity. The packet carries the hand and where on the entity it landed.
pub struct PlayerInteractEntityEvent {
    pub player: Player,
    pub entity_id: u64,
    pub kind: String,
    value: Value,
    cancelled: bool,
}

impl Event for PlayerInteractEntityEvent {
    const HOOK: &'static str = "pl_packet";
    fn more_hooks() -> &'static [&'static str] {
        &["use_entity"]
    }
    fn read(value: &Value) -> Option<Self> {
        let guest = value["kind"] == "use_entity" && value.get("player").is_none();
        // The game's own hook hears every guest too: from there, only the one playing here.
        if !guest && (value.get("player").is_none() || lan_acting()) {
            return None;
        }
        let kind = value[if guest { "type" } else { "kind" }].as_str().unwrap_or_default().to_owned();
        Some(Self { player: player_of(value)?, entity_id: value["entity"].as_u64()?, kind, value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

impl PlayerInteractEntityEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    pub fn get_right_clicked_id(&self) -> u64 {
        self.entity_id
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEntityEvent.getRightClicked
    pub fn get_right_clicked(&self) -> Entity {
        Entity::of(self.entity_id, &self.player.get_world())
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntityType
    pub fn get_entity_type(&self) -> &str {
        &self.kind
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEntityEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// Where on the entity it landed, from its position.
    pub fn get_clicked_position(&self) -> Option<crate::util::Vector> {
        let at = &self.value["at"];
        Some(crate::util::Vector::new(at[0].as_f64()?, at[1].as_f64()?, at[2].as_f64()?))
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// True while a guest's turn runs, which the packets have heard already.
fn lan_acting() -> bool {
    query(json!({ "op": "ev_player_lan_acting" })) == true
}

/// bukkit: org/bukkit/event/player/PlayerInteractEvent
///
/// A click as its packet comes in, before the game acts on it.
pub struct PlayerInteractEvent {
    pub player: Player,
    /// `Action`'s name.
    pub action: &'static str,
    pub block: Option<Block>,
    value: Value,
    use_block: String,
    use_item: String,
}

impl Event for PlayerInteractEvent {
    const HOOK: &'static str = "pl_packet";
    fn more_hooks() -> &'static [&'static str] {
        &["use_block", "attack_block", "use_item", "pl_step"]
    }
    fn read(value: &Value) -> Option<Self> {
        let (player, action) = match value["kind"].as_str() {
            Some("interact") => (player_of(value)?, match value["action"].as_str()? {
                "LEFT_CLICK_BLOCK" => "LEFT_CLICK_BLOCK",
                "RIGHT_CLICK_AIR" => "RIGHT_CLICK_AIR",
                _ => "RIGHT_CLICK_BLOCK",
            }),
            Some("swing") if value["air"] == true => (player_of(value)?, "LEFT_CLICK_AIR"),
            Some("swing") => return None,
            Some("step") => (player_of(value)?, "PHYSICAL"),
            None if value.get("player").is_some() && !lan_acting() => {
                // The game's own hooks hear the one playing here.
                let action = match (value.get("face").is_some(), value.get("stack").is_some()) {
                    (_, true) => "RIGHT_CLICK_AIR",
                    (true, false) if value["attack"].as_bool() == Some(true) => "LEFT_CLICK_BLOCK",
                    _ => "RIGHT_CLICK_BLOCK",
                };
                (player_of(value)?, action)
            }
            _ => return None,
        };
        let block = if value["pos"].is_array() { block_at(value, &value["pos"]).or_else(|| crate::event::block_of(&player, &value["pos"])) } else { None };
        let denied = crate::event::prior_cancelled() || block.is_none();
        let word = |key: &str, default: &str| value[key].as_str().unwrap_or(default).to_owned();
        let use_block = word("use_block", if denied { "DENY" } else { "ALLOW" });
        let use_item = word("use_item", if crate::event::prior_cancelled() { "DENY" } else { "DEFAULT" });
        Some(Self { player, action, block, value: value.clone(), use_block, use_item })
    }
    fn answer(&self) -> Answer {
        // A swing has its own event, whose cancel is the only one that stops it.
        answer_of(self.is_cancelled() && self.value["kind"] != "swing")
    }
}

impl PlayerInteractEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit~: org/bukkit/event/player/PlayerInteractEvent.getAction | `PHYSICAL` on farmland and turtle eggs lands on the next game turn: an egg break there has no sound, and farmland lifts the player by teleport
    pub fn get_action(&self) -> &str {
        self.action
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.getClickedBlock
    pub fn get_clicked_block(&self) -> Option<&Block> {
        self.block.as_ref()
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.getBlockFace
    ///
    /// A click in the air says `SOUTH`, and a step `SELF`, as Paper's do.
    pub fn get_block_face(&self) -> Option<&str> {
        if self.action == "PHYSICAL" {
            return Some("SELF");
        }
        block_face_of(&self.value["face"]).or(Some("SOUTH"))
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        if self.action == "PHYSICAL" {
            return None;
        }
        if self.value.get("kind").is_some() {
            return ItemStack::read(&self.value["stack"]);
        }
        self.player.get_inventory().get_item_in_main_hand()
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.getMaterial
    pub fn get_material(&self) -> crate::Material {
        self.get_item().map_or_else(|| crate::Material::of("AIR"), |stack| stack.get_type().clone())
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.hasItem
    pub fn has_item(&self) -> bool {
        self.get_item().is_some()
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.hasBlock
    pub fn has_block(&self) -> bool {
        self.block.is_some()
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.getHand
    ///
    /// `EquipmentSlot`'s name; a step has none.
    pub fn get_hand(&self) -> Option<&str> {
        if self.action == "PHYSICAL" {
            return None;
        }
        Some(self.value["hand"].as_str().unwrap_or("HAND"))
    }
    fn cursor(&self) -> Option<[f64; 3]> {
        let at = &self.value["cursor"];
        if self.value.get("kind").is_some() {
            return Some([at[0].as_f64()?, at[1].as_f64()?, at[2].as_f64()?]);
        }
        let at = query(json!({ "op": "ev_player_clicked", "uuid": self.player.uuid() }));
        let block = self.block.as_ref()?.get_location();
        Some([at[0].as_f64()? - block.x, at[1].as_f64()? - block.y, at[2].as_f64()? - block.z])
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.getClickedPosition
    pub fn get_clicked_position(&self) -> Option<crate::util::Vector> {
        self.cursor().map(|at| crate::util::Vector::new(at[0], at[1], at[2]))
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.getInteractionPoint
    pub fn get_interaction_point(&self) -> Option<Location> {
        let (at, block) = (self.cursor()?, self.block.as_ref()?);
        let corner = block.get_location();
        Some(Location::new(block.get_world().clone(), corner.x + at[0], corner.y + at[1], corner.z + at[2], 0.0, 0.0))
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.isBlockInHand
    pub fn is_block_in_hand(&self) -> bool {
        self.get_item().is_some_and(|stack| query(json!({ "op": "ev_player_is_block", "name": stack.get_type().key() })) == true)
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.useInteractedBlock
    ///
    /// `Result`'s name.
    pub fn use_interacted_block(&self) -> &str {
        &self.use_block
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.useItemInHand
    pub fn use_item_in_hand(&self) -> &str {
        &self.use_item
    }
    fn tell(&self) {
        crate::event::carry("use_block", json!(self.use_block));
        crate::event::carry("use_item", json!(self.use_item));
        query(json!({ "op": "ev_player_use", "block": self.use_block == "DENY", "item": self.use_item == "DENY" }));
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.setUseInteractedBlock
    pub fn set_use_interacted_block(&mut self, result: &str) {
        result.clone_into(&mut self.use_block);
        self.tell();
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.setUseItemInHand
    pub fn set_use_item_in_hand(&mut self, result: &str) {
        result.clone_into(&mut self.use_item);
        self.tell();
    }
    /// Both of the above.
    pub fn set_use(&mut self, result: &str) {
        result.clone_into(&mut self.use_block);
        result.clone_into(&mut self.use_item);
        self.tell();
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.use_block == "DENY"
    }
    /// bukkit: org/bukkit/event/player/PlayerInteractEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        if cancelled {
            self.set_use("DENY");
            return;
        }
        if self.use_block == "DENY" {
            "DEFAULT".clone_into(&mut self.use_block);
        }
        if self.use_item == "DENY" {
            "DEFAULT".clone_into(&mut self.use_item);
        }
        self.tell();
    }
}

/// bukkit: io/papermc/paper/event/packet/PlayerChunkLoadEvent
///
/// A chunk just sent to a player. Fired at the tick's end.
#[derive(Clone, Debug)]
pub struct PlayerChunkLoadEvent {
    player: Player,
    chunk: crate::entity::Chunk,
}

/// bukkit: io/papermc/paper/event/packet/PlayerChunkUnloadEvent
#[derive(Clone, Debug)]
pub struct PlayerChunkUnloadEvent {
    player: Player,
    chunk: crate::entity::Chunk,
}

/// What `ev_player_chunks` said this tick: it answers once, and both events read it.
static CHUNK_DELTAS: std::sync::Mutex<(u64, Value)> = std::sync::Mutex::new((u64::MAX, Value::Null));

fn chunk_deltas() -> Value {
    let now = crate::scheduler::now();
    {
        let deltas = CHUNK_DELTAS.lock().unwrap_or_else(|e| e.into_inner());
        if deltas.0 == now {
            return deltas.1.clone();
        }
    }
    let fresh = query(json!({ "op": "ev_player_chunks" }));
    *CHUNK_DELTAS.lock().unwrap_or_else(|e| e.into_inner()) = (now, fresh.clone());
    fresh
}

/// bukkit: io/papermc/paper/event/packet/PlayerChunkLoadEvent.getPlayer, io/papermc/paper/event/packet/PlayerChunkUnloadEvent.getPlayer
macro_rules! chunk_event {
    ($name:ident, $key:literal) => {
        impl Event for $name {
            const HOOK: &'static str = "tick";
            fn read(_: &Value) -> Option<Self> {
                None
            }
            fn read_all(_: &Value) -> Vec<Self> {
                let mut out = Vec::new();
                for row in chunk_deltas().as_array().into_iter().flatten() {
                    let (Some(player), world) = (Player::read(row), world_of(row)) else {
                        continue;
                    };
                    for chunk in row[$key].as_array().into_iter().flatten() {
                        if let (Some(x), Some(z)) = (chunk[0].as_i64(), chunk[1].as_i64()) {
                            out.push(Self { player: player.clone(), chunk: world.get_chunk_at(x as i32, z as i32) });
                        }
                    }
                }
                out
            }
        }
        impl $name {
            pub fn get_player(&self) -> &Player {
                &self.player
            }
            pub fn get_chunk(&self) -> &crate::entity::Chunk {
                &self.chunk
            }
        }
    };
}

chunk_event!(PlayerChunkLoadEvent, "loaded");
chunk_event!(PlayerChunkUnloadEvent, "unloaded");

/// bukkit: org/bukkit/event/player/PlayerRecipeBookSettingsChangeEvent
#[derive(Clone, Debug)]
pub struct PlayerRecipeBookSettingsChangeEvent {
    player: Player,
    book: usize,
    open: bool,
    filtering: bool,
}

impl Event for PlayerRecipeBookSettingsChangeEvent {
    const HOOK: &'static str = "pl_packet";
    fn read(value: &Value) -> Option<Self> {
        if value["kind"] != "book_settings" {
            return None;
        }
        Some(Self { player: player_of(value)?, book: value["book"].as_u64()? as usize, open: value["open"] == true, filtering: value["filtering"] == true })
    }
}

impl PlayerRecipeBookSettingsChangeEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeBookSettingsChangeEvent.isOpen
    pub fn is_open(&self) -> bool {
        self.open
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeBookSettingsChangeEvent.isFiltering
    pub fn is_filtering(&self) -> bool {
        self.filtering
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeBookSettingsChangeEvent.getRecipeBookType
    ///
    /// `RecipeBookType`'s name.
    pub fn get_recipe_book_type(&self) -> &'static str {
        ["CRAFTING", "FURNACE", "BLAST_FURNACE", "SMOKER"][self.book]
    }
}

/// `hideEntity`, `showEntity`, `hidePlayer` or `showPlayer` changed something: Paper tells at once.
pub(crate) fn visibility_changed(viewer: &Player, entity: u64, hidden: bool) {
    query(json!({ "op": "ev_player_hide_event", "uuid": viewer.uuid(), "name": viewer.get_name(), "entity": entity, "hidden": hidden }));
}

/// bukkit: org/bukkit/event/player/PlayerHideEntityEvent.getEntity, org/bukkit/event/player/PlayerShowEntityEvent.getEntity
macro_rules! hide_event {
    ($name:ident, $hidden:literal) => {
        impl Event for $name {
            const HOOK: &'static str = "pl_hide";
            fn read(value: &Value) -> Option<Self> {
                if value["hidden"] != $hidden {
                    return None;
                }
                let player = player_of(value)?;
                let entity = Entity::of(value["entity"].as_u64()?, &player.get_world());
                Some(Self { player, entity })
            }
        }
        impl $name {
            pub fn get_player(&self) -> &Player {
                &self.player
            }
            pub fn get_entity(&self) -> &Entity {
                &self.entity
            }
        }
    };
}

/// bukkit: org/bukkit/event/player/PlayerHideEntityEvent
#[derive(Clone, Debug)]
pub struct PlayerHideEntityEvent {
    player: Player,
    entity: Entity,
}

/// bukkit: org/bukkit/event/player/PlayerShowEntityEvent
#[derive(Clone, Debug)]
pub struct PlayerShowEntityEvent {
    player: Player,
    entity: Entity,
}

hide_event!(PlayerHideEntityEvent, true);
hide_event!(PlayerShowEntityEvent, false);

/// Lines the hub logs about a player's move, for the next tick.
/// bukkit: io/papermc/paper/event/player/PlayerFailMoveEvent
///
/// A move the server won't take, heard before it sends them back. Allowed, the move goes through.
#[derive(Clone, Debug)]
pub struct PlayerFailMoveEvent {
    player: Player,
    reason: String,
    from: Location,
    to: Location,
    allowed: bool,
    log: bool,
}

impl Event for PlayerFailMoveEvent {
    const HOOK: &'static str = "pl_fail_move";
    fn read(value: &Value) -> Option<Self> {
        let player = player_of(value)?;
        let world = player.get_world();
        let place = |at: &Value| Some(Location::new(world.clone(), at[0].as_f64()?, at[1].as_f64()?, at[2].as_f64()?, value["yaw"].as_f64()? as f32, value["pitch"].as_f64()? as f32));
        let (from, to) = (place(&value["from"])?, place(&value["to"])?);
        Some(Self { reason: value["reason"].as_str()?.to_owned(), allowed: value["allowed"] == true, log: value["log"] != false, player, from, to })
    }
}

impl PlayerFailMoveEvent {
    fn tell(&self) {
        crate::event::carry("allowed", json!(self.allowed));
        crate::event::carry("log", json!(self.log));
        query(json!({ "op": "ev_player_move_word", "allowed": self.allowed, "log": self.log }));
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFailMoveEvent.getFailReason
    ///
    /// `FailReason`'s name.
    pub fn get_fail_reason(&self) -> &str {
        &self.reason
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFailMoveEvent.getFrom
    pub fn get_from(&self) -> &Location {
        &self.from
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFailMoveEvent.getTo
    pub fn get_to(&self) -> &Location {
        &self.to
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFailMoveEvent.isAllowed
    pub fn is_allowed(&self) -> bool {
        self.allowed
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFailMoveEvent.setAllowed
    pub fn set_allowed(&mut self, allowed: bool) {
        self.allowed = allowed;
        self.tell();
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFailMoveEvent.getLogWarning
    pub fn get_log_warning(&self) -> bool {
        self.log
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFailMoveEvent.setLogWarning
    pub fn set_log_warning(&mut self, log: bool) {
        self.log = log;
        self.tell();
    }
}


// ---- Third pass ----

/// bukkit: org/bukkit/event/player/PlayerSignOpenEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerSignOpenEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerSignOpenEvent, "pl_sign_open", ["pl_sign_used"], |_value| true);

impl PlayerSignOpenEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerSignOpenEvent.getSign
    ///
    /// The sign's block.
    pub fn get_sign(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/player/PlayerSignOpenEvent.getSide
    ///
    /// `Side`'s name.
    pub fn get_side(&self) -> &'static str {
        if self.value["front"] == true { "FRONT" } else { "BACK" }
    }
    /// bukkit: org/bukkit/event/player/PlayerSignOpenEvent.getCause
    ///
    /// `Cause`'s name: `INTERACT` for a hand on the sign, `PLACE` for one just put down.
    pub fn get_cause(&self) -> &str {
        self.value["cause"].as_str().unwrap_or("UNKNOWN")
    }
    /// bukkit: org/bukkit/event/player/PlayerSignOpenEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerSignOpenEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerOpenSignEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerOpenSignEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerOpenSignEvent, "pl_sign_open", ["pl_sign_used"], |_value| true);

impl PlayerOpenSignEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerOpenSignEvent.getSign
    ///
    /// The sign's block.
    pub fn get_sign(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerOpenSignEvent.getSide
    ///
    /// `Side`'s name.
    pub fn get_side(&self) -> &'static str {
        if self.value["front"] == true { "FRONT" } else { "BACK" }
    }
    /// bukkit: io/papermc/paper/event/player/PlayerOpenSignEvent.getCause
    ///
    /// `Cause`'s name: `INTERACT` for a hand on the sign, `PLACE` for one just put down.
    pub fn get_cause(&self) -> &str {
        self.value["cause"].as_str().unwrap_or("UNKNOWN")
    }
    /// bukkit: io/papermc/paper/event/player/PlayerOpenSignEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerOpenSignEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerSignCommandPreprocessEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerSignCommandPreprocessEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerSignCommandPreprocessEvent, "pl_sign_command", ["pl_sign_click"], |_value| true);

impl PlayerSignCommandPreprocessEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// The command.
    pub fn get_message(&self) -> &str {
        self.value["command"].as_str().unwrap_or_default()
    }
    /// bukkit: io/papermc/paper/event/player/PlayerSignCommandPreprocessEvent.getSign
    ///
    /// The sign's block.
    pub fn get_sign(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerSignCommandPreprocessEvent.getSide
    ///
    /// `Side`'s name.
    pub fn get_side(&self) -> &'static str {
        if self.value["front"] == true { "FRONT" } else { "BACK" }
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerAdvancementDoneEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerAdvancementDoneEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerAdvancementDoneEvent, "pl_advancement", |_value| true);

impl PlayerAdvancementDoneEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerAdvancementDoneEvent.getAdvancement
    pub fn get_advancement(&self) -> Option<crate::core2::Advancement> {
        crate::core2::get_advancement(self.value["advancement"].as_str()?)
    }
    /// bukkit: org/bukkit/event/player/PlayerAdvancementDoneEvent.message
    ///
    /// The line the game will say: `None` for an advancement that isn't announced.
    pub fn message(&self) -> Option<crate::Component> {
        (!self.value["message"].is_null()).then(|| crate::Component(self.value["message"].clone()))
    }
    /// bukkit: org/bukkit/event/player/PlayerAdvancementDoneEvent.message
    ///
    /// What's said in its place: `None` says nothing.
    pub fn set_message(&mut self, message: Option<&crate::Component>) {
        self.value["message"] = message.map_or(Value::Null, |message| message.0.clone());
        query(json!({ "op": "ev_player_award_line", "line": message.map(crate::Component::to_legacy) }));
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerNameEntityEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerNameEntityEvent {
    value: Value,
    cancelled: bool,
    persistent: Option<bool>,
    other: Option<u64>,
}

hooked!(PlayerNameEntityEvent, "pl_name", |_value| true);

impl PlayerNameEntityEvent {
    fn send(&self) {
        query(json!({ "op": "ev_player_name", "new_name": self.value["new_name"], "persistent": self.persistent, "entity": self.other }));
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerNameEntityEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        Some(Entity::of(self.other.or(self.value["entity"].as_u64())?, &world_of(&self.value)))
    }
    /// bukkit: io/papermc/paper/event/player/PlayerNameEntityEvent.setEntity
    ///
    /// The mob that gets the name.
    pub fn set_entity(&mut self, entity_id: u64) {
        self.other = Some(entity_id);
        self.send();
    }
    /// bukkit: io/papermc/paper/event/player/PlayerNameEntityEvent.getName
    ///
    /// The name, as the tag holds it.
    pub fn get_name(&self) -> Option<&str> {
        self.value["new_name"].as_str()
    }
    /// bukkit: io/papermc/paper/event/player/PlayerNameEntityEvent.setName
    pub fn set_name(&mut self, name: &crate::Component) {
        self.value["new_name"] = json!(name.to_legacy());
        self.send();
    }
    /// bukkit: io/papermc/paper/event/player/PlayerNameEntityEvent.isPersistent
    ///
    /// A named mob stays, unless this says otherwise.
    pub fn is_persistent(&self) -> bool {
        self.persistent.unwrap_or(true)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerNameEntityEvent.setPersistent
    pub fn set_persistent(&mut self, persistent: bool) {
        self.persistent = Some(persistent);
        self.send();
    }
    /// bukkit: io/papermc/paper/event/player/PlayerNameEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerNameEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerMapFilledEvent
///
/// Once the empty map is filled, as in Paper: it can't be cancelled.
#[derive(Clone, Debug, Default)]
pub struct PlayerMapFilledEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerMapFilledEvent, "pl_map_fill", |_value| true);

impl PlayerMapFilledEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerMapFilledEvent.getOriginalItem
    pub fn get_original_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerMapFilledEvent.getCreatedMap
    pub fn get_created_map(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["made"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerMapFilledEvent.setCreatedMap
    pub fn set_created_map(&mut self, map: &ItemStack) {
        self.value["made"] = map.to_json();
        query(json!({ "op": "ev_player_map", "stack": snbt_of(Some(map)) }));
    }
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
}

/// bukkit: org/bukkit/event/player/PlayerRiptideEvent
///
/// As a Riptide trident lets go and the game is about to launch them. It can't be cancelled, as in Bukkit.
#[derive(Clone, Debug, Default)]
pub struct PlayerRiptideEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerRiptideEvent, "pl_riptide", |_value| true);

impl PlayerRiptideEvent {
    /// bukkit: org/bukkit/event/player/PlayerRiptideEvent.setCancelled
    ///
    /// They stay where they are, and the trident's use ends as if there was no riptide.
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerRiptideEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerRiptideEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: org/bukkit/event/player/PlayerRiptideEvent.getVelocity
    ///
    /// The push it adds.
    pub fn get_velocity(&self) -> crate::util::Vector {
        let at = |i: usize| self.value["velocity"][i].as_f64().unwrap_or(0.0);
        crate::util::Vector::new(at(0), at(1), at(2))
    }
}

/// bukkit: org/bukkit/event/player/PlayerVelocityEvent
///
/// A blow's knockback, worked out by the game: a cancel leaves the player's motion as it was.
#[derive(Clone, Debug, Default)]
pub struct PlayerVelocityEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerVelocityEvent, "pl_velocity", |_value| true);

impl PlayerVelocityEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerVelocityEvent.getVelocity
    pub fn get_velocity(&self) -> crate::util::Vector {
        let at = |i: usize| self.value["velocity"][i].as_f64().unwrap_or(0.0);
        crate::util::Vector::new(at(0), at(1), at(2))
    }
    /// bukkit: org/bukkit/event/player/PlayerVelocityEvent.setVelocity
    pub fn set_velocity(&mut self, velocity: &crate::util::Vector) {
        let at = [velocity.get_x(), velocity.get_y(), velocity.get_z()];
        self.value["velocity"] = json!(at);
        query(json!({ "op": "ev_player_velocity", "velocity": at }));
    }
    /// bukkit: org/bukkit/event/player/PlayerVelocityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerVelocityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerInsertLecternBookEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerInsertLecternBookEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerInsertLecternBookEvent, "pl_lectern_put", |_value| true);

impl PlayerInsertLecternBookEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerInsertLecternBookEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerInsertLecternBookEvent.getLectern
    pub fn get_lectern(&self) -> Option<Block> {
        self.get_block()
    }
    /// bukkit: io/papermc/paper/event/player/PlayerInsertLecternBookEvent.getBook
    pub fn get_book(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["book"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerInsertLecternBookEvent.setBook
    ///
    /// What goes on the lectern in its place.
    pub fn set_book(&mut self, book: &ItemStack) {
        self.value["book"] = book.to_json();
        query(json!({ "op": "ev_player_book_in", "stack": snbt_of(Some(book)) }));
    }
    /// bukkit: io/papermc/paper/event/player/PlayerInsertLecternBookEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerInsertLecternBookEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerItemFrameChangeEvent
///
/// A player's click. A cancel leaves the frame as it was.
#[derive(Clone, Debug, Default)]
pub struct PlayerItemFrameChangeEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerItemFrameChangeEvent, "pl_frame", ["pl_frame_hit"], |_value| true);

impl PlayerItemFrameChangeEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerItemFrameChangeEvent.getAction
    ///
    /// `ItemFrameChangeAction`'s name.
    pub fn get_action(&self) -> &str {
        self.value["frame"]["action"].as_str().unwrap_or_default()
    }
    /// bukkit: io/papermc/paper/event/player/PlayerItemFrameChangeEvent.getItemStack
    pub fn get_item_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["frame"]["item"])
    }
    /// bukkit: io/papermc/paper/event/player/PlayerItemFrameChangeEvent.setItemStack
    ///
    /// What the frame holds after this, or what's taken out of it.
    pub fn set_item_stack(&mut self, stack: Option<&ItemStack>) {
        self.value["frame"]["item"] = stack.map_or(Value::Null, ItemStack::to_json);
        query(json!({ "op": "ev_player_frame", "stack": snbt_of(stack) }));
    }
    /// bukkit: io/papermc/paper/event/player/PlayerItemFrameChangeEvent.getItemFrame
    pub fn get_item_frame(&self) -> Option<Entity> {
        Some(Entity::of(self.value["frame"]["entity"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: io/papermc/paper/event/player/PlayerItemFrameChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerItemFrameChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerClientLoadedWorldEvent
#[derive(Clone, Debug)]
pub struct PlayerClientLoadedWorldEvent {
    player: Player,
    timeout: bool,
}

impl Event for PlayerClientLoadedWorldEvent {
    const HOOK: &'static str = "tick";
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(_: &Value) -> Vec<Self> {
        query(json!({ "op": "ev_player_loaded" })).as_array().into_iter().flatten().filter_map(|row| Some(Self { player: Player::read(row)?, timeout: row["timeout"] == true })).collect()
    }
}

impl PlayerClientLoadedWorldEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: io/papermc/paper/event/player/PlayerClientLoadedWorldEvent.isTimeout
    pub fn is_timeout(&self) -> bool {
        self.timeout
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerConnectionCloseEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerConnectionCloseEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerConnectionCloseEvent, "player_leave", ["pl_packet", "pl_conn_close"], |value| value.get("kind").is_none());

impl PlayerConnectionCloseEvent {
    /// bukkit: com/destroystokyo/paper/event/player/PlayerConnectionCloseEvent.getPlayerName
    pub fn get_player_name(&self) -> &str {
        self.value["name"].as_str().unwrap_or_default()
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerConnectionCloseEvent.getPlayerUniqueId
    pub fn get_player_unique_id(&self) -> &str {
        self.value["uuid"].as_str().unwrap_or_default()
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerConnectionCloseEvent.getSocketAddress
    ///
    /// The address and port.
    pub fn get_socket_address(&self) -> Option<String> {
        // A login that never got to play says where it was: no game to ask then.
        self.value["address"].as_str().map(str::to_owned).or_else(|| query(json!({ "op": "ev_player_socket", "uuid": self.value["uuid"] })).as_str().map(str::to_owned))
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerConnectionCloseEvent.getIpAddress
    ///
    /// The address.
    pub fn get_ip_address(&self) -> Option<String> {
        self.get_socket_address().and_then(|socket| socket.rsplit_once(':').map(|(ip, _)| ip.to_owned()))
    }
}

/// bukkit~: io/papermc/paper/event/player/AsyncPlayerSpawnLocationEvent | the chunk is waited for only when its level is the open one at the turn, for 15 s at most
///
/// On the connection's thread, in configuration, which holds until the listeners are done.
#[derive(Clone, Debug, Default)]
pub struct AsyncPlayerSpawnLocationEvent {
    value: Value,
    to: Option<Location>,
}

impl Event for AsyncPlayerSpawnLocationEvent {
    const HOOK: &'static str = "pl_spawn_location";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), to: None })
    }
    fn finish(&self) {
        if let Some(to) = &self.to {
            crate::event::carry("to", place_json(to));
            crate::event::carry("to_level", json!(to.world.dimension()));
        }
    }
    fn is_asynchronous(&self) -> bool {
        true
    }
}

impl AsyncPlayerSpawnLocationEvent {
    /// bukkit: io/papermc/paper/event/player/AsyncPlayerSpawnLocationEvent.getConnection
    ///
    /// The connection: only what's in it can be read, as there's no game to ask.
    pub fn get_connection(&self) -> crate::fam_paper::PlayerConfigurationConnection {
        crate::fam_paper::PlayerConfigurationConnection(self.value.clone())
    }
    /// bukkit: io/papermc/paper/event/player/AsyncPlayerSpawnLocationEvent.getSpawnLocation
    pub fn get_spawn_location(&self) -> Location {
        self.to.clone().unwrap_or_else(|| {
            let at = &self.value["at"];
            let number = |i: usize| at[i].as_f64().unwrap_or(0.0);
            let world = self.value["to_level"].as_str().map_or_else(|| world_of(&self.value), World::of);
            Location::new(world, number(0), number(1), number(2), self.value["yaw"].as_f64().unwrap_or(0.0) as f32, self.value["pitch"].as_f64().unwrap_or(0.0) as f32)
        })
    }
    /// bukkit: io/papermc/paper/event/player/AsyncPlayerSpawnLocationEvent.setSpawnLocation
    pub fn set_spawn_location(&mut self, to: &Location) {
        self.to = Some(to.clone());
    }
    /// bukkit: io/papermc/paper/event/player/AsyncPlayerSpawnLocationEvent.isNewPlayer
    ///
    /// Whether they've never played here.
    pub fn is_new_player(&self) -> bool {
        self.value["new"] == true
    }
}

fn spawn_at(row: &Value) -> Option<Location> {
    let at = &row["at"];
    Some(Location::new(World::of(row["level"].as_str()?), at[0].as_f64()?, at[1].as_f64()?, at[2].as_f64()?, 0.0, 0.0))
}

/// bukkit: org/bukkit/event/player/PlayerSpawnChangeEvent.setForced, org/bukkit/event/player/PlayerSpawnChangeEvent.isForced, org/bukkit/event/player/PlayerSpawnChangeEvent.setCancelled, org/bukkit/event/player/PlayerSpawnChangeEvent.isCancelled, org/bukkit/event/player/PlayerSpawnChangeEvent.getCause, com/destroystokyo/paper/event/player/PlayerSetSpawnEvent.setForced, com/destroystokyo/paper/event/player/PlayerSetSpawnEvent.isForced, com/destroystokyo/paper/event/player/PlayerSetSpawnEvent.setCancelled, com/destroystokyo/paper/event/player/PlayerSetSpawnEvent.isCancelled, com/destroystokyo/paper/event/player/PlayerSetSpawnEvent.getCause
macro_rules! spawn_event {
    ($name:ident) => {
        impl Event for $name {
            const HOOK: &'static str = "pl_spawn_set";
            fn read(value: &Value) -> Option<Self> {
                Some(Self { player: player_of(value)?, value: value.clone(), to: None, cancelled: crate::event::prior_cancelled(), notify: None, line: None, forced: None })
            }
            fn answer(&self) -> Answer {
                answer_of(self.cancelled)
            }
            fn finish(&self) {
                let at = self.to.as_ref().map(|to| json!([to.x.floor() as i32, to.y.floor() as i32, to.z.floor() as i32]));
                let level = self.to.as_ref().map(|to| to.world.dimension());
                query(json!({ "op": "ev_player_spawn_answer", "at": at, "level": level, "notify": self.notify, "line": self.line.as_ref().and_then(|line| line.as_ref()).map(|line| line.to_legacy()), "forced": self.forced }));
            }
        }
        impl $name {
            pub fn get_player(&self) -> &Player {
                &self.player
            }
            /// The point stays what it was.
            pub fn set_cancelled(&mut self, cancelled: bool) {
                self.cancelled = cancelled;
            }
            pub fn is_cancelled(&self) -> bool {
                self.cancelled
            }
            /// `Cause`'s name: a plugin's own `setRespawnLocation` is a command here.
            pub fn get_cause(&self) -> &str {
                self.value["cause"].as_str().unwrap_or("UNKNOWN")
            }
            pub fn is_forced(&self) -> bool {
                self.forced.unwrap_or(self.value["forced"] == true)
            }
            pub fn set_forced(&mut self, forced: bool) {
                self.forced = Some(forced);
            }
        }
    };
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerSetSpawnEvent
///
/// A bed, an anchor or `/spawnpoint` about to set a player's spawn point.
#[derive(Clone, Debug)]
pub struct PlayerSetSpawnEvent {
    player: Player,
    value: Value,
    to: Option<Location>,
    cancelled: bool,
    notify: Option<bool>,
    line: Option<Option<crate::Component>>,
    forced: Option<bool>,
}

/// bukkit: org/bukkit/event/player/PlayerSpawnChangeEvent
///
/// The same moment as `PlayerSetSpawnEvent`.
#[derive(Clone, Debug)]
pub struct PlayerSpawnChangeEvent {
    player: Player,
    value: Value,
    to: Option<Location>,
    cancelled: bool,
    notify: Option<bool>,
    line: Option<Option<crate::Component>>,
    forced: Option<bool>,
}

spawn_event!(PlayerSetSpawnEvent);
spawn_event!(PlayerSpawnChangeEvent);

impl PlayerSetSpawnEvent {
    /// bukkit: com/destroystokyo/paper/event/player/PlayerSetSpawnEvent.getLocation
    pub fn get_location(&self) -> Option<Location> {
        self.to.clone().or_else(|| spawn_at(&self.value))
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerSetSpawnEvent.setLocation
    pub fn set_location(&mut self, to: &Location) {
        self.to = Some(to.clone());
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerSetSpawnEvent.getNotification
    ///
    /// What they'll be told, if anything.
    pub fn get_notification(&self) -> Option<crate::Component> {
        match &self.line {
            Some(line) => line.clone(),
            None => self.will_notify_player().then(|| crate::Component(json!({ "translate": "block.minecraft.set_spawn" }))),
        }
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerSetSpawnEvent.setNotification
    ///
    /// `None` tells them nothing.
    pub fn set_notification(&mut self, line: Option<&crate::Component>) {
        self.line = Some(line.cloned());
        self.notify = Some(line.is_some());
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerSetSpawnEvent.willNotifyPlayer
    pub fn will_notify_player(&self) -> bool {
        self.notify.unwrap_or(self.value["notify"] == true)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerSetSpawnEvent.setNotifyPlayer
    pub fn set_notify_player(&mut self, notify: bool) {
        self.notify = Some(notify);
    }
}

impl PlayerSpawnChangeEvent {
    /// bukkit: org/bukkit/event/player/PlayerSpawnChangeEvent.getNewSpawn
    pub fn get_new_spawn(&self) -> Option<Location> {
        self.to.clone().or_else(|| spawn_at(&self.value))
    }
    /// bukkit: org/bukkit/event/player/PlayerSpawnChangeEvent.setNewSpawn
    pub fn set_new_spawn(&mut self, to: &Location) {
        self.to = Some(to.clone());
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerArmorChangeEvent$SlotType.getTypes
///
/// The items that go in `slot` (`HEAD`, `CHEST`, `LEGS`, `FEET`), by key.
pub fn armor_slot_types(slot: &str) -> Vec<String> {
    query(json!({ "op": "ev_player_equip_items", "slot": slot.to_lowercase() })).as_array().into_iter().flatten().filter_map(|item| item.as_str().map(str::to_owned)).collect()
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerArmorChangeEvent$SlotType.getByMaterial
pub fn armor_slot_of(material: &crate::Material) -> Option<String> {
    let slot = query(json!({ "op": "ev_player_equip", "item": material.key() }));
    slot.as_str().filter(|slot| matches!(*slot, "head" | "chest" | "legs" | "feet")).map(str::to_uppercase)
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerArmorChangeEvent$SlotType.isEquipable
pub fn armor_slot_takes(slot: &str, material: &crate::Material) -> bool {
    armor_slot_of(material).as_deref() == Some(slot)
}

/// bukkit: org/bukkit/event/inventory/InventoryType.getMenuType
///
/// The menu of an `InventoryType`, by its key.
pub fn menu_type(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "CHEST" | "BARREL" | "ENDER_CHEST" => "minecraft:generic_9x3",
        "DISPENSER" | "DROPPER" => "minecraft:generic_3x3",
        "CRAFTER" => "minecraft:crafter_3x3",
        "FURNACE" => "minecraft:furnace",
        "BLAST_FURNACE" => "minecraft:blast_furnace",
        "SMOKER" => "minecraft:smoker",
        "WORKBENCH" => "minecraft:crafting",
        "ENCHANTING" => "minecraft:enchantment",
        "BREWING" => "minecraft:brewing_stand",
        "MERCHANT" => "minecraft:merchant",
        "ANVIL" => "minecraft:anvil",
        "SMITHING" => "minecraft:smithing",
        "BEACON" => "minecraft:beacon",
        "HOPPER" => "minecraft:hopper",
        "SHULKER_BOX" => "minecraft:shulker_box",
        "LECTERN" => "minecraft:lectern",
        "LOOM" => "minecraft:loom",
        "CARTOGRAPHY" => "minecraft:cartography_table",
        "GRINDSTONE" => "minecraft:grindstone",
        "STONECUTTER" => "minecraft:stonecutter",
        _ => return None,
    })
}

/// bukkit: io/papermc/paper/event/player/AsyncChatDecorateEvent
#[derive(Clone, Debug, Default)]
pub struct AsyncChatDecorateEvent {
    value: Value,
    cancelled: bool,
    result: Option<crate::Component>,
}

impl Event for AsyncChatDecorateEvent {
    const HOOK: &'static str = "pl_decorate";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_packet"]
    }
    fn read(value: &Value) -> Option<Self> {
        (!value["message"].is_null()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), result: None })
    }
    fn finish(&self) {
        if let (false, Some(result)) = (self.cancelled, &self.result) {
            horizon_mod::tell("ev_player_decorate", &result.to_legacy());
        }
    }
}

impl AsyncChatDecorateEvent {
    /// bukkit: io/papermc/paper/event/player/AsyncChatDecorateEvent.player
    pub fn player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/AsyncChatDecorateEvent.originalMessage
    pub fn original_message(&self) -> crate::Component {
        crate::Component(self.value["message"].clone())
    }
    /// bukkit: io/papermc/paper/event/player/AsyncChatDecorateEvent.result
    pub fn result(&self) -> crate::Component {
        self.result.clone().unwrap_or_else(|| self.original_message())
    }
    pub fn set_result(&mut self, result: &crate::Component) {
        self.result = Some(result.clone());
    }
    /// bukkit: io/papermc/paper/event/player/AsyncChatDecorateEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/AsyncChatDecorateEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Fourth pass ----

/// bukkit: com/destroystokyo/paper/event/player/PlayerElytraBoostEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerElytraBoostEvent {
    value: Value,
    cancelled: bool,
    keep: bool,
}

hooked!(PlayerElytraBoostEvent, "pl_boost", |_value| true);

impl PlayerElytraBoostEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerElytraBoostEvent.getItemStack
    pub fn get_item_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerElytraBoostEvent.getFirework
    ///
    /// The rocket, by the id it will have: it isn't in the world yet, as in Paper.
    pub fn get_firework(&self) -> Option<Entity> {
        Some(Entity::of(self.value["entity"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerElytraBoostEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerElytraBoostEvent.shouldConsume
    pub fn should_consume(&self) -> bool {
        !self.keep
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerElytraBoostEvent.setShouldConsume
    pub fn set_should_consume(&mut self, consume: bool) {
        self.keep = !consume;
        query(json!({ "op": "ev_player_boost_keep", "keep": !consume }));
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerElytraBoostEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerElytraBoostEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerSwapHandItemsEvent
///
/// The swap key. A cancel leaves both hands as they were, and the client is told.
#[derive(Clone, Debug, Default)]
pub struct PlayerSwapHandItemsEvent {
    value: Value,
    cancelled: bool,
    main: Option<Option<ItemStack>>,
    off: Option<Option<ItemStack>>,
}

held_event!(PlayerSwapHandItemsEvent, |value| value["kind"] == "swap");

impl PlayerSwapHandItemsEvent {
    fn send(&self) {
        query(json!({ "op": "ev_player_hands", "main": snbt_of(self.get_main_hand_item().as_ref()), "off": snbt_of(self.get_off_hand_item().as_ref()) }));
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerSwapHandItemsEvent.getMainHandItem
    ///
    /// What will be in the main hand: the off hand's stack.
    pub fn get_main_hand_item(&self) -> Option<ItemStack> {
        match &self.main {
            Some(stack) => stack.clone(),
            None => ItemStack::read(&self.value["off_hand"]),
        }
    }
    /// bukkit: org/bukkit/event/player/PlayerSwapHandItemsEvent.getOffHandItem
    ///
    /// What will be in the off hand: the main hand's stack.
    pub fn get_off_hand_item(&self) -> Option<ItemStack> {
        match &self.off {
            Some(stack) => stack.clone(),
            None => ItemStack::read(&self.value["stack"]),
        }
    }
    /// bukkit: org/bukkit/event/player/PlayerSwapHandItemsEvent.setMainHandItem
    pub fn set_main_hand_item(&mut self, stack: Option<&ItemStack>) {
        self.main = Some(stack.cloned());
        self.send();
    }
    /// bukkit: org/bukkit/event/player/PlayerSwapHandItemsEvent.setOffHandItem
    pub fn set_off_hand_item(&mut self, stack: Option<&ItemStack>) {
        self.off = Some(stack.cloned());
        self.send();
    }
    /// bukkit: org/bukkit/event/player/PlayerSwapHandItemsEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerSwapHandItemsEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: com/destroystokyo/paper/event/player/PlayerArmorChangeEvent
#[derive(Clone, Debug, Default)]
pub struct PlayerArmorChangeEvent {
    value: Value,
    cancelled: bool,
}

hooked!(PlayerArmorChangeEvent, "pl_armor", |value| matches!(value["slot"].as_str(), Some("head" | "chest" | "legs" | "feet")));

impl PlayerArmorChangeEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerArmorChangeEvent.getSlot
    ///
    /// `EquipmentSlot`'s name.
    pub fn get_slot(&self) -> String {
        self.value["slot"].as_str().unwrap_or_default().to_uppercase()
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerArmorChangeEvent.getSlotType
    ///
    /// `SlotType`'s name.
    pub fn get_slot_type(&self) -> String {
        self.get_slot()
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerArmorChangeEvent.getNewItem
    pub fn get_new_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["new"])
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerArmorChangeEvent.getOldItem
    pub fn get_old_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["old"])
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerStopUsingItemEvent
///
/// A client letting go of the use key with anything in use.
#[derive(Clone, Debug, Default)]
pub struct PlayerStopUsingItemEvent {
    value: Value,
    cancelled: bool,
}

held_event!(PlayerStopUsingItemEvent, |value| value["kind"] == "stop_use");

impl PlayerStopUsingItemEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerStopUsingItemEvent.getTicksHeldFor
    pub fn get_ticks_held_for(&self) -> i64 {
        self.value["ticks"].as_i64().unwrap_or(0)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerStopUsingItemEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
}

// ---- Moves and teleports, before they count ----

fn place_of(value: &Value, key: &str) -> Option<Location> {
    let at = &value[key];
    Some(Location::new(world_of(value), at[0].as_f64()?, at[1].as_f64()?, at[2].as_f64()?, at[3].as_f64()? as f32, at[4].as_f64()? as f32))
}

fn place_json(at: &Location) -> Value {
    json!([at.x, at.y, at.z, at.yaw, at.pitch])
}

/// bukkit: org/bukkit/event/player/PlayerMoveEvent
///
/// A move the hub has just read from a client, before the game does anything with it: others
/// haven't seen it, and a cancel or `set_to` sends the client back in the same tick.
pub struct PlayerMoveEvent {
    pub player: Player,
    from: Location,
    to: Location,
    cancelled: bool,
    changed: bool,
}

impl Event for PlayerMoveEvent {
    const HOOK: &'static str = "pl_moves";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_move_turn"]
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { player: player_of(value)?, from: place_of(value, "from")?, to: place_of(value, "to")?, cancelled: crate::event::prior_cancelled(), changed: false })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
    fn finish(&self) {
        if self.cancelled {
            query(json!({ "op": "ev_player_move_to", "to": place_json(&self.from) }));
        } else if self.changed {
            query(json!({ "op": "ev_player_move_to", "to": place_json(&self.to) }));
        }
    }
}

impl PlayerMoveEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/event/player/PlayerMoveEvent.getFrom
    pub fn get_from(&self) -> &Location {
        &self.from
    }
    /// bukkit: org/bukkit/event/player/PlayerMoveEvent.setFrom
    ///
    /// Where a cancel puts them.
    pub fn set_from(&mut self, from: &Location) {
        self.from = from.clone();
    }
    /// bukkit: org/bukkit/event/player/PlayerMoveEvent.getTo
    pub fn get_to(&self) -> &Location {
        &self.to
    }
    /// bukkit: org/bukkit/event/player/PlayerMoveEvent.setTo
    ///
    /// Where they end up: the client is told, in this tick.
    pub fn set_to(&mut self, to: &Location) {
        self.to = to.clone();
        self.changed = true;
        crate::event::carry("to", place_json(to));
    }
    /// bukkit: org/bukkit/event/player/PlayerMoveEvent.hasChangedPosition
    pub fn has_changed_position(&self) -> bool {
        self.from.x != self.to.x || self.from.y != self.to.y || self.from.z != self.to.z
    }
    /// bukkit: org/bukkit/event/player/PlayerMoveEvent.hasExplicitlyChangedPosition
    pub fn has_explicitly_changed_position(&self) -> bool {
        self.has_changed_position()
    }
    /// bukkit: org/bukkit/event/player/PlayerMoveEvent.hasChangedBlock
    pub fn has_changed_block(&self) -> bool {
        let block = |at: &Location| (at.x.floor(), at.y.floor(), at.z.floor());
        block(&self.from) != block(&self.to)
    }
    /// bukkit: org/bukkit/event/player/PlayerMoveEvent.hasExplicitlyChangedBlock
    pub fn has_explicitly_changed_block(&self) -> bool {
        self.has_changed_block()
    }
    /// bukkit: org/bukkit/event/player/PlayerMoveEvent.hasChangedOrientation
    pub fn has_changed_orientation(&self) -> bool {
        self.from.yaw != self.to.yaw || self.from.pitch != self.to.pitch
    }
    /// bukkit: org/bukkit/event/player/PlayerMoveEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerMoveEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/player/PlayerTeleportEvent
///
/// Before the teleport: `/tp` and `Player.teleport`, ender pearls, chorus fruit and a spectator's
/// jump to another player.
pub struct PlayerTeleportEvent {
    pub player: Player,
    from: Location,
    to: Location,
    cause: String,
    cancelled: bool,
    changed: bool,
}

impl Event for PlayerTeleportEvent {
    const HOOK: &'static str = "pl_teleport";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_tp_command", "pl_pearl", "pl_chorus", "pl_spectate"]
    }
    fn read(value: &Value) -> Option<Self> {
        let mut to = place_of(value, "to")?;
        if let Some(level) = value["to_level"].as_str() {
            to.world = World::of(level);
        }
        Some(Self { player: player_of(value)?, from: place_of(value, "from")?, to, cause: value["cause"].as_str().unwrap_or("UNKNOWN").to_owned(), cancelled: crate::event::prior_cancelled(), changed: false })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
    fn finish(&self) {
        if self.cancelled || !self.changed {
            return;
        }
        query(json!({ "op": "ev_player_tp_to", "to": place_json(&self.to), "level": self.to.world.dimension() }));
    }
}

impl PlayerTeleportEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    pub fn get_from(&self) -> &Location {
        &self.from
    }
    pub fn set_from(&mut self, from: &Location) {
        self.from = from.clone();
    }
    pub fn get_to(&self) -> &Location {
        &self.to
    }
    /// bukkit: org/bukkit/event/player/PlayerTeleportEvent.setTo
    pub fn set_to(&mut self, to: &Location) {
        self.to = to.clone();
        self.changed = true;
        crate::event::carry("to", place_json(to));
        crate::event::carry("to_level", json!(to.world.dimension()));
    }
    /// bukkit: org/bukkit/event/player/PlayerTeleportEvent.getCause
    pub fn get_cause(&self) -> &str {
        &self.cause
    }
    /// bukkit: org/bukkit/event/player/PlayerTeleportEvent.willDismountPlayer
    ///
    /// The game takes a teleported player off what they ride.
    pub fn will_dismount_player(&self) -> bool {
        true
    }
    /// bukkit: org/bukkit/event/player/PlayerTeleportEvent.getRelativeTeleportationFlags
    ///
    /// The game's teleports here give a whole position.
    pub fn get_relative_teleportation_flags(&self) -> Vec<String> {
        Vec::new()
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Picking things up ----

/// bukkit: org/bukkit/event/player/PlayerPickupItemEvent.getItem, org/bukkit/event/player/PlayerPickupItemEvent.getRemaining, org/bukkit/event/player/PlayerPickupItemEvent.getFlyAtPlayer, org/bukkit/event/player/PlayerPickupItemEvent.setFlyAtPlayer, org/bukkit/event/player/PlayerPickupItemEvent.setCancelled, org/bukkit/event/player/PlayerPickupItemEvent.isCancelled
/// bukkit: org/bukkit/event/player/PlayerAttemptPickupItemEvent.getItem, org/bukkit/event/player/PlayerAttemptPickupItemEvent.getRemaining, org/bukkit/event/player/PlayerAttemptPickupItemEvent.getFlyAtPlayer, org/bukkit/event/player/PlayerAttemptPickupItemEvent.setFlyAtPlayer, org/bukkit/event/player/PlayerAttemptPickupItemEvent.setCancelled, org/bukkit/event/player/PlayerAttemptPickupItemEvent.isCancelled
macro_rules! pickup_methods {
    ($name:ident) => {
        impl $name {
            pub fn get_player(&self) -> Option<Player> {
                player_of(&self.value)
            }
            /// The stack on the ground.
            pub fn get_item(&self) -> Option<ItemStack> {
                ItemStack::read(&self.value["stack"])
            }
            /// The item entity.
            pub fn get_item_entity(&self) -> Option<Entity> {
                Some(Entity::of(self.value["entity"].as_u64()?, &world_of(&self.value)))
            }
            /// What won't fit, and stays on the ground.
            pub fn get_remaining(&self) -> i32 {
                self.value["remaining"].as_i64().unwrap_or(0) as i32
            }
            /// Whether a refused item still flies at them. Bukkit sets this to the opposite of a
            /// cancel, and a plugin can set it again after.
            pub fn get_fly_at_player(&self) -> bool {
                self.fly
            }
            pub fn set_fly_at_player(&mut self, fly: bool) {
                self.fly = fly;
                query(json!({ "op": "ev_player_fly", "fly": fly }));
            }
            pub fn set_cancelled(&mut self, cancelled: bool) {
                self.cancelled = cancelled;
                self.set_fly_at_player(!cancelled);
            }
            pub fn is_cancelled(&self) -> bool {
                self.cancelled
            }
        }
    };
}

/// bukkit: org/bukkit/event/player/PlayerAttemptPickupItemEvent
///
/// Every stack touching a player, whether it fits or not. A cancel leaves it for 10 ticks.
#[derive(Clone, Debug)]
pub struct PlayerAttemptPickupItemEvent {
    value: Value,
    cancelled: bool,
    fly: bool,
}

/// bukkit: org/bukkit/event/player/PlayerPickupItemEvent
///
/// A stack touching a player that at least partly fits.
#[derive(Clone, Debug)]
pub struct PlayerPickupItemEvent {
    value: Value,
    cancelled: bool,
    fly: bool,
}

/// bukkit: org/bukkit/event/player/PlayerPickupArrowEvent
///
/// An arrow or trident that would go into a player's inventory. A cancel leaves it there.
#[derive(Clone, Debug)]
pub struct PlayerPickupArrowEvent {
    value: Value,
    cancelled: bool,
    fly: bool,
}

impl Event for PlayerAttemptPickupItemEvent {
    const HOOK: &'static str = "pl_pickup";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_moves", "pl_move_turn"]
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "attempt").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), fly: true })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

impl Event for PlayerPickupItemEvent {
    const HOOK: &'static str = "pl_pickup";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_moves", "pl_move_turn"]
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "pickup").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), fly: true })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

impl Event for PlayerPickupArrowEvent {
    const HOOK: &'static str = "pl_pickup_arrow";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_pickup_trident", "pl_moves", "pl_move_turn"]
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), fly: true })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

pickup_methods!(PlayerAttemptPickupItemEvent);
pickup_methods!(PlayerPickupItemEvent);
pickup_methods!(PlayerPickupArrowEvent);

impl PlayerPickupArrowEvent {
    /// bukkit: org/bukkit/event/player/PlayerPickupArrowEvent.getArrow
    pub fn get_arrow(&self) -> Option<Entity> {
        self.get_item_entity()
    }
}

// ---- Menus opening ----

/// bukkit: org/bukkit/event/inventory/InventoryOpenEvent
///
/// Before anything happens to the chest or goes to the client: every menu a player opens. A cancel
/// means none of it does.
#[derive(Clone, Debug, Default)]
pub struct InventoryOpenEvent {
    value: Value,
    cancelled: bool,
    title: Option<crate::Component>,
}

// `pl_held` is where it fires from; `pl_packet` and the next four are how the window is found.
hooked!(InventoryOpenEvent, "pl_open", ["pl_open_native", "pl_open_mount", "pl_open_boat", "pl_open_cart", "pl_open_left", "pl_held", "pl_packet", "pl_tables", "pl_stations", "pl_trades", "pl_lecterns", "pl_mounts"], |_value| true);

impl InventoryOpenEvent {
    /// bukkit: org/bukkit/event/inventory/InventoryOpenEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getInventory
    pub fn get_inventory(&self) -> MenuInventory {
        MenuInventory::read(&self.value)
    }
    /// bukkit: org/bukkit/event/inventory/InventoryEvent.getView
    pub fn get_view(&self) -> MenuView {
        MenuView { player: self.get_player(), top: self.get_inventory() }
    }
    /// The block, for a chest or a workstation.
    pub fn get_inventory_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/inventory/InventoryOpenEvent.titleOverride
    ///
    /// The title set here, if any.
    pub fn title_override(&self) -> Option<&crate::Component> {
        self.title.as_ref()
    }
    /// bukkit: org/bukkit/event/inventory/InventoryOpenEvent.titleOverride
    ///
    /// As Paper documents, a horse's or a nautilus's window keeps the entity's name.
    pub fn set_title_override(&mut self, title: &crate::Component) {
        self.title = Some(title.clone());
        query(json!({ "op": "ev_player_open_title", "title": title.to_legacy() }));
    }
    /// bukkit: org/bukkit/event/inventory/InventoryOpenEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/inventory/InventoryOpenEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Hoppers ----

/// bukkit: org/bukkit/event/inventory/HopperInventorySearchEvent
///
/// A hopper looking at the block above it or in front of it.
#[derive(Clone, Debug, Default)]
pub struct HopperInventorySearchEvent {
    value: Value,
    cancelled: bool,
    to: Option<Option<MenuInventory>>,
}

hooked!(HopperInventorySearchEvent, "pl_hopper_out", ["pl_hopper_in"], |value| value["kind"] == "search");

impl HopperInventorySearchEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    ///
    /// The hopper.
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/inventory/HopperInventorySearchEvent.getInventory
    ///
    /// What the hopper found, if it found anything.
    pub fn get_inventory(&self) -> Option<MenuInventory> {
        match &self.to {
            Some(to) => to.clone(),
            None => (!self.value["inventory"].is_null()).then(|| MenuInventory::read(&self.value["inventory"])),
        }
    }
    /// bukkit: org/bukkit/event/inventory/HopperInventorySearchEvent.setInventory
    pub fn set_inventory(&mut self, inventory: Option<&MenuInventory>) {
        self.to = Some(inventory.cloned());
        let at = inventory.and_then(MenuInventory::get_location).map(|at| json!([at.x.floor() as i32, at.y.floor() as i32, at.z.floor() as i32]));
        query(json!({ "op": "ev_player_hopper_to", "at": at }));
    }
    /// bukkit~: org/bukkit/event/inventory/HopperInventorySearchEvent.setInventory | a plugin's inventory on a hopper moves items, but `PrepareHopperTransferEvent` does not fire for it
    ///
    /// A plugin's own inventory, which lives in the plugin: the hopper's move is made here, with its
    /// `InventoryMoveItemEvent`, and the hopper then cools down as it does after any move.
    pub fn set_plugin_inventory(&mut self, inventory: &mut crate::Inventory) {
        self.to = Some(None);
        query(json!({ "op": "ev_player_hopper_to", "at": Value::Null }));
        let Some(mut hopper) = block_inventory(&self.value, &self.value["pos"]) else {
            return;
        };
        let pull = self.value["pull"] == true;
        let listed = json!({ "type": inventory.get_type(), "size": inventory.get_size(), "items": inventory.get_contents().iter().map(|stack| stack.as_ref().map_or(Value::Null, ItemStack::to_json)).collect::<Vec<_>>() });
        let items = hopper.get_contents();
        let from = if pull { inventory.get_contents().to_vec() } else { items.clone() };
        let Some((slot, stack)) = from.iter().enumerate().find_map(|(slot, stack)| stack.clone().map(|stack| (slot, stack))) else {
            return;
        };
        let mut one = stack.clone();
        one.set_amount(1);
        let place = |list: &[Option<ItemStack>]| list.iter().position(|held| held.as_ref().is_some_and(|held| held.is_similar(&one) && held.get_amount() < held.get_max_stack_size())).or_else(|| list.iter().position(Option::is_none));
        let Some(into) = place(&if pull { items.clone() } else { inventory.get_contents().to_vec() }) else {
            return;
        };
        let answer = query(json!({ "op": "ev_player_hopper_moved", "pos": self.value["pos"], "pull": pull, "stack": snbt_of(Some(&one)), "shown": one.to_json(), "inventory": listed }));
        if answer["refused"] == true {
            return;
        }
        let one = ItemStack::read(&answer["stack"]).unwrap_or(one);
        let mut left = stack;
        left.set_amount(left.get_amount() - 1);
        let left = (left.get_amount() > 0).then_some(left);
        let grown = |held: Option<ItemStack>| held.map_or_else(|| one.clone(), |mut held| { held.set_amount(held.get_amount() + one.get_amount()); held });
        if pull {
            inventory.set_item(slot, left);
            hopper.set_item(into, Some(&grown(items[into].clone())));
        } else {
            hopper.set_item(slot, left.as_ref());
            let held = inventory.get_contents()[into].clone();
            inventory.set_item(into, Some(grown(held)));
        }
    }
    /// bukkit: org/bukkit/event/inventory/HopperInventorySearchEvent.getContainerType
    ///
    /// `ContainerType`'s name.
    pub fn get_container_type(&self) -> &'static str {
        if self.value["pull"] == true { "SOURCE" } else { "DESTINATION" }
    }
    /// bukkit: org/bukkit/event/inventory/HopperInventorySearchEvent.getSearchBlock
    ///
    /// The block it looks at.
    pub fn get_search_block(&self) -> Option<Block> {
        block_at(&self.value, &self.value["at"])
    }
}

/// bukkit: io/papermc/paper/event/inventory/PrepareHopperTransferEvent
///
/// A hopper about to choose what to move. It doesn't see the slots a listener closes.
#[derive(Clone, Debug, Default)]
pub struct PrepareHopperTransferEvent {
    value: Value,
    cancelled: bool,
    closed: u64,
}

hooked!(PrepareHopperTransferEvent, "pl_hopper_out", ["pl_hopper_in", "pl_hopper_take"], |value| value["kind"] == "prepare");

impl PrepareHopperTransferEvent {
    /// bukkit: io/papermc/paper/event/inventory/PrepareHopperTransferEvent.getSource
    pub fn get_source(&self) -> MenuInventory {
        MenuInventory::read(&self.value["source"])
    }
    /// bukkit: io/papermc/paper/event/inventory/PrepareHopperTransferEvent.getDestination
    pub fn get_destination(&self) -> MenuInventory {
        MenuInventory::read(&self.value["destination"])
    }
    /// bukkit: io/papermc/paper/event/inventory/PrepareHopperTransferEvent.getInitiator
    ///
    /// The hopper that's doing it.
    pub fn get_initiator(&self) -> MenuInventory {
        if self.value["pull"] == true { self.get_destination() } else { self.get_source() }
    }
    /// bukkit: io/papermc/paper/event/inventory/PrepareHopperTransferEvent.isSlotAllowed
    pub fn is_slot_allowed(&self, slot: usize) -> bool {
        self.closed & 1u64.checked_shl(slot as u32).unwrap_or(0) == 0
    }
    /// bukkit: io/papermc/paper/event/inventory/PrepareHopperTransferEvent.setSlotAllowed
    pub fn set_slot_allowed(&mut self, slot: usize, allowed: bool) {
        let bit = 1u64.checked_shl(slot as u32).unwrap_or(0);
        if allowed {
            self.closed &= !bit;
        } else {
            self.closed |= bit;
        }
        query(json!({ "op": "ev_player_hopper_slots", "closed": self.closed }));
    }
}

// ---- Joining ----

/// bukkit: io/papermc/paper/event/player/PlayerServerFullCheckEvent
///
/// Every login, where the game looks at the count: on the connection's thread, with only what's in
/// the event to read, as on Paper.
#[derive(Clone, Debug, Default)]
pub struct PlayerServerFullCheckEvent {
    value: Value,
}

impl Event for PlayerServerFullCheckEvent {
    const HOOK: &'static str = "pl_full";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone() })
    }
}

impl PlayerServerFullCheckEvent {
    /// bukkit: io/papermc/paper/event/player/PlayerServerFullCheckEvent.getPlayerProfile
    pub fn get_player_profile(&self) -> crate::player::PlayerProfile {
        crate::player::PlayerProfile { name: self.value["name"].as_str().unwrap_or_default().to_owned(), id: self.value["uuid"].as_str().unwrap_or_default().to_owned(), properties: Vec::new() }
    }
    /// bukkit: io/papermc/paper/event/player/PlayerServerFullCheckEvent.isAllowed
    pub fn is_allowed(&self) -> bool {
        self.value["allowed"] == true
    }
    /// bukkit: io/papermc/paper/event/player/PlayerServerFullCheckEvent.kickMessage
    ///
    /// What they'd be told.
    pub fn kick_message(&self) -> crate::Component {
        if self.value["kick_message"].is_null() { crate::Component(json!({ "translate": "multiplayer.disconnect.server_full" })) } else { crate::Component(self.value["kick_message"].clone()) }
    }
    /// bukkit: io/papermc/paper/event/player/PlayerServerFullCheckEvent.deny
    pub fn deny(&mut self, message: &crate::Component) {
        self.allow(false);
        self.value["kick_message"] = message.0.clone();
        crate::event::carry("kick_message", message.0.clone());
    }
    /// bukkit: io/papermc/paper/event/player/PlayerServerFullCheckEvent.allow
    pub fn allow(&mut self, allow: bool) {
        self.value["allowed"] = json!(allow);
        crate::event::carry("allowed", json!(allow));
    }
}

/// bukkit: org/bukkit/event/player/PlayerLinksSendEvent
///
/// The links a client is about to get as it configures, on the connection's thread: only what's in
/// the event can be read. This connection's own copy.
#[derive(Clone, Debug)]
pub struct PlayerLinksSendEvent {
    value: Value,
    links: Vec<crate::fam_paper::ServerLinkEntry>,
    before: Vec<crate::fam_paper::ServerLinkEntry>,
}

impl Event for PlayerLinksSendEvent {
    const HOOK: &'static str = "pl_links";
    fn read(value: &Value) -> Option<Self> {
        let links: Vec<_> = value["links"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|row| crate::fam_paper::ServerLinkEntry {
                kind: row["known"].as_u64().and_then(|index| crate::fam_core::LINK_TYPES.get(index as usize)).map(|name| (*name).to_owned()),
                label: (!row["label"].is_null()).then(|| crate::Component(row["label"].clone())),
                url: row["url"].as_str().unwrap_or_default().to_owned(),
            })
            .collect();
        Some(Self { value: value.clone(), before: links.clone(), links })
    }
    fn finish(&self) {
        if self.links != self.before {
            let rows: Vec<Value> = self
                .links
                .iter()
                .map(|link| match link.kind.as_deref().and_then(|kind| crate::fam_core::LINK_TYPES.iter().position(|known| *known == kind)) {
                    Some(index) => json!({ "known": index, "url": link.url }),
                    None => json!({ "label": link.label.as_ref().map(|label| label.0.clone()), "url": link.url }),
                })
                .collect();
            crate::event::carry("links", json!(rows));
        }
    }
}

impl PlayerLinksSendEvent {
    /// bukkit: org/bukkit/event/player/PlayerLinksSendEvent.getLinks
    ///
    /// This connection's own copy of the list.
    pub fn get_links(&mut self) -> &mut Vec<crate::fam_paper::ServerLinkEntry> {
        &mut self.links
    }
    /// bukkit: org/bukkit/event/player/PlayerLinksSendEvent.getConnection
    pub fn get_connection(&self) -> crate::fam_paper::PlayerConfigurationConnection {
        crate::fam_paper::PlayerConfigurationConnection(self.value.clone())
    }
}

/// bukkit: org/bukkit/event/player/PlayerCommandSendEvent
///
/// At join and whenever a player's commands would be sent: the hub holds them to the next turn.
#[derive(Clone, Debug)]
pub struct PlayerCommandSendEvent {
    player: Option<Player>,
    commands: Vec<String>,
    before: Vec<String>,
}

impl Event for PlayerCommandSendEvent {
    const HOOK: &'static str = "pl_commands";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_sync", "pl_moves", "pl_move_turn"]
    }
    fn read(value: &Value) -> Option<Self> {
        let commands: Vec<String> = value["commands"].as_array()?.iter().filter_map(Value::as_str).map(str::to_owned).collect();
        Some(Self { player: player_of(value), before: commands.clone(), commands })
    }
    fn finish(&self) {
        if self.commands != self.before {
            crate::event::carry("commands", json!(self.commands));
            query(json!({ "op": "ev_player_commands_left", "left": self.commands }));
        }
    }
}

impl PlayerCommandSendEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<&Player> {
        self.player.as_ref()
    }
    /// bukkit: org/bukkit/event/player/PlayerCommandSendEvent.getCommands
    ///
    /// The commands they'll see: take away what they shouldn't.
    pub fn get_commands(&mut self) -> &mut Vec<String> {
        &mut self.commands
    }
}

/// bukkit: io/papermc/paper/event/player/AsyncChatCommandDecorateEvent
///
/// A player's `/say` or `/me`, before it goes out.
#[derive(Clone, Debug, Default)]
pub struct AsyncChatCommandDecorateEvent {
    value: Value,
    cancelled: bool,
    result: Option<crate::Component>,
}

hooked!(AsyncChatCommandDecorateEvent, "pl_say", |value| value["message"].is_string());

impl AsyncChatCommandDecorateEvent {
    pub fn player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    pub fn original_message(&self) -> crate::Component {
        crate::Component::text(self.value["message"].as_str().unwrap_or_default())
    }
    pub fn result(&self) -> crate::Component {
        self.result.clone().unwrap_or_else(|| self.original_message())
    }
    pub fn set_result(&mut self, result: &crate::Component) {
        self.result = Some(result.clone());
        crate::event::carry("message", json!(result.to_legacy()));
        query(json!({ "op": "ev_player_said", "text": result.to_legacy() }));
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    ///
    /// The message goes out as it was typed.
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
        if cancelled {
            query(json!({ "op": "ev_player_said", "text": Value::Null }));
        }
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Before the game has them: a connection's thread, as on Paper's netty thread ----

/// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent
///
/// A client's handshake, on its connection's thread with no game to ask: only what's in the event
/// can be read. It starts cancelled, as on Paper with no BungeeCord, and does nothing until a
/// listener lifts that.
#[derive(Clone, Debug, Default)]
pub struct PlayerHandshakeEvent {
    value: Value,
}

impl Event for PlayerHandshakeEvent {
    const HOOK: &'static str = "pl_handshake";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone() })
    }
}

impl PlayerHandshakeEvent {
    fn set(&mut self, key: &str, value: Value) {
        self.value[key] = value.clone();
        crate::event::carry(key, value);
    }
    fn text(&self, key: &str) -> Option<&str> {
        self.value[key].as_str()
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.getOriginalHandshake
    pub fn get_original_handshake(&self) -> &str {
        self.text("handshake").unwrap_or_default()
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.getOriginalSocketAddressHostname
    pub fn get_original_socket_address_hostname(&self) -> &str {
        self.text("remote").unwrap_or_default()
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.getServerHostname
    pub fn get_server_hostname(&self) -> Option<&str> {
        self.text("server_hostname")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.setServerHostname
    pub fn set_server_hostname(&mut self, hostname: &str) {
        self.set("server_hostname", json!(hostname));
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.getSocketAddressHostname
    pub fn get_socket_address_hostname(&self) -> Option<&str> {
        self.text("socket_hostname")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.setSocketAddressHostname
    ///
    /// An IP address: a name can't be looked up on this thread.
    pub fn set_socket_address_hostname(&mut self, hostname: &str) {
        self.set("socket_hostname", json!(hostname));
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.getUniqueId
    pub fn get_unique_id(&self) -> Option<&str> {
        self.text("unique_id")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.setUniqueId
    ///
    /// For an offline login: an online one is the session server's to say.
    pub fn set_unique_id(&mut self, uuid: &str) {
        self.set("unique_id", json!(uuid));
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.getPropertiesJson
    pub fn get_properties_json(&self) -> Option<&str> {
        self.text("properties")
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.setPropertiesJson
    pub fn set_properties_json(&mut self, json: &str) {
        self.set("properties", json!(json));
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.isFailed
    pub fn is_failed(&self) -> bool {
        self.value["failed"] == true
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.setFailed
    pub fn set_failed(&mut self, failed: bool) {
        self.set("failed", json!(failed));
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.failMessage
    pub fn fail_message(&self) -> crate::Component {
        if self.value["fail_message"].is_null() { crate::Component::text("You are not allowed to join this server.") } else { crate::Component(self.value["fail_message"].clone()) }
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.getFailMessage
    pub fn get_fail_message(&self) -> String {
        self.fail_message().to_legacy()
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.setFailMessage
    pub fn set_fail_message(&mut self, message: &crate::Component) {
        self.set("fail_message", message.0.clone());
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.set("cancelled", json!(cancelled));
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerHandshakeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.value["cancelled"] != false
    }
}

/// bukkit: io/papermc/paper/event/connection/configuration/PlayerCodeOfConductSendEvent
///
/// On the connection's thread, as configuration goes on: only what's in the event can be read.
#[derive(Clone, Debug, Default)]
pub struct PlayerCodeOfConductSendEvent {
    value: Value,
}

impl Event for PlayerCodeOfConductSendEvent {
    const HOOK: &'static str = "pl_conduct";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone() })
    }
}

impl PlayerCodeOfConductSendEvent {
    /// bukkit: io/papermc/paper/event/connection/configuration/PlayerCodeOfConductSendEvent.getConnection
    pub fn get_connection(&self) -> crate::fam_paper::PlayerConfigurationConnection {
        crate::fam_paper::PlayerConfigurationConnection(self.value.clone())
    }
    /// bukkit: io/papermc/paper/event/connection/configuration/PlayerCodeOfConductSendEvent.getCodeOfConduct
    pub fn get_code_of_conduct(&self) -> Option<&str> {
        self.value["code_of_conduct"].as_str()
    }
    /// bukkit: io/papermc/paper/event/connection/configuration/PlayerCodeOfConductSendEvent.setCodeOfConduct
    ///
    /// `None` sends nothing, and there's nothing to accept.
    pub fn set_code_of_conduct(&mut self, text: Option<&str>) {
        self.value["code_of_conduct"] = json!(text);
        crate::event::carry("code_of_conduct", json!(text));
    }
}

/// bukkit: com/destroystokyo/paper/event/player/IllegalPacketEvent
///
/// A packet that can't be read, on the connection's thread: only what's in the event can be read.
#[derive(Clone, Debug, Default)]
pub struct IllegalPacketEvent {
    value: Value,
}

impl Event for IllegalPacketEvent {
    const HOOK: &'static str = "pl_illegal";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone() })
    }
}

impl IllegalPacketEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        Player::read(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/player/IllegalPacketEvent.getType
    ///
    /// The packet's id.
    pub fn get_type(&self) -> Option<&str> {
        self.value["type"].as_str()
    }
    /// bukkit: com/destroystokyo/paper/event/player/IllegalPacketEvent.getExceptionMessage
    pub fn get_exception_message(&self) -> &str {
        self.value["exception"].as_str().unwrap_or_default()
    }
    /// bukkit: com/destroystokyo/paper/event/player/IllegalPacketEvent.isShouldKick
    pub fn is_should_kick(&self) -> bool {
        self.value["should_kick"] != false
    }
    /// bukkit: com/destroystokyo/paper/event/player/IllegalPacketEvent.setShouldKick
    pub fn set_should_kick(&mut self, kick: bool) {
        self.value["should_kick"] = json!(kick);
        crate::event::carry("should_kick", json!(kick));
    }
    /// bukkit: com/destroystokyo/paper/event/player/IllegalPacketEvent.getKickMessage
    pub fn get_kick_message(&self) -> Option<&str> {
        self.value["kick_message"].as_str()
    }
    /// bukkit: com/destroystokyo/paper/event/player/IllegalPacketEvent.setKickMessage
    pub fn set_kick_message(&mut self, message: Option<&str>) {
        self.value["kick_message"] = json!(message);
        crate::event::carry("kick_message", json!(message));
    }
}

// ---- Beds, tracking ----

/// bukkit: org/bukkit/event/player/PlayerBedLeaveEvent
///
/// A player getting up, at the turn after `Hub::wake`. A cancel leaves them in bed.
#[derive(Clone, Debug, Default)]
pub struct PlayerBedLeaveEvent {
    value: Value,
    cancelled: bool,
    keep: bool,
}

impl Event for PlayerBedLeaveEvent {
    const HOOK: &'static str = "pl_bed_leave";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_moves", "pl_move_turn"]
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), keep: true })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

impl PlayerBedLeaveEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/player/PlayerBedLeaveEvent.getBed
    pub fn get_bed(&self) -> Option<Block> {
        block_at(&self.value, &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/player/PlayerBedLeaveEvent.shouldSetSpawnLocation
    ///
    /// Whether the bed stays their spawn point.
    pub fn should_set_spawn_location(&self) -> bool {
        self.keep
    }
    /// bukkit: org/bukkit/event/player/PlayerBedLeaveEvent.setSpawnLocation
    ///
    /// `false` puts back the spawn point they had before they lay down.
    pub fn set_spawn_location(&mut self, keep: bool) {
        self.keep = keep;
        query(json!({ "op": "ev_player_bed_keep", "keep": keep }));
    }
    /// bukkit: org/bukkit/event/player/PlayerBedLeaveEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerBedLeaveEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerTrackEntityEvent
///
/// Before an entity is shown to a player, at the turn after the tracker would have. Refused, it stays
/// hidden and is asked about again a second later.
#[derive(Clone, Debug, Default)]
pub struct PlayerTrackEntityEvent {
    value: Value,
    cancelled: bool,
}

impl Event for PlayerTrackEntityEvent {
    const HOOK: &'static str = "pl_track";
    fn more_hooks() -> &'static [&'static str] {
        &["pl_packet", "pl_moves", "pl_move_turn"]
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer_of(self.cancelled)
    }
}

impl PlayerTrackEntityEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerTrackEntityEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.get_player().and_then(|player| Some(Entity::of(self.value["entity"].as_u64()?, &player.get_world())))
    }
    /// bukkit: io/papermc/paper/event/player/PlayerTrackEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/player/PlayerTrackEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/inventory/ItemCraftedEvent
///
/// A player taking what they crafted from the result slot. It can't be refused: that's `CraftItemEvent`.
#[derive(Clone, Debug, Default)]
pub struct ItemCraftedEvent {
    value: Value,
}

impl Event for ItemCraftedEvent {
    const HOOK: &'static str = "pl_packet";
    fn more_hooks() -> &'static [&'static str] {
        HELD_HOOKS
    }
    fn read(value: &Value) -> Option<Self> {
        (craft_click(value) && !crate::event::prior_cancelled()).then(|| Self { value: value.clone() })
    }
}

impl ItemCraftedEvent {
    /// bukkit: io/papermc/paper/event/inventory/ItemCraftedEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/inventory/ItemCraftedEvent.getCraftedItem
    pub fn get_crafted_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["current"])
    }
}

// ---- Back to configuration ----

/// bukkit: io/papermc/paper/event/connection/configuration/PlayerConnectionReconfigureEvent
///
/// A player sent back to configuration: on the connection's thread, with only what's in the event
/// to read. They stay until `complete_reconfiguration`.
#[derive(Clone, Debug, Default)]
pub struct PlayerConnectionReconfigureEvent {
    value: Value,
}

impl Event for PlayerConnectionReconfigureEvent {
    const HOOK: &'static str = "pl_reconfigure";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone() })
    }
}

impl PlayerConnectionReconfigureEvent {
    /// bukkit: io/papermc/paper/event/connection/configuration/PlayerConnectionReconfigureEvent.getConnection
    pub fn get_connection(&self) -> crate::fam_paper::PlayerConfigurationConnection {
        crate::fam_paper::PlayerConfigurationConnection(self.value.clone())
    }
}

impl crate::fam_paper::PlayerGameConnection {
    /// bukkit: io/papermc/paper/connection/PlayerGameConnection.reenterConfiguration
    ///
    /// They leave the game now, and join it again after `complete_reconfiguration`.
    pub fn reenter_configuration(&self) {
        query(json!({ "op": "ev_player_reconfigure", "uuid": self.0.uuid() }));
    }
}

impl crate::fam_paper::PlayerConfigurationConnection {
    /// bukkit: io/papermc/paper/connection/PlayerConfigurationConnection.completeReconfiguration
    pub fn complete_reconfiguration(&self) {
        query(json!({ "op": "paper_connection_complete", "uuid": self.0["uuid"] }));
    }
}

// ---- Logging in, on the connection's thread ----

/// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent, org/bukkit/event/player/PlayerPreLoginEvent
///
/// A client that's said who it is, before the game has anything to do with it: on the connection's
/// thread, with only what's in the event to read.
#[derive(Clone, Debug, Default)]
pub struct AsyncPlayerPreLoginEvent {
    value: Value,
}

/// The older name of the same moment.
pub type PlayerPreLoginEvent = AsyncPlayerPreLoginEvent;

impl Event for AsyncPlayerPreLoginEvent {
    const HOOK: &'static str = "pl_pre_login";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone() })
    }
}

impl AsyncPlayerPreLoginEvent {
    fn set(&mut self, key: &str, value: Value) {
        self.value[key] = value.clone();
        crate::event::carry(key, value);
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.getName
    pub fn get_name(&self) -> &str {
        self.value["name"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.getUniqueId
    pub fn get_unique_id(&self) -> String {
        crate::dashed(self.value["uuid"].as_str().unwrap_or_default())
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.getAddress
    pub fn get_address(&self) -> Option<std::net::IpAddr> {
        self.get_raw_socket_address().map(|socket| socket.ip())
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.getRawAddress
    pub fn get_raw_address(&self) -> Option<std::net::IpAddr> {
        self.get_address()
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.getRawSocketAddress
    pub fn get_raw_socket_address(&self) -> Option<std::net::SocketAddr> {
        self.value["address"].as_str()?.parse().ok()
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.getHostname
    ///
    /// The host and port the client says it connected to.
    pub fn get_hostname(&self) -> &str {
        self.value["host"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.isTransferred
    pub fn is_transferred(&self) -> bool {
        self.value["transferred"] == true
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.getConnection
    pub fn get_connection(&self) -> crate::fam_paper::PlayerLoginConnection {
        crate::fam_paper::PlayerLoginConnection(self.value.clone())
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.getPlayerProfile
    pub fn get_player_profile(&self) -> crate::player::PlayerProfile {
        let profile = if self.value["profile"].is_null() { &self.value } else { &self.value["profile"] };
        let id = if self.value["profile"].is_null() { &self.value["uuid"] } else { &profile["id"] };
        let properties = profile["properties"].as_array().into_iter().flatten().filter_map(|row| Some((row["name"].as_str()?.to_owned(), row["value"].as_str()?.to_owned(), row["signature"].as_str().map(str::to_owned)))).collect();
        crate::player::PlayerProfile { name: profile["name"].as_str().unwrap_or_default().to_owned(), id: id.as_str().unwrap_or_default().to_owned(), properties }
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.setPlayerProfile
    pub fn set_player_profile(&mut self, profile: &crate::player::PlayerProfile) {
        let properties: Vec<Value> = profile.properties.iter().map(|(name, value, signature)| json!({ "name": name, "value": value, "signature": signature })).collect();
        self.set("profile", json!({ "name": profile.name, "id": profile.id, "properties": properties }));
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.getLoginResult
    ///
    /// `Result`'s name.
    pub fn get_login_result(&self) -> &'static str {
        if self.value["refused"] == true { "KICK_OTHER" } else { "ALLOWED" }
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.getResult
    pub fn get_result(&self) -> &'static str {
        self.get_login_result()
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.setLoginResult
    pub fn set_login_result(&mut self, result: &str) {
        self.set("refused", json!(result != "ALLOWED"));
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.setResult
    pub fn set_result(&mut self, result: &str) {
        self.set_login_result(result);
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.disallow
    pub fn disallow(&mut self, result: &str, message: &crate::Component) {
        self.set_login_result(result);
        self.set("kick_message", message.0.clone());
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.allow
    pub fn allow(&mut self) {
        self.set("refused", json!(false));
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.kickMessage
    pub fn kick_message(&self) -> crate::Component {
        crate::Component(self.value["kick_message"].clone())
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.getKickMessage
    pub fn get_kick_message(&self) -> String {
        self.kick_message().to_legacy()
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent.setKickMessage
    pub fn set_kick_message(&mut self, message: &crate::Component) {
        self.set("kick_message", message.0.clone());
    }
}
