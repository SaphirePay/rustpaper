//! Menu: a chest GUI (`/menu`) whose items do things when clicked, and can't be taken out.
use horizon_bukkit::prelude::*;
use std::sync::Mutex;

const HEAL_SLOT: i32 = 11;
const SPAWN_SLOT: i32 = 15;

/// The GUI, made once. `Inventory` is a handle: every player who opens it sees the same menu.
static MENU: Mutex<Option<Inventory>> = Mutex::new(None);

fn menu() -> Inventory {
    MENU.lock().unwrap().get_or_insert_with(|| {
        let mut menu = Inventory::create(27, "Menu");
        menu.set_item(HEAL_SLOT as usize, Some(named("DIAMOND", "Heal me")));
        menu.set_item(SPAWN_SLOT as usize, Some(named("COMPASS", "Spawn")));
        menu
    })
    .clone()
}

/// An item with a display name. Materials are their Bukkit names: `Material::of("DIAMOND")`.
fn named(material: &str, name: &str) -> ItemStack {
    let mut stack = ItemStack::new(Material::of(material), 1);
    stack.set_display_name(name);
    stack
}

fn on_enable(plugin: Plugin) {
    plugin.set_executor("menu", |sender: &CommandSender, _label: &str, _args: &[&str]| {
        let Some(player) = sender.as_player() else {
            sender.send_message("§cOnly players can open the menu.");
            return true;
        };
        player.open_inventory(&menu());
        true
    });
    plugin.register_event(|event: &mut InventoryClickEvent| {
        // Java's `if (event.getInventory() != menu) return;`
        if !event.get_inventory().is(&menu()) {
            return;
        }
        // Nothing moves in or out of the menu, from either half of the window.
        event.set_cancelled(true);
        let Some(player) = event.get_who_clicked() else { return };
        match event.get_raw_slot() {
            HEAL_SLOT => {
                player.set_health(player.get_max_health());
                player.set_food_level(20);
                // Closing is safe inside the click: it happens once the click is answered.
                player.close_inventory();
            }
            SPAWN_SLOT => {
                player.teleport(&player.get_world().get_spawn_location());
            }
            _ => {}
        }
    });
    // Dragging items across the menu's slots is the other way to put something in.
    plugin.register_event(|event: &mut InventoryDragEvent| {
        if event.get_inventory().is(&menu()) {
            event.set_cancelled(true);
        }
    });
}

fn on_disable(_plugin: Plugin) {}

bukkit_plugin!(on_enable, on_disable);
