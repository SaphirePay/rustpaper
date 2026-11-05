//! The rest of `org.bukkit.inventory`: a player's slots, equipment of anything that wears it, the view
//! of an open GUI and the GUI's own calls.
use crate::entity::Entity;
use crate::event::InventoryClickEvent;
use crate::{Inventory, ItemStack, Material, OPEN_INVENTORIES, Player, PlayerInventory};
use horizon_mod::run_command;

impl PlayerInventory {
    fn replace(&self, slot: &str, stack: Option<&ItemStack>) {
        let name = self.owner().get_name().to_owned();
        match stack {
            Some(stack) => run_command(&format!("item replace entity {name} {slot} with {} {}", stack.to_command(), stack.get_amount())),
            None => run_command(&format!("item replace entity {name} {slot} with minecraft:air")),
        }
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.setHelmet
    pub fn set_helmet(&self, stack: Option<&ItemStack>) {
        self.replace("armor.head", stack);
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.setChestplate
    pub fn set_chestplate(&self, stack: Option<&ItemStack>) {
        self.replace("armor.chest", stack);
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.setLeggings
    pub fn set_leggings(&self, stack: Option<&ItemStack>) {
        self.replace("armor.legs", stack);
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.setBoots
    pub fn set_boots(&self, stack: Option<&ItemStack>) {
        self.replace("armor.feet", stack);
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.setArmorContents
    ///
    /// In Bukkit's order: boots, leggings, chestplate, helmet.
    pub fn set_armor_contents(&self, stacks: &[Option<ItemStack>; 4]) {
        self.set_boots(stacks[0].as_ref());
        self.set_leggings(stacks[1].as_ref());
        self.set_chestplate(stacks[2].as_ref());
        self.set_helmet(stacks[3].as_ref());
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getItemInHand
    pub fn get_item_in_hand(&self) -> Option<ItemStack> {
        self.get_item_in_main_hand()
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.setItemInHand
    pub fn set_item_in_hand(&self, stack: &ItemStack) {
        self.set_item_in_main_hand(stack);
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getExtraContents
    pub fn get_extra_contents(&self) -> Vec<Option<ItemStack>> {
        vec![self.get_item_in_off_hand()]
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.setHeldItemSlot
    pub fn set_held_item_slot(&self, slot: i32) {
        self.owner().send_held_slot(slot);
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getHolder
    pub fn get_holder(&self) -> &Player {
        self.owner()
    }
    /// bukkit: org/bukkit/inventory/Inventory.isEmpty
    pub fn is_empty(&self) -> bool {
        self.get_contents().iter().all(Option::is_none)
    }
    /// bukkit: org/bukkit/inventory/Inventory.contains
    pub fn contains(&self, material: &Material) -> bool {
        self.get_contents().iter().flatten().any(|stack| stack.get_type() == material)
    }
    /// bukkit: org/bukkit/inventory/Inventory.containsAtLeast
    pub fn contains_at_least(&self, like: &ItemStack, amount: i32) -> bool {
        self.get_contents().iter().flatten().filter(|stack| stack.is_similar(like)).map(ItemStack::get_amount).sum::<i32>() >= amount
    }
    /// bukkit: org/bukkit/inventory/Inventory.first
    pub fn first(&self, material: &Material) -> i32 {
        self.get_contents().iter().position(|stack| stack.as_ref().is_some_and(|stack| stack.get_type() == material)).map_or(-1, |slot| slot as i32)
    }
    /// bukkit: org/bukkit/inventory/Inventory.remove
    pub fn remove(&self, material: &Material) {
        run_command(&format!("clear {} {}", self.owner().get_name(), material.key()));
    }
    /// bukkit: org/bukkit/inventory/Inventory.removeItem, org/bukkit/inventory/Inventory.removeItemAnySlot
    pub fn remove_item(&self, stack: &ItemStack) {
        run_command(&format!("clear {} {} {}", self.owner().get_name(), stack.get_type().key(), stack.get_amount()));
    }
}

/// `org.bukkit.inventory.EntityEquipment`: what a mob or armor stand wears, in its saved data.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityEquipment {
    entity: Entity,
}

impl Entity {
    /// bukkit: org/bukkit/entity/LivingEntity.getEquipment, org/bukkit/entity/Mob.getEquipment, org/bukkit/entity/ArmorStand.getEquipment
    pub fn get_equipment(&self) -> EntityEquipment {
        EntityEquipment { entity: self.clone() }
    }
}

const SLOTS: [(&str, &str); 6] = [("HAND", "mainhand"), ("OFF_HAND", "offhand"), ("HEAD", "head"), ("CHEST", "chest"), ("LEGS", "legs"), ("FEET", "feet")];

fn saved_slot(slot: &str) -> Option<&'static str> {
    SLOTS.iter().find(|(java, _)| *java == slot).map(|(_, saved)| *saved)
}

impl EntityEquipment {
    /// bukkit: org/bukkit/inventory/EntityEquipment.getHolder
    pub fn get_holder(&self) -> &Entity {
        &self.entity
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getItem
    pub fn get_item(&self, slot: &str) -> Option<ItemStack> {
        ItemStack::read(&self.entity.data_at(&format!("equipment.{}", saved_slot(slot)?)))
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setItem
    pub fn set_item(&self, slot: &str, stack: &ItemStack) -> bool {
        saved_slot(slot).is_some_and(|saved| self.entity.merge(&format!("{{equipment:{{{saved}:{}}}}}", stack.to_snbt())))
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getItemInMainHand, org/bukkit/inventory/EntityEquipment.getItemInHand
    pub fn get_item_in_main_hand(&self) -> Option<ItemStack> {
        self.get_item("HAND")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setItemInMainHand, org/bukkit/inventory/EntityEquipment.setItemInHand
    pub fn set_item_in_main_hand(&self, stack: &ItemStack) -> bool {
        self.set_item("HAND", stack)
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getItemInOffHand
    pub fn get_item_in_off_hand(&self) -> Option<ItemStack> {
        self.get_item("OFF_HAND")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setItemInOffHand
    pub fn set_item_in_off_hand(&self, stack: &ItemStack) -> bool {
        self.set_item("OFF_HAND", stack)
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getHelmet
    pub fn get_helmet(&self) -> Option<ItemStack> {
        self.get_item("HEAD")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setHelmet
    pub fn set_helmet(&self, stack: &ItemStack) -> bool {
        self.set_item("HEAD", stack)
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getChestplate
    pub fn get_chestplate(&self) -> Option<ItemStack> {
        self.get_item("CHEST")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setChestplate
    pub fn set_chestplate(&self, stack: &ItemStack) -> bool {
        self.set_item("CHEST", stack)
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getLeggings
    pub fn get_leggings(&self) -> Option<ItemStack> {
        self.get_item("LEGS")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setLeggings
    pub fn set_leggings(&self, stack: &ItemStack) -> bool {
        self.set_item("LEGS", stack)
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getBoots
    pub fn get_boots(&self) -> Option<ItemStack> {
        self.get_item("FEET")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setBoots
    pub fn set_boots(&self, stack: &ItemStack) -> bool {
        self.set_item("FEET", stack)
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getArmorContents
    pub fn get_armor_contents(&self) -> [Option<ItemStack>; 4] {
        [self.get_boots(), self.get_leggings(), self.get_chestplate(), self.get_helmet()]
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.clear
    pub fn clear(&self) -> bool {
        self.entity.merge("{equipment:{}}")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getDropChance
    pub fn get_drop_chance(&self, slot: &str) -> f32 {
        saved_slot(slot).map_or(0.085, |saved| self.entity.data_at(&format!("drop_chances.{saved}")).as_f64().unwrap_or(0.085) as f32)
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setDropChance
    pub fn set_drop_chance(&self, slot: &str, chance: f32) -> bool {
        saved_slot(slot).is_some_and(|saved| self.entity.merge(&format!("{{drop_chances:{{{saved}:{chance:?}f}}}}")))
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getHelmetDropChance
    pub fn get_helmet_drop_chance(&self) -> f32 {
        self.get_drop_chance("HEAD")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setHelmetDropChance
    pub fn set_helmet_drop_chance(&self, chance: f32) -> bool {
        self.set_drop_chance("HEAD", chance)
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getChestplateDropChance
    pub fn get_chestplate_drop_chance(&self) -> f32 {
        self.get_drop_chance("CHEST")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setChestplateDropChance
    pub fn set_chestplate_drop_chance(&self, chance: f32) -> bool {
        self.set_drop_chance("CHEST", chance)
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getLeggingsDropChance
    pub fn get_leggings_drop_chance(&self) -> f32 {
        self.get_drop_chance("LEGS")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setLeggingsDropChance
    pub fn set_leggings_drop_chance(&self, chance: f32) -> bool {
        self.set_drop_chance("LEGS", chance)
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getBootsDropChance
    pub fn get_boots_drop_chance(&self) -> f32 {
        self.get_drop_chance("FEET")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setBootsDropChance
    pub fn set_boots_drop_chance(&self, chance: f32) -> bool {
        self.set_drop_chance("FEET", chance)
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getItemInMainHandDropChance, org/bukkit/inventory/EntityEquipment.getItemInHandDropChance
    pub fn get_item_in_main_hand_drop_chance(&self) -> f32 {
        self.get_drop_chance("HAND")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setItemInMainHandDropChance, org/bukkit/inventory/EntityEquipment.setItemInHandDropChance
    pub fn set_item_in_main_hand_drop_chance(&self, chance: f32) -> bool {
        self.set_drop_chance("HAND", chance)
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.getItemInOffHandDropChance
    pub fn get_item_in_off_hand_drop_chance(&self) -> f32 {
        self.get_drop_chance("OFF_HAND")
    }
    /// bukkit: org/bukkit/inventory/EntityEquipment.setItemInOffHandDropChance
    pub fn set_item_in_off_hand_drop_chance(&self, chance: f32) -> bool {
        self.set_drop_chance("OFF_HAND", chance)
    }
}

impl Inventory {
    /// bukkit: org/bukkit/inventory/Inventory.getMaxStackSize
    pub fn get_max_stack_size(&self) -> i32 {
        crate::fam_items::inventory_limit(self.id)
    }
    /// bukkit: org/bukkit/inventory/Inventory.isEmpty
    pub fn is_empty(&self) -> bool {
        self.get_contents().iter().all(Option::is_none)
    }
    /// bukkit: org/bukkit/inventory/Inventory.addItem
    ///
    /// Into the stacks that have room, then the empty slots; what didn't fit comes back.
    pub fn add_item(&mut self, stack: &ItemStack) -> Option<ItemStack> {
        let mut left = stack.get_amount();
        let limit = stack.get_max_stack_size();
        for slot in 0..self.get_size() {
            let Some(held) = self.get_item(slot).filter(|held| held.is_similar(stack)).cloned() else {
                continue;
            };
            let moved = (limit - held.get_amount()).clamp(0, left);
            left -= moved;
            self.set_item(slot, Some(held.as_quantity(held.get_amount() + moved)));
        }
        while left > 0 {
            let Ok(slot) = usize::try_from(self.first_empty()) else {
                return Some(stack.as_quantity(left));
            };
            let moved = left.min(self.get_max_stack_size());
            left -= moved;
            self.set_item(slot, Some(stack.as_quantity(moved)));
        }
        None
    }
    /// bukkit: org/bukkit/inventory/Inventory.removeItem, org/bukkit/inventory/Inventory.removeItemAnySlot
    ///
    /// What couldn't be found comes back.
    pub fn remove_item(&mut self, stack: &ItemStack) -> Option<ItemStack> {
        let mut left = stack.get_amount();
        for slot in 0..self.get_size() {
            let Some(held) = self.get_item(slot).filter(|held| held.is_similar(stack)).cloned() else {
                continue;
            };
            let taken = left.min(held.get_amount());
            left -= taken;
            self.set_item(slot, (held.get_amount() > taken).then(|| held.as_quantity(held.get_amount() - taken)));
            if left == 0 {
                return None;
            }
        }
        Some(stack.as_quantity(left))
    }
    /// bukkit: org/bukkit/inventory/Inventory.setContents, org/bukkit/inventory/Inventory.setStorageContents
    pub fn set_contents(&mut self, stacks: Vec<Option<ItemStack>>) {
        for slot in 0..self.get_size() {
            self.set_item(slot, stacks.get(slot).cloned().flatten());
        }
    }
    /// bukkit: org/bukkit/inventory/Inventory.contains
    pub fn contains(&self, material: &Material) -> bool {
        self.get_contents().iter().flatten().any(|stack| stack.get_type() == material)
    }
    /// bukkit: org/bukkit/inventory/Inventory.containsAtLeast
    pub fn contains_at_least(&self, like: &ItemStack, amount: i32) -> bool {
        self.get_contents().iter().flatten().filter(|stack| stack.is_similar(like)).map(ItemStack::get_amount).sum::<i32>() >= amount
    }
    /// bukkit: org/bukkit/inventory/Inventory.first
    pub fn first(&self, material: &Material) -> i32 {
        self.get_contents().iter().position(|stack| stack.as_ref().is_some_and(|stack| stack.get_type() == material)).map_or(-1, |slot| slot as i32)
    }
    /// bukkit: org/bukkit/inventory/Inventory.all
    pub fn all(&self, material: &Material) -> Vec<(usize, ItemStack)> {
        self.get_contents().iter().enumerate().filter_map(|(slot, stack)| stack.as_ref().filter(|stack| stack.get_type() == material).map(|stack| (slot, stack.clone()))).collect()
    }
    /// bukkit: org/bukkit/inventory/Inventory.remove
    pub fn remove(&mut self, material: &Material) {
        for (slot, _) in self.all(material) {
            self.set_item(slot, None);
        }
    }
    /// bukkit: org/bukkit/inventory/Inventory.iterator
    pub fn iter(&self) -> impl Iterator<Item = &Option<ItemStack>> {
        self.get_contents().iter()
    }
    /// bukkit: org/bukkit/inventory/Inventory.getViewers
    pub fn get_viewers(&self) -> Vec<Player> {
        let open = OPEN_INVENTORIES.lock().unwrap_or_else(|e| e.into_inner());
        crate::get_online_players().into_iter().filter(|player| open.get(player.uuid()).is_some_and(|inventory| inventory.same_as(self))).collect()
    }
    /// bukkit: org/bukkit/inventory/Inventory.close
    pub fn close(&self) -> usize {
        let viewers = self.get_viewers();
        viewers.iter().for_each(Player::close_inventory);
        viewers.len()
    }
    /// bukkit: org/bukkit/inventory/Inventory.getHolder
    ///
    /// A GUI made by a plugin has none unless the plugin keeps one itself, which Rust does by its own
    /// map from `same_as`: there's no class to compare with `instanceof`.
    pub fn get_holder(&self) -> Option<&Player> {
        None
    }
    /// bukkit: org/bukkit/inventory/Inventory.getLocation
    pub fn get_location(&self) -> Option<crate::Location> {
        None
    }
}

/// `org.bukkit.inventory.InventoryView`: a player and the GUI they have open.
#[derive(Clone, Debug)]
pub struct InventoryView {
    player: Player,
    top: Option<Inventory>,
}

impl Player {
    /// bukkit: org/bukkit/entity/Player.getOpenInventory, org/bukkit/entity/HumanEntity.getOpenInventory
    pub fn get_open_inventory(&self) -> InventoryView {
        let top = OPEN_INVENTORIES.lock().unwrap_or_else(|e| e.into_inner()).get(self.uuid()).cloned();
        InventoryView { player: self.clone(), top }
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getItemInHand
    pub fn get_item_in_hand(&self) -> Option<ItemStack> {
        self.get_inventory().get_item_in_main_hand()
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setItemInHand
    pub fn set_item_in_hand(&self, stack: &ItemStack) {
        self.get_inventory().set_item_in_main_hand(stack);
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getItemOnCursor
    pub fn get_item_on_cursor(&self) -> Option<ItemStack> {
        ItemStack::read(&self.equipment_state()["cursor"])
    }
}

impl InventoryView {
    /// bukkit: org/bukkit/inventory/InventoryView.getTopInventory
    pub fn get_top_inventory(&self) -> Option<&Inventory> {
        self.top.as_ref()
    }
    /// bukkit: org/bukkit/inventory/InventoryView.getBottomInventory
    pub fn get_bottom_inventory(&self) -> PlayerInventory {
        self.player.get_inventory()
    }
    /// bukkit: org/bukkit/inventory/InventoryView.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/inventory/InventoryView.getType
    pub fn get_type(&self) -> &'static str {
        if self.top.is_some() { "CHEST" } else { "CRAFTING" }
    }
    /// bukkit: org/bukkit/inventory/InventoryView.getTitle
    pub fn get_title(&self) -> &str {
        self.top.as_ref().map_or("Crafting", Inventory::get_title)
    }
    /// bukkit: org/bukkit/inventory/InventoryView.getOriginalTitle
    pub fn get_original_title(&self) -> &str {
        self.get_title()
    }
    /// bukkit: org/bukkit/inventory/InventoryView.title
    pub fn title(&self) -> crate::Component {
        crate::adventure::legacy(self.get_title())
    }
    /// bukkit: org/bukkit/inventory/InventoryView.countSlots
    pub fn count_slots(&self) -> usize {
        self.top.as_ref().map_or(0, Inventory::get_size) + 36
    }
    /// bukkit: org/bukkit/inventory/InventoryView.getItem
    pub fn get_item(&self, raw_slot: usize) -> Option<ItemStack> {
        let size = self.top.as_ref().map_or(0, Inventory::get_size);
        if raw_slot < size {
            return self.top.as_ref()?.get_item(raw_slot).cloned();
        }
        // After the GUI come the main rows, then the hotbar, as the game counts a menu's slots.
        let below = raw_slot - size;
        let slot = if below < 27 { below + 9 } else { below - 27 };
        self.player.get_inventory().get_item(i32::try_from(slot).ok()?)
    }
    /// bukkit: org/bukkit/inventory/InventoryView.convertSlot
    pub fn convert_slot(&self, raw_slot: i32) -> i32 {
        let size = self.top.as_ref().map_or(0, Inventory::get_size) as i32;
        if raw_slot < size {
            return raw_slot;
        }
        let below = raw_slot - size;
        if below < 27 { below + 9 } else { below - 27 }
    }
    /// bukkit: org/bukkit/inventory/InventoryView.getCursor
    pub fn get_cursor(&self) -> Option<ItemStack> {
        self.player.get_item_on_cursor()
    }
    /// bukkit: org/bukkit/inventory/InventoryView.close
    pub fn close(&self) {
        self.player.close_inventory();
    }
}

