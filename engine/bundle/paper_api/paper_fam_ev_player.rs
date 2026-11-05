// Family ev_player: answers to the queries its SDK file makes, and the menu packets it holds back.
//
// A guest's packet runs with the hub out of `game.lan`, so a listener there can ask nothing of any
// player and what it says to one is lost. A menu packet is therefore held: `hold_packet` (at the
// head of `lan_guest_packet`) keeps it and leaves a marker in `host.commands`, which makes `tick_lan`
// give the hub back and call `run_guest_commands` straight after this packet, before the next one.
// `run_held`, at the head of that, fires the event and then sends the packet through
// `lan_guest_packet` again, or takes back what the client predicted.
use crate::game::{
    Dim, Game, paper_bridge,
    lan::LanHost,
    lan_actions::{GuestCommand, GuestPlay},
};
use horizon_client::{
    items::ItemStack,
    lan::menu::Slots,
    menu::{Kind, Menu},
};
use horizon_client::first_person::Hand;
use horizon_core::random::RandomSource;
use horizon_mod_runtime as runtime;
use horizon_client::{containers::BlockEntity, furnace::FurnaceCooking};
use horizon_protocol::packets::play::{self, Serverbound as S};
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::ops::ControlFlow;

/// The window, the block and the menu the game keeps out of this module's sight: a crafting
/// table's or a workstation's. Made where those fields are visible, and kept as a plain function.
pub(in crate::game) type FindMenu = for<'a> fn(&'a mut LanHost, u32) -> Option<(i32, [i32; 3], &'a mut Menu)>;
/// A merchant's window and its payment slots and result.
pub(in crate::game) type FindTrade = for<'a> fn(&'a mut LanHost, u32) -> Option<(i32, &'a mut [Option<ItemStack>; 3])>;
/// A guest's open lectern: its window and block.
pub(in crate::game) type FindLectern = for<'a> fn(&'a LanHost, u32) -> Option<(i32, (i32, i32, i32))>;
/// A guest's open mount inventory: its window, mob and columns, and how the game builds and keeps its menu.
pub(in crate::game) type FindMount = for<'a> fn(&'a LanHost, u32) -> Option<(i32, u32, usize)>;
pub(in crate::game) type MountMenu = for<'a, 'b> fn(&'a Game<'b>, usize, usize) -> Menu;
pub(in crate::game) type MountStore = for<'a, 'b, 'c> fn(&'a mut Game<'b>, usize, usize, &'c Menu);
/// One of `GuestPlay`'s private tick counters.
pub(in crate::game) type PlayTicks = for<'a> fn(&'a mut GuestPlay) -> &'a mut u32;

thread_local! {
    /// How to get the data the hub was started on: `game::lan` keeps it private.
    static SYNC_DATA: Cell<Option<fn() -> Option<std::sync::Arc<horizon_client::lan::data::SyncData>>>> = const { Cell::new(None) };
    /// Server mods' commands, as the hub's tree was made from them.
    static MOD_COMMANDS: RefCell<Vec<crate::game::native_server::NativeCommand>> = const { RefCell::new(Vec::new()) };
    /// What `PlayerCommandSendEvent` left, when a listener took any away.
    static COMMANDS_LEFT: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
    /// What `PlayerPortalEvent` said of a trip, by where its exit is looked for: how far, whether
    /// one may be built, and how far from there.
    static PORTALS: RefCell<std::collections::HashMap<[i32; 3], PortalRule>> = RefCell::new(std::collections::HashMap::new());
    /// The radius the next look for a portal uses.
    static PORTAL_RADIUS: Cell<Option<i32>> = const { Cell::new(None) };
    /// Whether the bed a player used outside the overworld still blows up.
    static BED_BLAST: Cell<bool> = const { Cell::new(true) };
    static BED_PASS: Cell<bool> = const { Cell::new(false) };
    static SAY_PASS: Cell<bool> = const { Cell::new(false) };
    /// What `AsyncChatCommandDecorateEvent` left, while it fires.
    static SAID: RefCell<Option<String>> = const { RefCell::new(None) };
    /// The id a thrown potion is to have.
    static THROWN_ID: Cell<u32> = const { Cell::new(0) };
    /// What `PlayerCommandPreprocessEvent` turned a command into: the command, the new one, the tick.
    static COMMAND_TO: RefCell<Option<(String, String, u64)>> = const { RefCell::new(None) };
    /// `PlayerBedEnterEvent` said `ALLOW` of a refusal.
    static BED_FORCE: Cell<bool> = const { Cell::new(false) };
    /// Who's to go back to configuration, once the hub is here to tell them.
    static RECONFIGURES: RefCell<Vec<u128>> = const { RefCell::new(Vec::new()) };
    /// The kick to come: whose `/kick` ran.
    static KICK_BY: Cell<Option<&'static str>> = const { Cell::new(None) };
    /// `Player.kickPlayer` is about to run `/kick`.
    static KICK_PLUGIN: Cell<bool> = const { Cell::new(false) };
    /// What `PlayerKickEvent.setReason` left, while it fires.
    static KICK_REASON: RefCell<Option<String>> = const { RefCell::new(None) };
    /// What `PlayerLoginEvent` refused them with, while it fires.
    static LOGIN_LINE: RefCell<Option<String>> = const { RefCell::new(None) };
    /// Whether a refused item flies at the player anyway.
    static FLY: Cell<bool> = const { Cell::new(false) };
    static ARROW_PASS: Cell<bool> = const { Cell::new(false) };
    static OPENING: Cell<bool> = const { Cell::new(false) };
    /// `InventoryOpenEvent.titleOverride`, while it fires.
    static OPEN_TITLE: RefCell<Option<String>> = const { RefCell::new(None) };
    /// Menus waiting for `InventoryOpenEvent` until the hub is back.
    static LATER: RefCell<Vec<Later>> = const { RefCell::new(Vec::new()) };
    /// Stats wait for their event during a guest's turn.
    static STAT_ASKING: Cell<bool> = const { Cell::new(false) };
    /// Where a portal's refused exit sends its traveller back to.
    static PORTAL_RETURN: RefCell<Option<(Dim, [f64; 3])>> = const { RefCell::new(None) };
    /// The arrows a listener refused this tick, by connection and arrow.
    static ARROWS_LEFT: RefCell<Vec<(u32, u32)>> = const { RefCell::new(Vec::new()) };
    static HOPPER_PASS: Cell<bool> = const { Cell::new(false) };
    /// Where a listener pointed a hopper, while it fires: `None` inside for no inventory.
    static HOPPER_TO: Cell<Option<Option<(i32, i32, i32)>>> = const { Cell::new(None) };
    /// The source's slots a listener closed to the hopper at work.
    static HOPPER_SLOTS: Cell<u64> = const { Cell::new(0) };
    static RECIPE_PASS: Cell<bool> = const { Cell::new(false) };
    /// `PlayerPickItemEvent`'s source and target, when a listener changed them.
    static PICK_TO: Cell<Option<(i32, i32)>> = const { Cell::new(None) };
    static RESPAWN_PASS: Cell<bool> = const { Cell::new(false) };
    /// What each player's last respawn came to.
    static RESPAWNS: RefCell<std::collections::HashMap<u128, Value>> = RefCell::new(std::collections::HashMap::new());
}

thread_local! {
    static HELD: RefCell<Vec<Held>> = const { RefCell::new(Vec::new()) };
    /// True while a held packet goes through: nothing is held then.
    static PASSING: Cell<bool> = const { Cell::new(false) };
    static TABLES: Cell<Option<FindMenu>> = const { Cell::new(None) };
    static STATIONS: Cell<Option<FindMenu>> = const { Cell::new(None) };
    static TRADES: Cell<Option<FindTrade>> = const { Cell::new(None) };
    /// The slots of a drag still going, by connection.
    static DRAGS: RefCell<Vec<(u32, Vec<i16>)>> = const { RefCell::new(Vec::new()) };
    /// What each guest's prepare event last saw.
    static PREPARED: RefCell<Vec<(u32, Value)>> = const { RefCell::new(Vec::new()) };
    /// Slots to set once the held packet has gone through: -1 is the cursor.
    static AFTER: RefCell<Vec<(i32, Option<ItemStack>)>> = const { RefCell::new(Vec::new()) };
    /// A listener changed a slot: the client must be told.
    static WROTE: Cell<bool> = const { Cell::new(false) };
    /// `EnchantItemEvent.setExpLevelCost`.
    static TOO_DEAR: Cell<bool> = const { Cell::new(false) };
    /// `PlayerItemDamageEvent.setDamage`: the wear in place of the game's.
    static LOSS: Cell<u32> = const { Cell::new(0) };
    /// `InventoryMoveItemEvent.setItem`, and that the move is being made again with it.
    static MOVED: RefCell<Option<ItemStack>> = const { RefCell::new(None) };
    static MOVED_AGAIN: Cell<bool> = const { Cell::new(false) };
    /// `PlayerEditBookEvent`'s pages and title, in place of the client's.
    static BOOK: RefCell<Option<(Vec<String>, Option<String>)>> = const { RefCell::new(None) };
    /// `PlayerExpChangeEvent.setAmount`.
    static EXP: Cell<Option<u32>> = const { Cell::new(None) };
}

thread_local! {
    /// What a furnace's or brewing stand's listeners changed: burn or fuel time, cook time, keep the fuel.
    static BURN: Cell<(Option<i32>, Option<i32>, bool)> = const { Cell::new((None, None, false)) };
    /// A block entity's tick is going through with its listeners answered.
    static TICKING: Cell<bool> = const { Cell::new(false) };
    static LECTERNS: Cell<Option<FindLectern>> = const { Cell::new(None) };
    static MOUNTS: Cell<Option<(FindMount, MountMenu, MountStore)>> = const { Cell::new(None) };
    /// The tick the `PlayerFishEvent` of a catch fired in, and the experience it settled on.
    static FISH_EXP: Cell<Option<(u32, u64)>> = const { Cell::new(None) };
    /// `give_orb_points` going through with its event answered.
    static EXP_PASS: Cell<bool> = const { Cell::new(false) };
    /// `PlayerItemMendEvent`'s repair and the experience it uses, in place of the game's.
    static MEND: Cell<Option<(u32, u32)>> = const { Cell::new(None) };
    /// The orb being taken, for what it sets off.
    static LAST_ORB: Cell<Option<u32>> = const { Cell::new(None) };
    static ORB_PASS: Cell<bool> = const { Cell::new(false) };
    /// `PlayerExpCooldownChangeEvent.setNewCooldown`.
    static ORB_COOLDOWN: Cell<Option<u32>> = const { Cell::new(None) };
    /// `PlayerItemGroupCooldownEvent.setCooldown`, and the cooldown going through with it.
    static COOLDOWN: Cell<Option<i32>> = const { Cell::new(None) };
    static COOLING: Cell<bool> = const { Cell::new(false) };
    /// A throw under way: the item, and whether the player keeps it.
    static THROWN: RefCell<Option<Value>> = const { RefCell::new(None) };
    static THROW_KEEP: Cell<bool> = const { Cell::new(false) };
    /// `PlayerShieldDisableEvent`: who hit the shield, and the cooldown in place of the game's.
    static SHIELD_ATTACKER: Cell<Option<u32>> = const { Cell::new(None) };
    static SHIELD_TICKS: Cell<Option<u32>> = const { Cell::new(None) };
}

thread_local! {
    static SHIELD_PASS: Cell<bool> = const { Cell::new(false) };
    /// `PlayerItemConsumeEvent.setItem`, and the meal going through with it.
    static EATEN: RefCell<Option<ItemStack>> = const { RefCell::new(None) };
    static EATING: Cell<bool> = const { Cell::new(false) };
    /// The tick a player used a sign by hand: a sign editor opened then isn't one just placed.
    static SIGN_USED: Cell<u64> = const { Cell::new(u64::MAX) };
    /// The side `execute_sign_click_commands` is on.
    static SIGN_FRONT: Cell<bool> = const { Cell::new(true) };
    /// What's in the packet, in place of what the client sent: a new button or recipe.
    static NEW_BUTTON: Cell<Option<i32>> = const { Cell::new(None) };
    static NEW_RECIPE: Cell<Option<(Option<i32>, Option<bool>)>> = const { Cell::new(None) };
    static NEW_BEACON: Cell<Option<(Option<i32>, Option<i32>)>> = const { Cell::new(None) };
    /// `PlayerChangeBeaconEffectEvent.setConsumeItem(false)`.
    static KEEP_PAYMENT: Cell<bool> = const { Cell::new(false) };
    /// `PlayerLecternPageChangeEvent.setNewPage`.
    static NEW_PAGE: Cell<Option<i32>> = const { Cell::new(None) };
    /// `PlayerAttackEntityCooldownResetEvent`'s cancel: `attack_ticks` of the guest, and how to reach it.
    static ATTACK_TICKS: Cell<Option<PlayTicks>> = const { Cell::new(None) };
    static KEEP_ATTACK: Cell<bool> = const { Cell::new(false) };
    /// The chunks each guest was last seen to have, by connection.
    static CHUNKS: RefCell<std::collections::HashMap<u32, horizon_client::fast_hash::FxHashSet<(i32, i32)>>> = RefCell::new(std::collections::HashMap::new());
    static EGG_PASS: Cell<bool> = const { Cell::new(false) };
    /// Each guest as `moves_check` last told of them: position, look and the moves they'd sent.
    static WALKED: RefCell<std::collections::HashMap<u128, MovedTo>> = RefCell::new(std::collections::HashMap::new());
    /// `PlayerMoveEvent`'s answer: where they end up, in place of where they asked.
    static MOVE_TO: Cell<Option<[f64; 5]>> = const { Cell::new(None) };
}

thread_local! {
    /// `PlayerTeleportEvent`'s answer, and the teleport going through with it.
    static TP_TO: Cell<Option<[f64; 5]>> = const { Cell::new(None) };
    static TP_PASS: Cell<bool> = const { Cell::new(false) };
    /// What's asking for the teleport about to happen.
    static TP_CAUSE: Cell<Option<&'static str>> = const { Cell::new(None) };
    /// Results a listener put in a 2x2 grid, which the game works out again on every click: by
    /// connection, the grid it was for and the stack.
    static GRID_RESULTS: RefCell<Vec<(u32, [Option<ItemStack>; 4], ItemStack)>> = const { RefCell::new(Vec::new()) };
    /// `PlayerItemConsumeEvent.setReplacement`.
    static REPLACED: RefCell<Option<ItemStack>> = const { RefCell::new(None) };
    /// `PlayerElytraBoostEvent.setShouldConsume(false)`.
    static BOOST_KEEP: Cell<bool> = const { Cell::new(false) };
    static BOOST_PASS: Cell<bool> = const { Cell::new(false) };
    /// `BrewEvent`'s results as a listener left them.
    static BREWED: RefCell<Option<Vec<Option<ItemStack>>>> = const { RefCell::new(None) };
    /// `PlayerBedEnterEvent.setUseBed(ALLOW)`.
    static BED_ALLOW: Cell<bool> = const { Cell::new(false) };
    /// `PlayerBedFailEnterEvent.setMessage`.
    static BED_LINE: RefCell<Option<String>> = const { RefCell::new(None) };
    /// The container a hopper is pulling from.
    static PULLING: Cell<Option<(i32, i32, i32)>> = const { Cell::new(None) };
    /// `PlayerSwapHandItemsEvent`'s hands, in place of the swap.
    static HANDS: RefCell<Option<(Option<ItemStack>, Option<ItemStack>)>> = const { RefCell::new(None) };
    /// How a use in progress is read, from `hold_packet`.
    static USING: Cell<Option<PlayUsing>> = const { Cell::new(None) };
    static AWARD_PASS: Cell<bool> = const { Cell::new(false) };
    /// `PlayerAdvancementDoneEvent.message`: `Some(None)` says nothing.
    static AWARD_LINE: RefCell<Option<Option<String>>> = const { RefCell::new(None) };
    static MAP_PASS: Cell<bool> = const { Cell::new(false) };
}

thread_local! {
    static NEW_MAP: RefCell<Option<ItemStack>> = const { RefCell::new(None) };
    static VELOCITY_PASS: Cell<bool> = const { Cell::new(false) };
    static NEW_VELOCITY: Cell<Option<[f64; 3]>> = const { Cell::new(None) };
    /// `PlayerNameEntityEvent`'s changes: name, whether it stays, and which mob.
    static NAME: RefCell<Option<(Option<String>, Option<bool>, Option<u32>)>> = const { RefCell::new(None) };
    static NAME_PASS: Cell<bool> = const { Cell::new(false) };
    static NEW_BOOK: RefCell<Option<ItemStack>> = const { RefCell::new(None) };
    static BOOK_PASS: Cell<bool> = const { Cell::new(false) };
    static HOPPER_DONE: Cell<bool> = const { Cell::new(false) };
    static RESPAWN_PEEK: Cell<Option<u128>> = const { Cell::new(None) };
    static RESPAWN_TO: RefCell<Option<(String, [f64; 3], f32, f32)>> = const { RefCell::new(None) };
    static RESPAWN_ASKED: RefCell<std::collections::HashMap<u128, u64>> = RefCell::new(std::collections::HashMap::new());
    static STEPS_REFUSED: RefCell<std::collections::HashSet<u128>> = RefCell::new(std::collections::HashSet::new());
    static TP_LEVEL: RefCell<Option<String>> = const { RefCell::new(None) };
    static TP_ELSEWHERE: RefCell<std::collections::HashSet<u32>> = RefCell::new(std::collections::HashSet::new());
    /// `setHealth(0)`, which is no one's doing: "died", not "was killed".
    static PLAIN_KILL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static GATEWAYS: RefCell<std::collections::HashMap<u32, [i32; 3]>> = RefCell::new(std::collections::HashMap::new());
    static INVENTORIES_BACK: RefCell<Vec<u128>> = const { RefCell::new(Vec::new()) };
    static MODE_BY: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static MODE_MESSAGE: RefCell<Option<Value>> = const { RefCell::new(None) };
    static MODES_LATER: RefCell<Vec<(u32, i32, &'static str)>> = const { RefCell::new(Vec::new()) };
    static TP_BY_PLUGIN: Cell<u32> = const { Cell::new(0) };
    static RESPAWN_BY_PLUGIN: RefCell<std::collections::HashSet<u128>> = RefCell::new(std::collections::HashSet::new());
    static COMMAND_AS: Cell<Option<(u128, u128, u64)>> = const { Cell::new(None) };
    static MOVE_WORD: Cell<Option<(bool, bool)>> = const { Cell::new(None) };
    static GATEWAY_TO: Cell<Option<[f64; 3]>> = const { Cell::new(None) };
    /// What the click being run may not do: use the block, use the item.
    static USES: Cell<(bool, bool)> = const { Cell::new((false, false)) };
    static USE_WORD: Cell<Option<(bool, bool)>> = const { Cell::new(None) };
    /// The last click on a block: who, where, when, and if the item was denied. The item's own packet
    /// after it is the same click: `firedInteract`.
    static FIRED: Cell<Option<(u32, [i32; 3], u64, bool)>> = const { Cell::new(None) };
    static UNLEASH_DROP: Cell<Option<bool>> = const { Cell::new(None) };
    static BUCKET_RESULT: RefCell<Option<Option<ItemStack>>> = const { RefCell::new(None) };
    static BOOK_META: RefCell<Option<BookMeta>> = const { RefCell::new(None) };
    static NEW_FRAME: RefCell<Option<Option<ItemStack>>> = const { RefCell::new(None) };
    static FRAME_PASS: Cell<bool> = const { Cell::new(false) };
    /// Why each guest left, as the hub says, for `PlayerQuitEvent`.
    static LEFT: RefCell<Vec<(u128, String)>> = const { RefCell::new(Vec::new()) };
    /// Each guest's socket, kept past their leaving.
    static SOCKETS: RefCell<std::collections::HashMap<u128, String>> = RefCell::new(std::collections::HashMap::new());
    /// Who the event last heard has loaded in.
    static LOADED: RefCell<Vec<u128>> = const { RefCell::new(Vec::new()) };
    /// Each guest's respawn point as last seen.
    /// What a listener said of the spawn point being set.
    static SPAWN_SAID: RefCell<SpawnSaid> = RefCell::new(SpawnSaid::default());
    /// Each sleeper's spawn point from before the bed changed it.
    static BED_BEFORE: RefCell<std::collections::HashMap<u128, SpawnPoint>> = RefCell::new(std::collections::HashMap::new());
    /// `PlayerBedLeaveEvent.setSpawnLocation`, while it fires.
    static BED_KEEP: Cell<bool> = const { Cell::new(true) };
    /// The last tick the failed moves were read in.
    /// This tick's lines, for every plugin that asks.
    /// The lines no listener kept from the log.
    /// Each player's 36 slots as last seen.
    static SLOTS_SEEN: RefCell<std::collections::HashMap<u128, Vec<Value>>> = RefCell::new(std::collections::HashMap::new());
    /// `PlayerInventorySlotChangeEvent.shouldTriggerAdvancements`, while it fires.
    static SLOT_TRIGGER: Cell<bool> = const { Cell::new(true) };
    /// What the event left of the drops for the shearing that follows.
    static SHEAR_OUT: RefCell<Option<Vec<ItemStack>>> = const { RefCell::new(None) };
    /// What a listener set the drops to, while it fires.
    static SHEAR_SET: RefCell<Option<Vec<ItemStack>>> = const { RefCell::new(None) };
    /// The berries rolled for a harvest, by the block, until it happens.
    static HARVESTS: RefCell<std::collections::HashMap<[i32; 3], Vec<ItemStack>>> = RefCell::new(std::collections::HashMap::new());
    /// Tracking a listener refused, by viewer and entity, with the tick it's next asked about.
    static TRACKS: RefCell<std::collections::HashMap<(i32, i32), u64>> = RefCell::new(std::collections::HashMap::new());
    /// What a decorator made of a chat line.
    static DECORATED: RefCell<Option<String>> = const { RefCell::new(None) };
}

thread_local! {
    /// `PlayerPurchaseEvent`'s changes: the offer, whether it pays experience, and whether it counts a use.
    /// `PlayerEggThrowEvent`'s answer: hatching, how many, of what.
    static HATCH: RefCell<Option<(bool, i32, String)>> = const { RefCell::new(None) };
    static TRADE: RefCell<Option<(Option<Value>, Option<bool>, bool)>> = const { RefCell::new(None) };
    /// `install`'s once.
    static INSTALLED: Cell<bool> = const { Cell::new(false) };
    /// Where each player's last use landed.
    static CLICKED: RefCell<Vec<(u128, [f64; 3])>> = const { RefCell::new(Vec::new()) };
}

const MARK: &str = "\u{1}ev_player";

/// Whether a queued command is only the marker that wakes `run_guest_commands`.
pub(in crate::game) fn is_mark(command: &str) -> bool {
    command == MARK
}

/// A use being held: the slot and how long.
pub(in crate::game) type PlayUsing = for<'a> fn(&'a GuestPlay) -> Option<(usize, u32)>;

#[derive(Clone, Copy)]
struct MovedTo {
    at: [f64; 3],
    look: [f32; 2],
    moves: i32,
    level: u64,
    ground: bool,
}

type SpawnPoint = (Option<[f64; 3]>, Option<(String, [i32; 3], f32, bool)>);

struct Held {
    conn: u32,
    packet: S,
    payload: Value,
    /// What a pick packet asked for.
    picked: Option<ItemStack>,
}

/// Fires a hook's event with the game open to the listeners' questions: true if one refused. No
/// flush: what they said to a player waits for the tick's end, where the hub is in `game.lan`.
pub(in crate::game) fn fire(game: &mut Game<'_>, event: &str, payload: Value) -> bool {
    runtime::lend(game, || runtime::fire(event, &payload.to_string())) == 2
}

/// `EquipmentSlot`'s name of the hand the game is acting with.
pub(in crate::game) fn hand(game: &Game<'_>) -> &'static str {
    if matches!(game.hand, Hand::Off) { "OFF_HAND" } else { "HAND" }
}

pub(in crate::game) fn stack_value(stack: &ItemStack) -> Value {
    crate::game::paper_query::stack_json(stack)
}

/// What the player family hears without a bridge row: from the first packet a listener sees.
fn install() {
    if INSTALLED.replace(true) {
        return;
    }
    horizon_client::paper_hub::set_decorator(decorate);
}

/// Whether `PlayerBedEnterEvent`'s checks are heard.
pub(in crate::game) fn bed_heard() -> bool {
    INSTALLED.get()
}

/// Whether a player's tracking stopping is heard.
pub(in crate::game) fn tracking_heard() -> bool {
    INSTALLED.get()
}

/// `PlayerBedEnterEvent` for what the game's own checks would say: a problem stops the player lying down.
pub(in crate::game) fn bed_allow_sleeping(game: &mut Game<'_>, player: &crate::game::native_server::NativeServerPlayer, pos: [i32; 3]) -> Option<crate::game::paper_events::entity_sleep_events::BedProblem> {
    use crate::game::paper_events::entity_sleep_events as sleep;
    // Try_sleep's own checks come after this: what they'd say is worked out for the event.
    BED_FORCE.set(false);
    let at = player.position;
    let far = |bed: [i32; 3]| (at[0] - f64::from(bed[0]) - 0.5).abs() > 3.0 || (at[1] - f64::from(bed[1])).abs() > 2.0 || (at[2] - f64::from(bed[2]) - 0.5).abs() > 3.0;
    let time = game.world.day_time.rem_euclid(24000);
    let result = if game.dim.id().trim_start_matches("minecraft:") != "overworld" {
        "NOT_POSSIBLE_HERE"
    } else if far(pos) {
        "TOO_FAR_AWAY"
    } else if !(12542..23460).contains(&time) && game.world.weather.levels(1.0).1 <= 0.9 {
        "NOT_POSSIBLE_NOW"
    } else {
        // The rest is `ALLOW_NEARBY_MONSTERS`, which knows the answer.
        return None;
    };
    BED_LINE.set(None);
    BED_BLAST.set(false);
    BED_ALLOW.set(false);
    let payload = json!({ "uuid": format!("{:032x}", player.uuid), "name": player.name, "level": player.dimension, "pos": pos, "result": result });
    let refused = fire(game, "pl_bed", payload);
    refusal_blast(game, pos, refused);
    BED_FORCE.set(BED_ALLOW.take() && !refused);
    if BED_FORCE.get() {
        return None;
    }
    if let Some(line) = BED_LINE.take() {
        game.native_tell(player.uuid, horizon_client::text::Text::literal(line));
        return Some(sleep::BedProblem::Other);
    }
    refused.then_some(sleep::BedProblem::Other)
}

/// `PlayerBedEnterEvent` for monsters near: `clear` is the game's answer.
pub(in crate::game) fn bed_nearby_monsters(game: &mut Game<'_>, player: &crate::game::native_server::NativeServerPlayer, pos: [i32; 3], clear: bool) -> crate::game::paper_events::Decision {
    use crate::game::paper_events::Decision as ActionResult;
    BED_ALLOW.set(false);
    BED_BLAST.set(false);
    let result = if clear { "OK" } else { "NOT_SAFE" };
    let payload = json!({ "uuid": format!("{:032x}", player.uuid), "name": player.name, "level": player.dimension, "pos": pos, "result": result });
    let refused = fire(game, "pl_bed", payload);
    refusal_blast(game, pos, refused);
    if refused {
        ActionResult::Fail
    } else if BED_ALLOW.take() {
        ActionResult::Success
    } else {
        ActionResult::Pass
    }
}

/// Tracked again later, it's asked about again.
pub(in crate::game) fn on_tracking_stop(game: &mut Game<'_>, entity: i32, player: u128) {
    if let Some(g) = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.uuid == player)) {
        horizon_client::paper_hub::set_track_allowed((g.entity_id, entity), false);
    }
}

/// `PlayerItemFrameChangeEvent`'s facts: null for a click that won't change a frame.
pub(in crate::game) fn frame_change(game: &Game<'_>, attack: bool, named: Option<i32>) -> Value {
    let found = match named {
        Some(id) => game.item_frames.iter().position(|frame| frame.id as i32 == id),
        None => game.frame_hit().map(|hit| hit.0),
    };
    let Some(i) = found else {
        return Value::Null;
    };
    let frame = &game.item_frames[i];
    if frame.fixed {
        return Value::Null;
    }
    let action = match (attack, frame.item.is_some()) {
        (false, false) if game.held().is_none() => return Value::Null,
        (false, false) => "PLACE",
        (false, true) => "ROTATE",
        (true, true) => "REMOVE",
        (true, false) => return Value::Null,
    };
    let item = frame.item.as_ref().or(game.held());
    json!({ "entity": frame.id, "action": action, "item": item.map(stack_value), "hand": hand(game) })
}

/// The head of `tick_furnace`: `FurnaceBurnEvent` as it lights, `FurnaceStartSmeltEvent` as it starts.
pub(in crate::game) fn furnace_tick(game: &mut Game<'_>, pos: (i32, i32, i32), tick: impl Fn(&mut Game<'_>, (i32, i32, i32)) -> horizon_client::Result<()>) -> ControlFlow<horizon_client::Result<()>> {
    if TICKING.get() {
        return ControlFlow::Continue(());
    }
    let Some(BlockEntity::Furnace(f)) = game.world.block_entities.get(&pos) else {
        return ControlFlow::Continue(());
    };
    let Some(input) = f.items[0].clone() else {
        return ControlFlow::Continue(());
    };
    let recipes = world_recipes!(game);
    let Some(recipe) = recipes.cook(f.kind.recipe_type(), &input) else {
        return ControlFlow::Continue(());
    };
    let result = &recipe.result;
    let max = game.res.items().get(&result.item).map_or(64, |item| item.max_stack);
    let room = f.items[2].as_ref().is_none_or(|out| out.item == result.item && out.count + result.count <= max);
    let lit = f.lit_remaining > 1;
    let fuel = f.items[1].clone();
    if !room || !(lit || fuel.is_some()) {
        return ControlFlow::Continue(());
    }
    let burns = !lit;
    let starts = f.cooking_timer == 0;
    if !burns && !starts {
        return ControlFlow::Continue(());
    }
    let cook = recipe.time;
    let fast = f.kind.fast();
    let burn_time = fuel.as_ref().map_or(0, |fuel| game.res.providers().burn_time(game.res.items(), fuel, fast));
    let kind_item = stack_value(&input);
    let id = recipe.id.clone();
    let block = json!([pos.0, pos.1, pos.2]);
    let level = game.dim.id().to_owned();
    BURN.set((None, None, false));
    if burns && fuel.is_some() && burn_time > 0 {
        let payload = json!({ "kind": "burn", "pos": block, "level": level, "fuel": fuel.as_ref().map(stack_value), "burn_time": burn_time });
        if fire(game, "pl_furnace", payload) {
            return ControlFlow::Break(Ok(()));
        }
    }
    if starts {
        fire(game, "pl_furnace", json!({ "kind": "start", "pos": block, "level": level, "source": kind_item, "recipe": id, "cook_time": cook }));
    }
    let (burn, total, keep) = BURN.take();
    if burn.is_none() && total.is_none() && !keep {
        return ControlFlow::Continue(());
    }
    TICKING.set(true);
    let done = tick(game, pos);
    TICKING.set(false);
    if let Some(BlockEntity::Furnace(f)) = game.world.block_entities.get_mut(&pos) {
        if let Some(burn) = burn {
            f.lit_remaining = burn;
            f.lit_total = burn;
        }
        if let Some(total) = total {
            f.cooking_total = total;
        }
        if keep {
            f.items[1] = fuel;
        }
    }
    ControlFlow::Break(done)
}

/// The head of `tick_brewing_stand`: `BrewingStandFuelEvent` as it takes powder, `BrewEvent` as the time's up.
pub(in crate::game) fn brew_tick(game: &mut Game<'_>, pos: (i32, i32, i32), tick: impl Fn(&mut Game<'_>, (i32, i32, i32)) -> horizon_client::Result<()>) -> ControlFlow<horizon_client::Result<()>> {
    if TICKING.get() {
        return ControlFlow::Continue(());
    }
    let Some(BlockEntity::Brewing(b)) = game.world.block_entities.get(&pos) else {
        return ControlFlow::Continue(());
    };
    let block = json!([pos.0, pos.1, pos.2]);
    let level = game.dim.id().to_owned();
    let powder = b.items[4].clone();
    let items = stacks(b.items.iter().map(Option::as_ref));
    let fuel_level = b.fuel;
    let same = b.items[3].as_ref().map(|s| s.item.clone()) == b.ingredient && b.ingredient.is_some();
    let brews = b.brew_time == 1 && same;
    let empty = b.fuel <= 0 && powder.is_some();
    let copy = brews.then(|| b.clone());
    BURN.set((None, None, false));
    if empty {
        let payload = json!({ "kind": "fuel", "pos": block, "level": level, "fuel": powder.as_ref().map(stack_value), "power": 20 });
        if fire(game, "pl_brew", payload) {
            return ControlFlow::Break(Ok(()));
        }
    }
    if let Some(mut copy) = copy {
        // What `tick` will make, on a copy.
        copy.tick(&world_stations!(game).brewing, world_items!(game), game.res.providers());
        let results = stacks(copy.items.iter().take(3).map(Option::as_ref));
        let payload = json!({ "kind": "brew", "pos": block, "level": level, "type": "BREWING", "size": 5, "items": items, "fuel_level": fuel_level, "results": results });
        BREWED.set(None);
        if fire(game, "pl_brew", payload) {
            // The time's up, with nothing made: it starts over, as in Bukkit.
            if let Some(BlockEntity::Brewing(b)) = game.world.block_entities.get_mut(&pos) {
                b.brew_time = 0;
            }
            return ControlFlow::Break(Ok(()));
        }
    }
    let (power, _, keep) = BURN.take();
    let brewed = BREWED.take();
    if power.is_none() && !keep && brewed.is_none() {
        return ControlFlow::Continue(());
    }
    TICKING.set(true);
    let done = tick(game, pos);
    TICKING.set(false);
    if let Some(BlockEntity::Brewing(b)) = game.world.block_entities.get_mut(&pos) {
        if let Some(power) = power {
            b.fuel = power;
            b.total_fuel = power;
        }
        if keep {
            b.items[4] = powder;
        }
        if let Some(brewed) = brewed {
            for (slot, stack) in b.items.iter_mut().zip(brewed) {
                *slot = stack;
            }
        }
    }
    ControlFlow::Break(done)
}

/// The item entity a hopper is about to take: the nearest one holding this stack.
pub(in crate::game) fn sucked_item(game: &Game<'_>, ground: bool, hopper: Option<(i32, i32, i32)>, stack: &ItemStack) -> Option<u32> {
    let hopper = hopper.filter(|_| ground)?;
    let (x, y, z) = (f64::from(hopper.0) + 0.5, f64::from(hopper.1) + 0.5, f64::from(hopper.2) + 0.5);
    let distance = |item: &&horizon_client::entities::ItemEntity| {
        let (dx, dy, dz) = (item.position.x - x, item.position.y - y, item.position.z - z);
        dx * dx + dy * dy + dz * dz
    };
    let near = game.dropped.iter().filter(|item| item.stack == *stack && distance(item) < 16.0);
    near.min_by(|a, b| distance(a).total_cmp(&distance(b))).map(|item| item.id)
}

/// What `equip_from_hand` will swap the held stack with.
pub(in crate::game) fn worn_in(game: &Game<'_>, stack: &ItemStack) -> Value {
    let slot = game.res.items().get(&stack.item).and_then(|item| item.equip_slot.as_deref());
    let index = match slot {
        Some("head") => 0,
        Some("chest") => 1,
        Some("legs") => 2,
        Some("feet") => 3,
        _ => return Value::Null,
    };
    game.armor[index].as_ref().map_or(Value::Null, stack_value)
}

/// `PlayerDeepSleepEvent`: the tick before a sleeper counts for the night. A refusal keeps them short of it.
pub(in crate::game) fn deep_sleep(game: &mut Game<'_>, host: &LanHost, gi: usize) -> ControlFlow<()> {
    let g = &host.hub.guests[gi];
    if g.bed.is_none() || g.sleep_timer != 99 {
        return ControlFlow::Continue(());
    }
    let payload = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension });
    if fire(game, "pl_deep_sleep", payload) { ControlFlow::Break(()) } else { ControlFlow::Continue(()) }
}

/// The `acting` op's answer: who the game is acting for, with no hub to ask.
fn acting(game: &mut Game<'_>) -> Value {
    json!({ "uuid": format!("{:032x}", game.native_acting_id()), "name": game.acting_player_name(), "hand": hand(game) })
}

/// A catch's experience: the event's to change, and `fish_orbs` makes the game's first orb that size.
pub(in crate::game) fn fish_exp(catch: bool, game: &mut Game<'_>) -> Option<u32> {
    if !catch {
        return None;
    }
    let exp = game.world.random.next_int_bound(6).unwrap_or(0) as u32 + 1;
    FISH_EXP.set(Some((exp, game.ticks)));
    Some(exp)
}

/// The head of `spawn_orbs`: the catch's orb, at the size the event left it.
pub(in crate::game) fn fish_orbs(game: &mut Game<'_>, at: [f64; 3], spawn: impl Fn(&mut Game<'_>, [f64; 3], u32)) -> ControlFlow<()> {
    let Some((exp, tick)) = FISH_EXP.take() else {
        return ControlFlow::Continue(());
    };
    if tick != game.ticks {
        return ControlFlow::Continue(());
    }
    if exp > 0 {
        spawn(game, at, exp);
    }
    ControlFlow::Break(())
}

/// The head of `give_orb_points`: `PlayerExpChangeEvent`, and the amount it settled on.
pub(in crate::game) fn exp_change(game: &mut Game<'_>, gained: u32, give: impl Fn(&mut Game<'_>, u32)) -> ControlFlow<()> {
    if EXP_PASS.get() {
        return ControlFlow::Continue(());
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    EXP.set(None);
    fire(game, "pl_exp", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "amount": gained, "orb": LAST_ORB.get() }));
    let Some(amount) = EXP.take() else {
        return ControlFlow::Continue(());
    };
    if amount > 0 {
        EXP_PASS.set(true);
        give(game, amount);
        EXP_PASS.set(false);
    }
    ControlFlow::Break(())
}

/// `ExperienceOrb.repairPlayerItems` with `PlayerItemMendEvent` in it: the game's own loop, as
/// `enchant_effects::mending` has it, with the choice of item made before the event.
pub(in crate::game) fn mend(game: &mut Game<'_>, amount: u32) -> ControlFlow<u32> {
    use horizon_client::enchant::EffectContext;
    let mut amount = amount;
    let mending = |stack: &Option<ItemStack>| stack.as_ref().is_some_and(|s| s.damage > 0 && s.enchantment_level("minecraft:mending") > 0);
    loop {
        // -1 the main hand, -2 the off hand, else an `armor` index: feet first, as `EquipmentSlot`'s order.
        let mut candidates: Vec<i32> = Vec::new();
        if game.inventory.get(game.selected).is_some_and(|held| mending(held)) {
            candidates.push(-1);
        }
        if mending(&game.offhand) {
            candidates.push(-2);
        }
        candidates.extend((0..game.armor.len()).rev().filter(|&i| mending(&game.armor[i])).map(|i| i as i32));
        if candidates.is_empty() {
            return ControlFlow::Break(amount);
        }
        let slot = candidates[game.world.random.next_int_bound(candidates.len() as i32).unwrap_or(0) as usize];
        let held = match slot {
            -1 => game.inventory[game.selected].clone(),
            -2 => game.offhand.clone(),
            i => game.armor[i as usize].clone(),
        };
        let Some(stack) = held else {
            return ControlFlow::Break(amount);
        };
        let ctx = EffectContext { items: Some(world_items!(game)), ..Default::default() };
        let to_repair_xp = world_stations!(game).enchantments.modify_value(Some(&stack), "minecraft:repair_with_xp", &ctx, &mut game.world.random, amount as f32) as u32;
        let repaired = to_repair_xp.min(stack.damage);
        let used = if to_repair_xp == 0 { 0 } else { repaired * amount / to_repair_xp };
        let name = match slot {
            -1 => "HAND",
            -2 => "OFF_HAND",
            i => ["HEAD", "CHEST", "LEGS", "FEET"][(i as usize).min(3)],
        };
        let (uuid, player, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
        let payload = json!({ "uuid": format!("{uuid:032x}"), "name": player, "level": level, "stack": stack_value(&stack), "slot": name, "repair": repaired, "used": used, "amount": amount, "orb": LAST_ORB.get() });
        MEND.set(None);
        if fire(game, "pl_mend", payload) {
            return ControlFlow::Break(amount);
        }
        let (repaired, used) = MEND.take().unwrap_or((repaired, used));
        let place = match slot {
            -1 => &mut game.inventory[game.selected],
            -2 => &mut game.offhand,
            i => &mut game.armor[i as usize],
        };
        if let Some(stack) = place.as_mut() {
            stack.damage = stack.damage.saturating_sub(repaired);
        }
        if repaired == 0 || to_repair_xp == 0 {
            return ControlFlow::Break(0);
        }
        let remaining = amount.saturating_sub(used);
        if remaining == 0 {
            return ControlFlow::Break(0);
        }
        amount = remaining;
    }
}

/// The head of `guest_orbs`: the orb the game would take is chosen here, with the game's random,
/// so the events can name it; the rest stand aside while the game takes it.
pub(in crate::game) fn orb_gate(game: &mut Game<'_>, host: &mut LanHost, gi: usize, delay: PlayTicks, take: impl Fn(&mut Game<'_>, &mut LanHost, usize)) -> ControlFlow<()> {
    if ORB_PASS.get() {
        return ControlFlow::Continue(());
    }
    let g = &host.hub.guests[gi];
    if g.spectator() || g.dead || g.health <= 0.0 {
        return ControlFlow::Continue(());
    }
    let (conn, at, height) = (g.conn, g.position, f64::from(g.dimensions().height));
    let (uuid, name, level) = (format!("{:032x}", g.uuid), g.name.clone(), g.dimension.clone());
    if *delay(host.play.entry(conn).or_default()) > 1 {
        return ControlFlow::Continue(());
    }
    let (min, max) = ([at[0] - 1.3, at[1] - 0.5, at[2] - 1.3], [at[0] + 1.3, at[1] + height + 0.5, at[2] + 1.3]);
    let touching: Vec<usize> = game.orbs.iter().enumerate()
        .filter(|(_, o)| o.position.x + 0.25 > min[0] && o.position.x - 0.25 < max[0] && o.position.y + 0.5 > min[1] && o.position.y < max[1] && o.position.z + 0.25 > min[2] && o.position.z - 0.25 < max[2])
        .map(|(i, _)| i)
        .collect();
    if touching.is_empty() {
        return ControlFlow::Continue(());
    }
    let pick = touching[game.world.random.next_int_bound(touching.len() as i32).unwrap_or(0) as usize];
    let (orb, value) = (game.orbs[pick].id, game.orbs[pick].value);
    if fire(game, "pl_orb", json!({ "kind": "pickup", "uuid": uuid, "name": name, "level": level, "orb": orb, "value": value })) {
        let wait = delay(host.play.entry(conn).or_default());
        *wait = wait.saturating_sub(1);
        return ControlFlow::Break(());
    }
    LAST_ORB.set(Some(orb));
    // Largest first: each removal leaves the indices before it good.
    let mut aside = Vec::new();
    for &i in touching.iter().rev().filter(|&&i| i != pick) {
        aside.push(game.orbs.remove(i));
    }
    ORB_PASS.set(true);
    take(game, host, gi);
    ORB_PASS.set(false);
    game.orbs.extend(aside);
    LAST_ORB.set(None);
    ORB_COOLDOWN.set(None);
    fire(game, "pl_orb", json!({ "kind": "cooldown", "uuid": uuid, "name": name, "level": level, "old": 0, "new": 2 }));
    if let Some(ticks) = ORB_COOLDOWN.take() {
        *delay(host.play.entry(conn).or_default()) = ticks;
    }
    ControlFlow::Break(())
}

/// The head of `add_cooldown`: `PlayerItemGroupCooldownEvent`.
pub(in crate::game) fn cooldown_gate(game: &mut Game<'_>, group: &std::sync::Arc<str>, ticks: i32, add: impl Fn(&mut Game<'_>, std::sync::Arc<str>, i32)) -> ControlFlow<()> {
    if COOLING.get() {
        return ControlFlow::Continue(());
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    COOLDOWN.set(None);
    if fire(game, "pl_cooldown", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "group": &**group, "ticks": ticks })) {
        return ControlFlow::Break(());
    }
    let Some(ticks) = COOLDOWN.take() else {
        return ControlFlow::Continue(());
    };
    COOLING.set(true);
    add(game, group.clone(), ticks);
    COOLING.set(false);
    ControlFlow::Break(())
}

/// The head of `blocked_by_attacker`: `PlayerShieldDisableEvent`, when the blow disables the shield.
pub(in crate::game) fn shield_gate(game: &mut Game<'_>, disable: f32, scale: f32, run: impl Fn(&mut Game<'_>, f32)) -> ControlFlow<()> {
    let ticks = disable * scale;
    if SHIELD_PASS.get() || ticks <= 0.0 {
        return ControlFlow::Continue(());
    }
    let ticks = (ticks * 20.0).round() as u32;
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    SHIELD_TICKS.set(None);
    let refused = fire(game, "pl_shield", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "ticks": ticks, "attacker": SHIELD_ATTACKER.get() }));
    let wanted = SHIELD_TICKS.take();
    if !refused && wanted.is_none() {
        return ControlFlow::Continue(());
    }
    SHIELD_PASS.set(true);
    let ticks = if refused { 0 } else { wanted.unwrap_or(ticks) };
    run(game, if scale > 0.0 { ticks as f32 / 20.0 / scale } else { 0.0 });
    SHIELD_PASS.set(false);
    ControlFlow::Break(())
}

/// Who `after_block` is answering, for `shield_gate`.
pub(in crate::game) fn shield_attacker(attacker: Option<u32>) {
    SHIELD_ATTACKER.set(attacker);
}

/// The head of `finish_consuming`: `PlayerItemConsumeEvent`, and the meal it settled on.
pub(in crate::game) fn consume_gate<I, C>(game: &mut Game<'_>, stack: &ItemStack, parts: impl Fn(&Game<'_>, &ItemStack) -> Option<(I, C)>, eat: impl Fn(&mut Game<'_>, &I, &C, &ItemStack)) -> ControlFlow<()> {
    if EATING.get() {
        return ControlFlow::Continue(());
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let hand = hand(game);
    EATEN.set(None);
    REPLACED.set(None);
    if fire(game, "pl_consume", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "stack": stack_value(stack), "hand": hand })) {
        return ControlFlow::Break(());
    }
    let eaten = EATEN.take();
    let replaced = REPLACED.take();
    if eaten.is_none() && replaced.is_none() {
        return ControlFlow::Continue(());
    }
    let eaten = eaten.unwrap_or_else(|| stack.clone());
    let Some((info, consumable)) = parts(game, &eaten) else {
        return ControlFlow::Continue(());
    };
    EATING.set(true);
    eat(game, &info, &consumable, &eaten);
    EATING.set(false);
    // LivingEntity.completeUsingItem: the hand gets it, as the game would give it the rest.
    if let Some(replaced) = replaced {
        *game.held_mut() = Some(replaced);
    }
    ControlFlow::Break(())
}

/// The head of `throw_item`: what `add_projectile` is about to launch, for `launch`.
pub(in crate::game) fn throw_gate(game: &mut Game<'_>, stack: &ItemStack, throw: impl Fn(&mut Game<'_>, &ItemStack) -> horizon_client::Result<bool>) -> ControlFlow<horizon_client::Result<bool>> {
    if THROWN.with(|thrown| thrown.borrow().is_some()) || !matches!(&*stack.item, "minecraft:snowball" | "minecraft:egg" | "minecraft:blue_egg" | "minecraft:brown_egg" | "minecraft:ender_pearl" | "minecraft:splash_potion" | "minecraft:lingering_potion" | "minecraft:experience_bottle") {
        return ControlFlow::Continue(());
    }
    THROW_KEEP.set(false);
    // A potion or bottle is asked here, before its sound, with the id it will have.
    if matches!(&*stack.item, "minecraft:splash_potion" | "minecraft:lingering_potion" | "minecraft:experience_bottle") {
        game.next_entity += 1;
        let id = game.next_entity;
        let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
        if fire(game, "pl_launch", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "stack": stack_value(stack), "entity": id, "hand": hand(game) })) {
            return ControlFlow::Break(Ok(false));
        }
        THROWN_ID.set(id);
        THROWN.set(Some(Value::Null));
    } else {
        THROWN.set(Some(stack_value(stack)));
    }
    let done = throw(game, stack);
    THROWN.set(None);
    if THROW_KEEP.take() {
        *game.held_mut() = Some(stack.clone());
    }
    ControlFlow::Break(done)
}

/// The head of `add_projectile`, for a throw: `PlayerLaunchProjectileEvent`, with the id it will have.
pub(in crate::game) fn launch(game: &mut Game<'_>, p: &mut horizon_client::projectiles::Projectile) -> ControlFlow<()> {
    let Some(stack) = THROWN.take() else {
        return ControlFlow::Continue(());
    };
    if p.id == 0 {
        game.next_entity += 1;
        p.id = game.next_entity;
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let refused = fire(game, "pl_launch", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "stack": stack, "entity": p.id, "hand": hand(game) }));
    THROWN.set(Some(Value::Null));
    if refused {
        THROW_KEEP.set(true);
        return ControlFlow::Break(());
    }
    ControlFlow::Continue(())
}

/// The head of `projectile_hit`: `PlayerEggThrowEvent`. The game's two rolls are read off a copy of
/// its random; a listener's answer takes the place of what they made.
pub(in crate::game) fn egg_hit(game: &mut Game<'_>, id: u32, egg: bool, at: [f64; 3], guest: Option<u32>, hit: impl Fn(&mut Game<'_>) -> horizon_client::Result<()>) -> ControlFlow<horizon_client::Result<()>> {
    let (true, Some(conn)) = (egg && !EGG_PASS.get(), guest) else {
        return ControlFlow::Continue(());
    };
    let who = game.lan.as_ref().and_then(|host| host.hub.guest(conn)).map(|g| (g.uuid, g.name.clone()));
    let Some((uuid, name)) = who.or_else(|| Some((game.native_acting_id(), game.acting_player_name().to_owned()))) else {
        return ControlFlow::Continue(());
    };
    let mut random = game.world.random.clone();
    let hatching = random.next_int_bound(8).unwrap_or(1) == 0;
    let hatches = if hatching && random.next_int_bound(32).unwrap_or(1) == 0 { 4 } else { 1 };
    let level = game.dim.id().to_owned();
    HATCH.set(None);
    fire(game, "pl_egg", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "entity": id, "hatching": hatching, "hatches": if hatching { hatches } else { 0 }, "type": "minecraft:chicken" }));
    let Some((wanted, count, kind)) = HATCH.take() else {
        return ControlFlow::Continue(());
    };
    let last = game.next_entity;
    EGG_PASS.set(true);
    let done = hit(game);
    EGG_PASS.set(false);
    // What the game hatched goes, and what the event wants comes.
    let chicks: Vec<u32> = game.mobs.iter().filter(|m| m.id > last && m.kind.id.ends_with("chicken")).map(|m| m.id).collect();
    for chick in chicks {
        game.native_remove_entity(&level, chick);
    }
    if wanted {
        for _ in 0..count {
            let spawned = game.spawn_mob(&kind, at, true);
            if kind.ends_with("chicken")
                && let Some(chick) = spawned
                && let Some(mob) = game.mobs.iter_mut().find(|m| m.id == chick)
            {
                mob.extra.age = -24000;
            }
        }
    }
    ControlFlow::Break(done)
}

/// The head of `guest_set_camera`: `PlayerStartSpectatingEntityEvent` and `PlayerStopSpectatingEntityEvent`.
pub(in crate::game) fn camera_gate(game: &mut Game<'_>, host: &LanHost, gi: usize, now: Option<i32>, camera: Option<i32>) -> ControlFlow<()> {
    let g = &host.hub.guests[gi];
    let camera = camera.filter(|id| *id != g.entity_id);
    if now == camera {
        return ControlFlow::Continue(());
    }
    let payload = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension, "entity_id": g.entity_id, "old": now, "new": camera });
    if fire(game, "pl_camera", payload) { ControlFlow::Break(()) } else { ControlFlow::Continue(()) }
}

/// The head of `award`: `PlayerAdvancementDoneEvent`, with the line the game is about to say.
pub(in crate::game) fn award_gate(game: &mut Game<'_>, index: usize, award: impl Fn(&mut Game<'_>, usize)) -> ControlFlow<()> {
    if AWARD_PASS.get() {
        return ControlFlow::Continue(());
    }
    use horizon_client::advancements::Frame;
    let tree = game.advancement_tree();
    let Some(a) = tree.list.get(index) else {
        return ControlFlow::Continue(());
    };
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let line = a.display.as_ref().filter(|d| d.announce).map(|d| {
        let key = match d.frame {
            Frame::Task => "chat.type.advancement.task",
            Frame::Goal => "chat.type.advancement.goal",
            Frame::Challenge => "chat.type.advancement.challenge",
        };
        json!({ "translate": key, "with": [{ "text": name }, d.title] })
    });
    AWARD_LINE.set(None);
    fire(game, "pl_advancement", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "advancement": a.id, "message": line }));
    let Some(wanted) = AWARD_LINE.take() else {
        return ControlFlow::Continue(());
    };
    let before = game.adv.announced.len();
    AWARD_PASS.set(true);
    award(game, index);
    AWARD_PASS.set(false);
    if game.adv.announced.len() > before {
        game.adv.announced.truncate(before);
        if let Some(text) = wanted {
            game.adv.announced.push(horizon_client::text::Text::literal(text));
        }
    }
    ControlFlow::Break(())
}

/// The head of `use_empty_map`, which Paper's event follows: the map is made, then the event sees it.
pub(in crate::game) fn map_gate(game: &mut Game<'_>, stack: &ItemStack, fill: impl Fn(&mut Game<'_>, &ItemStack) -> bool) -> ControlFlow<bool> {
    if MAP_PASS.get() || &*stack.item != "minecraft:map" {
        return ControlFlow::Continue(());
    }
    let before = game.inventory.clone();
    let hand = hand(game);
    MAP_PASS.set(true);
    let done = fill(game, stack);
    MAP_PASS.set(false);
    if !done {
        return ControlFlow::Break(false);
    }
    let is_map = |held: &Option<ItemStack>| held.as_ref().is_some_and(|held| &*held.item == "minecraft:filled_map");
    let slot = game.inventory.iter().enumerate().find(|(i, held)| is_map(held) && before.get(*i) != Some(*held)).map(|(i, _)| i);
    let made = slot.and_then(|slot| game.inventory[slot].clone());
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    NEW_MAP.set(None);
    fire(game, "pl_map_fill", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "stack": stack_value(stack), "made": made.as_ref().map(stack_value), "hand": hand }));
    if let (Some(slot), Some(map)) = (slot, NEW_MAP.take()) {
        game.inventory[slot] = Some(map);
    }
    ControlFlow::Break(true)
}

/// The head of `knock_back_player`: `PlayerVelocityEvent`, with what the knock comes to.
pub(in crate::game) fn velocity_gate(game: &mut Game<'_>, knock: impl Fn(&mut Game<'_>)) -> ControlFlow<()> {
    if VELOCITY_PASS.get() {
        return ControlFlow::Continue(());
    }
    let Some(old) = game.player.as_ref().map(|p| p.velocity) else {
        return ControlFlow::Continue(());
    };
    VELOCITY_PASS.set(true);
    knock(game);
    VELOCITY_PASS.set(false);
    let Some(new) = game.player.as_ref().map(|p| p.velocity) else {
        return ControlFlow::Break(());
    };
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    NEW_VELOCITY.set(None);
    let refused = fire(game, "pl_velocity", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "velocity": [new.x, new.y, new.z] }));
    let wanted = NEW_VELOCITY.take();
    if let Some(p) = game.player.as_mut() {
        if refused {
            p.velocity = old;
        } else if let Some([x, y, z]) = wanted {
            p.velocity = horizon_core::geometry::Vec3::new(x, y, z);
        }
    }
    ControlFlow::Break(())
}

/// The head of `release_trident`: `PlayerRiptideEvent`, when the game is about to launch them.
pub(in crate::game) fn riptide(wet: bool, look: [f64; 3], game: &mut Game<'_>, ticks: u32) -> bool {
    let Some(stack) = game.held().cloned() else {
        return false;
    };
    let level = stack.enchantment_level("minecraft:riptide");
    if ticks < 10 || level == 0 || !wet || game.riding.is_some() {
        return false;
    }
    let strength = 1.5 + 0.75 * (level as f64 - 1.0);
    let (uuid, name, world) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let hand = hand(game);
    return fire(game, "pl_riptide", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": world, "stack": stack_value(&stack), "hand": hand, "velocity": look.map(|d| d * strength) }));
}

/// The head of `find_respawn_and_use`: which block held the respawn, and whether it was gone, for
/// `AbstractRespawnEvent`, which fires once the player is back.
pub(in crate::game) fn respawn_gate<R>(game: &mut Game<'_>, pos: [i32; 3], consume: bool, none: impl Fn(&R) -> bool, find: impl FnOnce(&mut Game<'_>) -> R) -> ControlFlow<R> {
    if RESPAWN_PASS.get() {
        return ControlFlow::Continue(());
    }
    let name = game.res.facts().get(game.world.block(pos[0], pos[1], pos[2])).name.to_string();
    let peek = RESPAWN_PEEK.get();
    let uuid = peek.unwrap_or(game.native_acting_id());
    RESPAWN_PASS.set(true);
    let found = find(game);
    RESPAWN_PASS.set(false);
    let block = if name.ends_with("_bed") { "BED" } else if name == "minecraft:respawn_anchor" { "ANCHOR" } else { "" };
    let plugin = peek.is_none() && RESPAWN_BY_PLUGIN.with(|by| by.borrow_mut().remove(&uuid));
    RESPAWNS.with(|seen| seen.borrow_mut().insert(uuid, json!({ "block": block, "missing": none(&found), "end": !consume && peek.is_none(), "plugin": plugin })));
    ControlFlow::Break(found)
}

/// The head of `name_mob`: `PlayerNameEntityEvent`, and what it changed.
pub(in crate::game) fn name_gate(game: &mut Game<'_>, index: usize, name_it: impl Fn(&mut Game<'_>, usize) -> bool) -> ControlFlow<bool> {
    if NAME_PASS.get() {
        return ControlFlow::Continue(());
    }
    let (Some(name), Some(mob)) = (held_name(game), game.mobs.get(index).map(|mob| mob.id)) else {
        return ControlFlow::Continue(());
    };
    let (uuid, player, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    NAME.set(None);
    if fire(game, "pl_name", json!({ "uuid": format!("{uuid:032x}"), "name": player, "level": level, "entity": mob, "new_name": name })) {
        return ControlFlow::Break(false);
    }
    let Some((new, persistent, other)) = NAME.take() else {
        return ControlFlow::Continue(());
    };
    let index = other.and_then(|id| game.mobs.iter().position(|m| m.id == id)).unwrap_or(index);
    NAME_PASS.set(true);
    let done = name_it(game, index);
    NAME_PASS.set(false);
    if done && let Some(m) = game.mobs.get_mut(index) {
        if new.is_some() {
            m.custom_name = new;
        }
        if let Some(persistent) = persistent {
            m.persistent = persistent;
        }
    }
    ControlFlow::Break(done)
}

/// The head of `use_lectern`: `PlayerInsertLecternBookEvent` for a book going onto an empty lectern.
pub(in crate::game) fn book_gate(game: &mut Game<'_>, pos: [i32; 3], held: Option<&ItemStack>, put: impl Fn(&mut Game<'_>, [i32; 3], Option<&ItemStack>) -> bool) -> ControlFlow<bool> {
    let key = (pos[0], pos[1], pos[2]);
    let empty = matches!(game.world.block_entities.get(&key), Some(BlockEntity::Lectern(lectern)) if lectern.book.is_none());
    let book = held.filter(|held| matches!(&*held.item, "minecraft:writable_book" | "minecraft:written_book"));
    let (false, true, Some(book)) = (BOOK_PASS.get(), empty, book) else {
        return ControlFlow::Continue(());
    };
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    NEW_BOOK.set(None);
    if fire(game, "pl_lectern_put", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "pos": pos, "book": stack_value(book) })) {
        return ControlFlow::Break(true);
    }
    let Some(new) = NEW_BOOK.take() else {
        return ControlFlow::Continue(());
    };
    BOOK_PASS.set(true);
    let done = put(game, pos, held);
    BOOK_PASS.set(false);
    if done && let Some(BlockEntity::Lectern(lectern)) = game.world.block_entities.get_mut(&key) {
        lectern.book = Some(new);
    }
    ControlFlow::Break(done)
}

/// The head of `interact_frame` and `attack_frame`: `PlayerItemFrameChangeEvent`.
pub(in crate::game) fn frame_gate(game: &mut Game<'_>, attack: bool, named: Option<i32>, run: impl Fn(&mut Game<'_>) -> bool) -> ControlFlow<bool> {
    if FRAME_PASS.get() {
        return ControlFlow::Continue(());
    }
    let change = frame_change(game, attack, named);
    if change.is_null() {
        return ControlFlow::Continue(());
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    NEW_FRAME.set(None);
    if fire(game, "pl_frame", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "frame": change })) {
        return ControlFlow::Break(true);
    }
    let Some(new) = NEW_FRAME.take() else {
        return ControlFlow::Continue(());
    };
    let id = change["entity"].as_u64().unwrap_or(0) as u32;
    let place = |game: &mut Game<'_>, new: Option<ItemStack>| {
        if let Some(frame) = game.item_frames.iter_mut().find(|frame| frame.id == id) {
            frame.item = new;
        }
    };
    // What's taken out is what the event says; what's put in or turned, the game does first.
    if change["action"] == "REMOVE" {
        place(game, new.clone());
    }
    FRAME_PASS.set(true);
    let done = run(game);
    FRAME_PASS.set(false);
    if change["action"] != "REMOVE" {
        place(game, new);
    }
    ControlFlow::Break(done)
}

/// Why a guest is going, for `PlayerQuitEvent`, which comes after.
pub(in crate::game) fn note_left(g: &horizon_client::lan::guest::Guest) {
    LEFT.with(|left| {
        let mut left = left.borrow_mut();
        left.retain(|(uuid, _)| *uuid != g.uuid);
        left.push((g.uuid, g.left_reason.clone()));
    });
}

/// What a chat line's decorators may do, as `AsyncChatDecorateEvent`: the answer comes back as an
/// action, since no game is lent there.
fn decorate(sender: Option<u128>, line: horizon_world::nbt::Tag) -> horizon_world::nbt::Tag {
    if !runtime::listens("pl_decorate") {
        return line;
    }
    let text = crate::game::commands::paper_entities::tag_json(&line);
    runtime::fire("pl_decorate", &json!({ "uuid": sender.map(|sender| format!("{sender:032x}")), "message": text }).to_string());
    for action in runtime::take_actions() {
        match serde_json::from_str::<Value>(&action) {
            Ok(value) if value["player"] == "ev_player_decorate" => DECORATED.set(value["text"].as_str().map(str::to_owned)),
            _ => runtime::push(action),
        }
    }
    DECORATED.take().map_or(line, |text| horizon_protocol::text::literal(&text))
}

/// A marker in the hub's menu events, so `run_native_menu_events` runs: it comes right after the
/// hub has read this tick's packets, with the hub back in `game.lan`, before anything acts on them.
pub(in crate::game) fn moves_turn(host: &mut LanHost) {
    use horizon_client::lan::menu::NativeMenuEvent;
    host.hub.native_menu_events.push(NativeMenuEvent::Closed { uuid: 0, window: -1 });
}

fn level_key(name: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
}

/// `PlayerMoveEvent`: each guest the hub just moved, as Bukkit counts a move, with a listener's
/// answer applied before the game has done anything with where they are.
pub(in crate::game) fn moves_check(game: &mut Game<'_>) {
    use horizon_client::lan::menu::NativeMenuEvent;
    commands_check(game);
    pickups_check(game);
    wakes_check(game);
    tracks_check(game);
    kicks_check(game);
    if runtime::listens("pl_turn") {
        fire(game, "pl_turn", Value::Null);
    }
    let Some(host) = game.lan.as_mut() else {
        return;
    };
    host.hub.native_menu_events.retain(|event| !matches!(event, NativeMenuEvent::Closed { uuid: 0, window: -1 }));
    let mut moved = Vec::new();
    for g in host.hub.guests.iter().filter(|g| g.placed && !g.dead) {
        let now = MovedTo { at: g.position, look: [g.yaw, g.pitch], moves: g.received_moves, level: level_key(&g.dimension), ground: g.on_ground };
        let last = WALKED.with(|seen| seen.borrow().get(&g.uuid).copied());
        let Some(last) = last.filter(|last| last.level == now.level && g.awaiting_teleport.is_none()) else {
            WALKED.with(|seen| seen.borrow_mut().insert(g.uuid, now));
            continue;
        };
        if last.moves == now.moves {
            // The game moved them, not the client.
            if last.at != now.at {
                WALKED.with(|seen| seen.borrow_mut().insert(g.uuid, now));
            }
            continue;
        }
        let d2: f64 = (0..3).map(|k| (now.at[k] - last.at[k]).powi(2)).sum();
        let turned = (0..2).map(|k| (now.look[k] - last.look[k]).abs()).sum::<f32>();
        if d2 <= 1.0 / 256.0 && turned <= 10.0 {
            WALKED.with(|seen| seen.borrow_mut().entry(g.uuid).or_insert(now).moves = now.moves);
            continue;
        }
        moved.push((g.uuid, g.name.clone(), g.dimension.clone(), last, now));
    }
    for (uuid, name, level, last, now) in moved {
        let (from, to) = ([last.at[0], last.at[1], last.at[2], f64::from(last.look[0]), f64::from(last.look[1])], [now.at[0], now.at[1], now.at[2], f64::from(now.look[0]), f64::from(now.look[1])]);
        MOVE_TO.set(None);
        let payload = json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "from": from, "to": to });
        let jumped = last.ground && !now.ground && now.at[1] > last.at[1];
        let refused = jumped && runtime::listens("pl_jump") && paper_bridge::on_refusable(game, "pl_jump", payload.clone());
        let refused = paper_bridge::on_refusable(game, "pl_moves", payload) || refused;
        let wanted = MOVE_TO.take().or(refused.then_some(from));
        let mut stands = now;
        if let (Some(target), Some(host)) = (wanted, game.lan.as_mut())
            && let Some(gi) = host.hub.guests.iter().position(|g| g.uuid == uuid)
        {
            let g = &mut host.hub.guests[gi];
            (g.yaw, g.pitch) = (target[3] as f32, target[4] as f32);
            host.hub.teleport(gi, Some([target[0], target[1], target[2]]));
            stands.at = [target[0], target[1], target[2]];
            stands.look = [target[3] as f32, target[4] as f32];
        }
        WALKED.with(|seen| seen.borrow_mut().insert(uuid, stands));
    }
}

/// What `cmd_teleport` is: `Player.teleport` said so beforehand, or it's a command.
pub(in crate::game) fn tp_cause(game: &Game<'_>) {
    let by_player = game.command_source.is_some_and(|source| source.entity.is_some());
    let by_plugin = !by_player && TP_BY_PLUGIN.get() > 0;
    if by_plugin {
        TP_BY_PLUGIN.set(TP_BY_PLUGIN.get() - 1);
    }
    TP_CAUSE.set(Some(if by_plugin { "PLUGIN" } else { "COMMAND" }));
}

fn tp_payload(game: &mut Game<'_>, to: [f64; 3], look: Option<[f32; 2]>, cause: &str) -> Option<Value> {
    let p = game.player.as_ref()?;
    let from = [p.position.x, p.position.y, p.position.z, f64::from(game.yaw), f64::from(game.pitch)];
    let look = look.unwrap_or([game.yaw, game.pitch]);
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    Some(json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "from": from, "to": [to[0], to[1], to[2], f64::from(look[0]), f64::from(look[1])], "cause": cause }))
}

/// Who a command runs as, after `setPlayer`.
pub(in crate::game) fn command_as(game: &Game<'_>, host: &LanHost, conn: u32) -> u32 {
    let Some((by, other, tick)) = COMMAND_AS.take() else {
        return conn;
    };
    let asked = host.hub.guests.iter().any(|g| g.conn == conn && g.uuid == by);
    host.hub.guests.iter().find(|g| asked && tick == game.ticks && g.uuid == other).map_or(conn, |g| g.conn)
}

/// The head of `teleport_to_in_level` for the player whose turn it is: `PlayerTeleportEvent`.
pub(in crate::game) fn teleport_gate(game: &mut Game<'_>, player: bool, pos: [f64; 3], look: Option<[f32; 2]>, run: impl Fn(&mut Game<'_>, [f64; 3], Option<[f32; 2]>)) -> ControlFlow<()> {
    if !player || TP_PASS.get() || game.lan_acting.is_none() {
        return ControlFlow::Continue(());
    }
    let cause = TP_CAUSE.take().unwrap_or("UNKNOWN");
    let Some(payload) = tp_payload(game, pos, look, cause) else {
        return ControlFlow::Continue(());
    };
    TP_TO.set(None);
    if fire(game, "pl_teleport", payload) {
        return ControlFlow::Break(());
    }
    if let Some(conn) = game.lan_acting
        && tp_elsewhere(game, conn)
    {
        return ControlFlow::Break(());
    }
    let Some(to) = TP_TO.take() else {
        return ControlFlow::Continue(());
    };
    TP_PASS.set(true);
    run(game, [to[0], to[1], to[2]], Some([to[3] as f32, to[4] as f32]));
    TP_PASS.set(false);
    ControlFlow::Break(())
}

fn place_player(game: &mut Game<'_>, to: [f64; 5]) {
    let at = horizon_core::geometry::Vec3::new(to[0], to[1], to[2]);
    if let Some(p) = game.player.as_mut() {
        p.position = at;
    }
    game.previous_position = at;
    game.yaw = to[3] as f32;
    game.pitch = to[4] as f32;
}

/// The head of `projectile_hit` for a player's ender pearl: `PlayerTeleportEvent`, cause `ENDER_PEARL`.
pub(in crate::game) fn pearl_hit(game: &mut Game<'_>, pearl: bool, to: [f64; 3], hit: impl Fn(&mut Game<'_>) -> horizon_client::Result<()>) -> ControlFlow<horizon_client::Result<()>> {
    if !pearl || TP_PASS.get() {
        return ControlFlow::Continue(());
    }
    let Some(payload) = tp_payload(game, to, None, "ENDER_PEARL") else {
        return ControlFlow::Continue(());
    };
    TP_TO.set(None);
    if fire(game, "pl_teleport", payload) {
        return ControlFlow::Break(Ok(()));
    }
    if let Some(conn) = game.lan_acting
        && tp_elsewhere(game, conn)
    {
        return ControlFlow::Break(Ok(()));
    }
    let Some(wanted) = TP_TO.take() else {
        return ControlFlow::Continue(());
    };
    TP_PASS.set(true);
    let done = hit(game);
    TP_PASS.set(false);
    place_player(game, wanted);
    ControlFlow::Break(done)
}

/// The head of `teleport_randomly`: a chorus fruit's jump is only known once it's made, so the
/// event comes then, before the turn ends, and a refusal puts them back.
pub(in crate::game) fn chorus(game: &mut Game<'_>, jump: impl Fn(&mut Game<'_>)) -> ControlFlow<()> {
    if TP_PASS.get() {
        return ControlFlow::Continue(());
    }
    let Some(p) = game.player.as_ref() else {
        return ControlFlow::Continue(());
    };
    let (at, before) = (p.position, [p.position.x, p.position.y, p.position.z, f64::from(game.yaw), f64::from(game.pitch)]);
    TP_PASS.set(true);
    jump(game);
    TP_PASS.set(false);
    let Some(now) = game.player.as_ref().map(|p| p.position).filter(|now| *now != at) else {
        return ControlFlow::Break(());
    };
    // The event reads as if they hadn't gone yet.
    place_player(game, before);
    let Some(payload) = tp_payload(game, [now.x, now.y, now.z], None, "CONSUMABLE_EFFECT") else {
        return ControlFlow::Break(());
    };
    TP_TO.set(None);
    let refused = fire(game, "pl_teleport", payload);
    let elsewhere = game.lan_acting.is_some_and(|conn| tp_elsewhere(game, conn));
    if !refused && !elsewhere {
        place_player(game, TP_TO.take().unwrap_or([now.x, now.y, now.z, before[3], before[4]]));
    }
    ControlFlow::Break(())
}

/// The head of `guest_spectate_teleport`: `PlayerTeleportEvent`, cause `SPECTATE`.
pub(in crate::game) fn spectate_gate(game: &mut Game<'_>, host: &mut LanHost, gi: usize, target: u128) -> ControlFlow<()> {
    let Some(to) = host.hub.guests.iter().find(|g| g.uuid == target).map(|g| [g.position[0], g.position[1], g.position[2], f64::from(g.yaw), f64::from(g.pitch)]) else {
        return ControlFlow::Continue(());
    };
    let g = &host.hub.guests[gi];
    let payload = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension, "from": [g.position[0], g.position[1], g.position[2], f64::from(g.yaw), f64::from(g.pitch)], "to": to, "cause": "SPECTATE" });
    TP_TO.set(None);
    if fire(game, "pl_teleport", payload) {
        return ControlFlow::Break(());
    }
    if tp_elsewhere(game, host.hub.guests[gi].conn) {
        return ControlFlow::Break(());
    }
    let Some(wanted) = TP_TO.take() else {
        return ControlFlow::Continue(());
    };
    let g = &mut host.hub.guests[gi];
    (g.yaw, g.pitch) = (wanted[3] as f32, wanted[4] as f32);
    host.hub.teleport(gi, Some([wanted[0], wanted[1], wanted[2]]));
    ControlFlow::Break(())
}

/// `use_sign_empty_hand`: the hand, this tick.
pub(in crate::game) fn sign_used(game: &Game<'_>) {
    SIGN_USED.set(game.ticks);
}

/// Why a sign editor opens: `PLACE`, unless a hand just used the sign.
pub(in crate::game) fn sign_cause(game: &Game<'_>) -> &'static str {
    if SIGN_USED.get() == game.ticks { "INTERACT" } else { "PLACE" }
}

pub(in crate::game) fn sign_side(front: bool) {
    SIGN_FRONT.set(front);
}

pub(in crate::game) fn sign_front() -> bool {
    SIGN_FRONT.get()
}

/// The head of `native_fill_bucket`, which fills and empties alike: the player's bucket events.
pub(in crate::game) fn bucket_gate(game: &mut Game<'_>, stack: &ItemStack, pos: [i32; 3], canceled: &mut bool) {
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let face = game.pick().map(|hit| hit.face);
    let hand = hand(game);
    // What's in the hand after it: the bucket of what's there, or the empty one.
    let makes = if &*stack.item == "minecraft:bucket" {
        let block = &*game.res.facts().get(game.world.block(pos[0], pos[1], pos[2])).name;
        match block {
            "minecraft:powder_snow" => "minecraft:powder_snow_bucket",
            "minecraft:lava" => "minecraft:lava_bucket",
            _ => "minecraft:water_bucket",
        }
    } else {
        "minecraft:bucket"
    };
    let payload = json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "stack": stack_value(stack), "result": stack_value(&ItemStack::new(makes, 1)), "pos": pos, "face": face, "hand": hand });
    BUCKET_RESULT.take();
    *canceled |= fire(game, "pl_bucket", payload);
}

/// What `setItemStack` left for the bucket just used: the stack it makes, `Some(None)` for nothing.
pub(in crate::game) fn bucket_left() -> Option<Option<ItemStack>> {
    BUCKET_RESULT.take()
}

/// The filled bucket going into the hand: `PlayerBucketFillEvent`'s word on it, if it had one.
pub(in crate::game) fn bucket_filled(filled: ItemStack) -> ItemStack {
    bucket_left().flatten().unwrap_or(filled)
}

/// `PlayerUnleashEntityEvent`, a player's hand or shears about to take a lead off `id`: whether the
/// lead drops, or `None` if it stays on.
pub(in crate::game) fn unleash(game: &mut Game<'_>, id: u32, drop: bool) -> Option<bool> {
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let hand = hand(game);
    UNLEASH_DROP.set(None);
    if fire(game, "pl_unleash", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "entity": id, "held": true, "hand": hand, "drop": drop })) {
        return None;
    }
    horizon_client::mobs::ee::unleash_as("PLAYER_UNLEASH");
    Some(UNLEASH_DROP.take().unwrap_or(drop))
}

/// What's changing this player's game mode: a plugin's `setGameMode` said so, or it's `/gamemode`.
pub(in crate::game) fn mode_cause(game: &Game<'_>) -> &'static str {
    let name = game.acting_player_name();
    let by_plugin = MODE_BY.with(|by| {
        let mut by = by.borrow_mut();
        let found = by.iter().position(|who| who == name);
        found.map(|at| by.remove(at))
    });
    if by_plugin.is_some() { "PLUGIN" } else { "COMMAND" }
}

/// What `cancelMessage` left for the one who typed the command.
pub(in crate::game) fn mode_message() -> Option<Value> {
    MODE_MESSAGE.with(|message| message.borrow_mut().take())
}

/// A mode change away from any command, with the hub out of the game's hands: it waits for the
/// turn if a plugin wants to hear of it. False if it doesn't, and the caller goes on.
pub(in crate::game) fn mode_held(conn: u32, mode: i32, cause: &'static str) -> bool {
    if !runtime::listens("pl_game_mode") {
        return false;
    }
    MODES_LATER.with(|later| later.borrow_mut().push((conn, mode, cause)));
    true
}

fn modes_check(game: &mut Game<'_>) {
    const NAMES: [&str; 4] = ["survival", "creative", "adventure", "spectator"];
    for (conn, mode, cause) in MODES_LATER.with(|later| std::mem::take(&mut *later.borrow_mut())) {
        let Some(g) = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.conn == conn && g.game_mode != mode)) else {
            continue;
        };
        let name = NAMES.get(usize::try_from(mode).unwrap_or(0)).copied().unwrap_or("survival");
        let payload = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension, "mode": name, "changes": true, "cause": cause });
        if !fire(game, "pl_game_mode", payload)
            && let Some(mut host) = game.lan.take()
        {
            host.hub.set_game_mode(conn, mode, &game.world, Some(game.collision));
            game.lan = Some(host);
        }
    }
}

/// The tail of `throw_from_eyes`, which is `Player.drop`: `PlayerDropItemEvent`, with the item in the
/// world. Refused, it goes back where it came from.
pub(in crate::game) fn dropped(game: &mut Game<'_>, from_hand: bool) {
    if !runtime::listens("pl_drop") {
        return;
    }
    let Some(item) = game.just_spawned_item() else {
        return;
    };
    let (id, stack) = (item.id, item.stack.clone());
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    if !fire(game, "pl_drop", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "entity": id, "stack": stack_value(&stack) })) {
        return;
    }
    game.dropped.retain(|item| item.id != id);
    let held = game.held().cloned();
    match held {
        None if from_hand => *game.held_mut() = Some(stack),
        Some(held) if from_hand && stack.count == 1 && held.same_item_same_components(&stack) && held.count < max_stack(game, Some(&held)) => {
            if let Some(held) = game.held_mut().as_mut() {
                held.count += 1;
            }
        }
        _ => {
            let _ = game.give(stack);
        }
    }
    INVENTORIES_BACK.with(|back| back.borrow_mut().push(uuid));
}

/// The client took those drops off its own screen: it's told again.
fn inventories_back(game: &mut Game<'_>) {
    let Some(host) = game.lan.as_mut() else {
        return;
    };
    for uuid in INVENTORIES_BACK.with(|back| std::mem::take(&mut *back.borrow_mut())) {
        if let Some(gi) = host.hub.guests.iter().position(|g| g.uuid == uuid) {
            host.hub.send_inventory(gi);
        }
    }
}

/// Where the connections asking stand: their saved place, or for someone new the world's spawn.
fn spawns_turn(game: &mut Game<'_>) {
    use horizon_client::paper_hub as hub;
    let mut loads = std::mem::take(&mut *hub::SPAWN_LOADS.lock().unwrap_or_else(|e| e.into_inner()));
    loads.retain_mut(|load| {
        if load.level != game.dim.id() {
            return true;
        }
        if game.world.chunks.contains_key(&load.chunk) {
            if load.asked {
                game.streaming.extra.retain(|extra| extra.0 != load.chunk);
            }
            let _ = load.done.send(());
            return false;
        }
        if !load.asked {
            load.asked = true;
            game.streaming.extra.push((load.chunk, 1));
        }
        true
    });
    hub::SPAWN_LOADS.lock().unwrap_or_else(|e| e.into_inner()).extend(loads);
    let asks = std::mem::take(&mut *hub::SPAWN_ASKS.lock().unwrap_or_else(|e| e.into_inner()));
    for (uuid, answer) in asks {
        let saved = horizon_client::lan::playerdata::saved_place(&game.world_root, uuid);
        let new = saved.is_none();
        let (level, at, yaw, pitch) = saved.unwrap_or_else(|| {
            let world = game.level.world_spawn.map_or([0.5, 80.0, 0.5], |[x, y, z]| [f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5]);
            let feet = if game.dim.kind() == Dim::Overworld { game.server_player_spawn(world) } else { world };
            (Dim::Overworld.id().to_owned(), feet, 0.0, 0.0)
        });
        let _ = answer.send(json!({ "level": level, "at": at, "yaw": yaw, "pitch": pitch, "new": new }));
    }
}

/// The head of a guest's placing: where `AsyncPlayerSpawnLocationEvent` left them.
pub(in crate::game) fn spawn_chosen(g: &mut horizon_client::lan::guest::Guest) {
    if let Some((level, at, yaw, pitch)) = horizon_client::paper_hub::spawn_chosen(g.uuid) {
        g.dimension = level;
        g.position = at;
        g.previous_position = at;
        (g.yaw, g.pitch) = (yaw, pitch);
    }
}

/// `PlayerRespawnEvent`, asked before the respawn with the hub in the game's hands. The level with
/// their bed or anchor opens for the look, its chunks read from disk, and nothing is used up.
fn respawn_ask(game: &mut Game<'_>, conn: u32) {
    let Some(mut host) = game.lan.take() else {
        return;
    };
    let Some(g) = host.hub.guests.iter().find(|g| g.conn == conn) else {
        game.lan = Some(host);
        return;
    };
    let (uuid, name, home, config) = (g.uuid, g.name.clone(), g.spawn, g.respawn_elsewhere.as_ref().map(crate::game::respawn_anchor::RespawnConfig::from_saved));
    let world = game.level.world_spawn.map(|[x, y, z]| [f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5]);
    let spawn = home.or(world).or(game.spawn.map(|s| [s.x, s.y, s.z])).unwrap_or([0.5, 80.0, 0.5]);
    let plugin = RESPAWN_BY_PLUGIN.with(|by| by.borrow_mut().remove(&uuid));
    RESPAWNS.with(|seen| seen.borrow_mut().insert(uuid, json!({ "block": "", "missing": false, "end": false, "plugin": plugin })));
    let mut facts = (Dim::Overworld.id().to_owned(), spawn, g_yaw(&host, conn), 0.0_f32);
    if let Some(c) = config {
        let back = game.dim;
        let open = c.dim == back || game.enter_level(Some(&mut host), c.dim).is_ok();
        if open {
            RESPAWN_PEEK.set(Some(uuid));
            let found = game.with_saved_chunks(c.pos, |game| game.find_respawn_and_use(c, false));
            RESPAWN_PEEK.set(None);
            if let Ok(Some((at, yaw))) = found {
                facts = (c.dim.id().to_owned(), [at.x, at.y, at.z], yaw, 0.0);
            }
            if c.dim != back && let Err(e) = game.enter_level(Some(&mut host), back) {
                log::error!("could not reopen {}: {e}", back.id());
            }
        }
        RESPAWNS.with(|seen| {
            if let Some(row) = seen.borrow_mut().get_mut(&uuid) {
                row["plugin"] = json!(plugin);
            }
        });
    }
    game.lan = Some(host);
    let (level, at, yaw, pitch) = facts;
    RESPAWN_TO.with(|to| to.borrow_mut().take());
    RESPAWN_ASKED.with(|asked| asked.borrow_mut().insert(uuid, game.ticks));
    fire(game, "pl_respawn_ask", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "to": [at[0], at[1], at[2], yaw, pitch] }));
}

fn g_yaw(host: &LanHost, conn: u32) -> f32 {
    host.hub.guests.iter().find(|g| g.conn == conn).map_or(0.0, |g| g.yaw)
}

/// What `setRespawnLocation` left: the level and the place, with the look put on the guest.
pub(in crate::game) fn respawn_chosen(g: &mut horizon_client::lan::guest::Guest) -> Option<(Dim, [f64; 3])> {
    let (level, at, yaw, pitch) = RESPAWN_TO.with(|to| to.borrow_mut().take())?;
    (g.yaw, g.pitch) = (yaw, pitch);
    Some((Dim::from_id(&level), at))
}

/// `PlayerInteractEvent` with `PHYSICAL` for a step, with the player whose block it is.
fn step_event(game: &mut Game<'_>, uuid: u128, name: &str, level: &str, pos: [i32; 3]) -> bool {
    fire(game, "pl_step", json!({ "kind": "step", "uuid": format!("{uuid:032x}"), "name": name, "level": level, "pos": pos }))
}

/// The turn's steps. A plate or a tripwire asks about each player in it, as `getSignalStrength` does:
/// each tick till it's pressed, then at its checks. Refused, they don't count for it. The blocks
/// motion would change wait here for their answer.
fn steps_turn(game: &mut Game<'_>) {
    if !runtime::listens("pl_step") {
        STEPS_REFUSED.with(|refused| refused.borrow_mut().clear());
        return;
    }
    let Some(host) = game.lan.as_ref() else {
        return;
    };
    let guests: Vec<_> = host.hub.guests.iter().filter(|g| host.hub.here(g) && g.loaded && !g.dead && !g.spectator()).map(|g| (g.uuid, g.name.clone(), g.dimension.clone(), g.bounds())).collect();
    for (uuid, name, level, bounds) in guests {
        let (lo, hi) = (bounds.min_position(), bounds.max_position());
        let y = lo.y.floor() as i32;
        let mut pressed = None;
        for x in lo.x.floor() as i32..=hi.x.floor() as i32 {
            for z in lo.z.floor() as i32..=hi.z.floor() as i32 {
                let state = game.world.block(x, y, z);
                let Some(found) = game.pack.assets.registry.state(state) else {
                    continue;
                };
                if found.name.contains("pressure_plate") || found.name == "minecraft:tripwire" {
                    let on = found.properties.get("powered").is_some_and(|value| value == "true") || found.properties.get("power").is_some_and(|value| value != "0");
                    pressed = pressed.or(Some(([x, y, z], on)));
                }
            }
        }
        let Some((pos, on)) = pressed else {
            STEPS_REFUSED.with(|refused| refused.borrow_mut().remove(&uuid));
            continue;
        };
        if on && game.ticks % 20 != 0 {
            continue;
        }
        let refused = step_event(game, uuid, &name, &level, pos);
        STEPS_REFUSED.with(|all| {
            if refused {
                all.borrow_mut().insert(uuid);
            } else {
                all.borrow_mut().remove(&uuid);
            }
        });
    }
    let held = std::mem::take(&mut *horizon_client::paper_hub::STEPS_HELD.lock().unwrap_or_else(|e| e.into_inner()));
    for (pos, next) in held {
        let at = |g: &&horizon_client::lan::guest::Guest| (g.position[0] - (f64::from(pos[0]) + 0.5)).abs() < 1.0 && (g.position[2] - (f64::from(pos[2]) + 0.5)).abs() < 1.0 && (g.position[1] - f64::from(pos[1])).abs() < 2.0;
        let Some((conn, uuid, name, level)) = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(at)).map(|g| (g.conn, g.uuid, g.name.clone(), g.dimension.clone())) else {
            continue;
        };
        if step_event(game, uuid, &name, &level, pos) {
            continue;
        }
        let state = next.unwrap_or(horizon_world::registry::BlockStateId(0));
        let dim = game.dim.id().to_owned();
        game.native_set_block(&dim, pos, state, 3);
        if let Some(host) = game.lan.as_mut()
            && let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn)
            && host.hub.guests[gi].position[1] < f64::from(pos[1]) + 1.0
        {
            let g = &host.hub.guests[gi];
            let up = [g.position[0], f64::from(pos[1]) + 1.0, g.position[2]];
            host.hub.teleport_keeping_motion(gi, up);
        }
    }
}

/// True if this player doesn't count on the plate or tripwire they're on.
pub(in crate::game) fn step_uncounted(uuid: u128) -> bool {
    STEPS_REFUSED.with(|refused| refused.borrow().contains(&uuid))
}

/// `RedStoneOreBlock.stepOn`: true if a player's step doesn't light it.
pub(in crate::game) fn ore_refused(game: &mut Game<'_>, pos: [i32; 3]) -> bool {
    if !runtime::listens("pl_step") {
        return false;
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    step_event(game, uuid, &name, &level, pos)
}

/// `setTo` with another world: the teleport here gives way to one there, with this event as its own.
fn tp_elsewhere(game: &mut Game<'_>, conn: u32) -> bool {
    let Some(level) = TP_LEVEL.with(|level| level.borrow_mut().take()) else {
        return false;
    };
    if level == game.dim.id() {
        return false;
    }
    let Some(to) = TP_TO.take() else {
        return false;
    };
    TP_ELSEWHERE.with(|all| all.borrow_mut().insert(conn));
    game.level_teleports.push(crate::game::server_travel::LevelTeleport { conn, dim: Dim::from_id(&level), at: [to[0], to[1], to[2]], rotation: Some([to[3] as f32, to[4] as f32]) });
    true
}

/// The head of a teleport to another level: `PlayerTeleportEvent`, unless one has been asked already.
pub(in crate::game) fn level_tp_gate(game: &mut Game<'_>, host: &LanHost, mut t: crate::game::server_travel::LevelTeleport) -> Option<crate::game::server_travel::LevelTeleport> {
    if TP_ELSEWHERE.with(|all| all.borrow_mut().remove(&t.conn)) || !runtime::listens("pl_teleport") {
        return Some(t);
    }
    let g = host.hub.guest(t.conn)?;
    let cause = TP_CAUSE.take().unwrap_or("UNKNOWN");
    let look = t.rotation.unwrap_or([g.yaw, g.pitch]);
    let payload = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension, "from": [g.position[0], g.position[1], g.position[2], f64::from(g.yaw), f64::from(g.pitch)], "to": [t.at[0], t.at[1], t.at[2], f64::from(look[0]), f64::from(look[1])], "to_level": t.dim.id(), "cause": cause });
    TP_TO.set(None);
    TP_LEVEL.with(|level| level.borrow_mut().take());
    if fire(game, "pl_teleport", payload) {
        return None;
    }
    if let Some(to) = TP_TO.take() {
        t.at = [to[0], to[1], to[2]];
        t.rotation = Some([to[3] as f32, to[4] as f32]);
        if let Some(level) = TP_LEVEL.with(|level| level.borrow_mut().take()) {
            t.dim = Dim::from_id(&level);
        }
    }
    Some(t)
}

/// A gateway's way, waiting for the chunks there: the event comes when the exit is known.
pub(in crate::game) fn gateway_pending(conn: u32, gateway: [i32; 3]) {
    GATEWAYS.with(|all| all.borrow_mut().insert(conn, gateway));
}

/// The exit found: `PlayerTeleportEndGatewayEvent`, with the exact spot.
pub(in crate::game) fn gateway_arrived(game: &mut Game<'_>, host: &LanHost, conn: u32, at: horizon_core::geometry::Vec3) -> Option<horizon_core::geometry::Vec3> {
    let Some(gateway) = GATEWAYS.with(|all| all.borrow_mut().remove(&conn)) else {
        return Some(at);
    };
    gateway_asked(game, host, conn, gateway, at).map(|(at, _)| at)
}

/// What the chat threads want of the game: `PlayerChatEvent`, their questions, and while any line is
/// on its way, a copy of each player's state for them to read as they please.
fn chat_turn(game: &mut Game<'_>) {
    use horizon_client::paper_hub as hub;
    let running = hub::CHATS_RUNNING.load(std::sync::atomic::Ordering::Relaxed) > 0;
    let mut copy = None;
    if running && let Some(host) = game.lan.as_ref() {
        let uuids: Vec<String> = host.hub.guests.iter().map(|g| format!("{:032x}", g.uuid)).collect();
        let mut all = std::collections::HashMap::new();
        for uuid in uuids {
            let answer = crate::game::paper_query::answer(game, &json!({ "op": "player", "uuid": uuid }).to_string());
            all.insert(uuid, answer);
        }
        copy = Some(all);
    }
    *hub::CHAT_PLAYERS.lock().unwrap_or_else(|e| e.into_inner()) = copy;
    let main = std::mem::take(&mut *hub::CHAT_MAIN.lock().unwrap_or_else(|e| e.into_inner()));
    for (text, answer) in main {
        let (_, carried) = runtime::lend(game, || runtime::fire_carried("pl_chat_main", &text));
        let _ = answer.send(carried.unwrap_or(text));
    }
    // A listener's questions come one after another: answering a batch a tick would give a line a tick
    // per question, seconds with a plugin that asks a lot. While a line is on its way they're answered as they
    // come, for a moment of each tick.
    let until = std::time::Instant::now() + std::time::Duration::from_millis(if running { 4 } else { 0 });
    loop {
        let asks = std::mem::take(&mut *hub::CHAT_ASKS.lock().unwrap_or_else(|e| e.into_inner()));
        for (request, answer) in asks {
            let _ = answer.send(crate::game::paper_query::answer(game, &request));
        }
        if std::time::Instant::now() >= until || hub::CHATS_RUNNING.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            break;
        }
        std::thread::yield_now();
    }
}

/// True for a block a step can change.
pub(in crate::game) fn maybe_step(block: &str) -> bool {
    block.contains("pressure_plate") || block.contains("tripwire") || block.ends_with("farmland")
}

/// `PlayerInteractEvent` with `PHYSICAL`, for a block that just changed under a player. True puts it back.
pub(in crate::game) fn step_refused(game: &mut Game<'_>, change: &Value) -> bool {
    if !runtime::listens("pl_step") {
        return false;
    }
    let (Some(old), Some(new)) = (change["old"].as_str(), change["new"].as_str()) else {
        return false;
    };
    let current = |key: &str| change[key].as_i64().unwrap_or(0);
    let plate = (old.contains("pressure_plate") || old.contains("tripwire")) && change["kind"] == "redstone" && current("new_current") > current("old_current");
    let trampled = old.starts_with("minecraft:farmland") && new.starts_with("minecraft:dirt");
    if !plate && !trampled {
        return false;
    }
    let at = |i: usize| change["pos"][i].as_f64().unwrap_or(0.0);
    let on = |g: &&horizon_client::lan::guest::Guest| json!(g.dimension) == change["level"] && (g.position[0] - (at(0) + 0.5)).abs() < 0.8 && (g.position[2] - (at(2) + 0.5)).abs() < 0.8 && (g.position[1] - at(1)).abs() < 1.5;
    let Some(g) = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(on)) else {
        return false;
    };
    let payload = json!({ "kind": "step", "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension, "pos": change["pos"] });
    fire(game, "pl_step", payload)
}

/// `interact_mob`'s mob, if a water bucket is about to scoop it: id, kind and the bucket it makes.
pub(in crate::game) fn scooped(game: &Game<'_>, bucket_of: impl Fn(&horizon_client::mobs::Mob) -> Option<&'static str>) -> Value {
    let Some((index, distance)) = game.mob_hit() else {
        return Value::Null;
    };
    let mob = &game.mobs[index];
    if game.held().map(|held| &*held.item) != Some("minecraft:water_bucket") || !mob.alive() {
        return Value::Null;
    }
    if let (Some(hit), Some(eye)) = (game.pick(), game.eye_position()) {
        let d = ((hit.point[0] - eye[0]).powi(2) + (hit.point[1] - eye[1]).powi(2) + (hit.point[2] - eye[2]).powi(2)).sqrt();
        if d < distance {
            return Value::Null;
        }
    }
    let Some(bucket) = bucket_of(mob) else {
        return Value::Null;
    };
    json!({ "entity": mob.id, "kind": &*mob.kind.id, "bucket": bucket })
}

/// Where a click lands, for a listener of the use it turns into.
pub(in crate::game) fn note_click(host: &LanHost, conn: u32, packet: &S) {
    let at = match packet {
        S::UseItemOn(p) => [0, 1, 2].map(|k| f64::from(p.pos[k]) + f64::from(p.cursor[k])),
        S::Interact(p) => p.location,
        _ => return,
    };
    let Some(uuid) = host.hub.guest(conn).map(|g| g.uuid) else {
        return;
    };
    CLICKED.with(|clicked| {
        let mut clicked = clicked.borrow_mut();
        clicked.retain(|(player, _)| *player != uuid);
        clicked.push((uuid, at));
    });
}

/// The head of `wear`: `PlayerItemDamageEvent`, after Unbreaking, as in Bukkit.
pub(in crate::game) fn wear_gate(game: &mut Game<'_>, stack: &ItemStack, amount: u32) -> ControlFlow<u32> {
    let loss = game.durability_loss(stack, amount);
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    LOSS.set(loss);
    let refused = fire(game, "pl_item_damage", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "stack": stack_value(stack), "amount": amount, "loss": loss }));
    let loss = if refused { 0 } else { LOSS.take() };
    if loss > 0 {
        game.trigger_item_durability(stack, stack.damage + loss);
    }
    ControlFlow::Break(loss)
}

/// The head of `boost_firework`: `PlayerElytraBoostEvent`, with the id the rocket will have.
pub(in crate::game) fn boost_gate(game: &mut Game<'_>, stack: &ItemStack, boost: impl Fn(&mut Game<'_>, &ItemStack) -> bool) -> ControlFlow<bool> {
    if BOOST_PASS.get() || &*stack.item != "minecraft:firework_rocket" || !game.player.as_ref().is_some_and(|p| p.fall_flying) {
        return ControlFlow::Continue(());
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let hand = hand(game);
    BOOST_KEEP.set(false);
    if fire(game, "pl_boost", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "stack": stack_value(stack), "hand": hand, "entity": game.next_entity + 1 })) {
        return ControlFlow::Break(false);
    }
    if !BOOST_KEEP.take() {
        return ControlFlow::Continue(());
    }
    BOOST_PASS.set(true);
    let done = boost(game, stack);
    BOOST_PASS.set(false);
    if done {
        *game.held_mut() = Some(stack.clone());
    }
    ControlFlow::Break(done)
}

/// A hopper starting on a source of its own: what follows comes from there, or from nowhere.
pub(in crate::game) fn pulling(from: Option<(i32, i32, i32)>) {
    PULLING.set(from);
}

pub(in crate::game) fn pulled() -> Option<[i32; 3]> {
    PULLING.get().map(|pos| [pos.0, pos.1, pos.2])
}

/// A menu closed by the game, not the client: `InventoryCloseEvent`, with why.
pub(in crate::game) fn close_gate(game: &mut Game<'_>, host: &mut LanHost, conn: u32, window: Option<Option<i32>>) {
    if PASSING.get() {
        return;
    }
    let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn) else {
        return;
    };
    let open = match window {
        Some(Some(window)) => Some(window),
        Some(None) => find_open(host, conn),
        None => host.hub.guests[gi].menu.as_ref().filter(|menu| menu.slots != Slots::Native).map(|menu| menu.window),
    };
    let Some(open) = open else {
        return;
    };
    let Some(view) = view_of(game, host, gi, open) else {
        return;
    };
    let mut payload = base("close", &host.hub.guests[gi], open, &view);
    payload["reason"] = json!(match window {
        Some(Some(_)) => "CANT_USE",
        Some(None) => "OPEN_NEW",
        None => "UNKNOWN",
    });
    fire(game, "pl_closing", payload);
}

/// The wear a refused `PlayerItemDamageEvent` leaves: none, or what `setDamage` asked.
pub(in crate::game) fn take_moved() -> Option<ItemStack> {
    let moved = MOVED.take();
    MOVED_AGAIN.set(moved.is_some());
    moved
}

pub(in crate::game) fn moved_again() -> bool {
    MOVED_AGAIN.take()
}

/// `name_mob`'s name tag, as the game wrote it.
pub(in crate::game) fn held_name(game: &Game<'_>) -> Option<String> {
    let held = game.held().filter(|held| &*held.item == "minecraft:name_tag")?;
    held.custom_name()
}

/// What `interact_armor_stand` is about to do: the stand, the stand's slot and the two stacks. Null
/// when it won't do anything. `slot_of` is the private `equipment_slot`.
pub(in crate::game) fn stand_use(game: &Game<'_>, slot_of: impl Fn(&Game<'_>, &ItemStack) -> usize) -> Value {
    let Some((i, _, point)) = game.armor_stand_hit() else {
        return Value::Null;
    };
    let stand = &game.armor_stands[i];
    let held = game.held().cloned();
    let slot = match &held {
        Some(held) => {
            let slot = slot_of(game, held);
            if slot < 2 && !stand.show_arms {
                return Value::Null;
            }
            slot
        }
        None => {
            let y = (point[1] - stand.position.y) / if stand.small { 0.5 } else { 1.0 };
            let has = |s: usize| stand.equipment[s].is_some();
            let small = stand.small;
            if y >= 0.1 && y < 0.1 + if small { 0.8 } else { 0.45 } && has(2) {
                2
            } else if y >= 0.9 + if small { 0.3 } else { 0.0 } && y < 0.9 + if small { 1.0 } else { 0.7 } && has(4) {
                4
            } else if y >= 0.4 && y < 0.4 + if small { 1.0 } else { 0.8 } && has(3) {
                3
            } else if y >= 1.6 && has(5) {
                5
            } else if !has(0) && has(1) {
                1
            } else {
                0
            }
        }
    };
    let current = stand.equipment[slot].as_ref();
    if held.is_none() && current.is_none() {
        return Value::Null;
    }
    let slot = ["HAND", "OFF_HAND", "FEET", "LEGS", "CHEST", "HEAD"][slot];
    json!({
        "entity": stand.id,
        "slot": slot,
        "held": held.as_ref().map(stack_value),
        "worn": current.map(stack_value),
        "hand": hand(game),
    })
}

fn stack(stack: Option<&ItemStack>) -> Value {
    stack.map_or(Value::Null, crate::game::paper_query::stack_json)
}

fn stacks<'a>(list: impl Iterator<Item = Option<&'a ItemStack>>) -> Value {
    Value::Array(list.map(stack).collect())
}

/// The game's item from a stack's SNBT, as `ItemStack::to_snbt` writes it.
fn from_snbt(snbt: &str) -> Option<ItemStack> {
    let (tag, _) = horizon_world::nbt::parse_snbt(snbt).ok()?;
    horizon_client::save::item_from_nbt(&tag)
}

fn stack_of(value: &Value) -> Option<ItemStack> {
    from_snbt(value.as_str()?)
}

/// Bukkit's `InventoryType` of an engine menu.
fn type_of_kind(kind: Kind) -> &'static str {
    match kind {
        Kind::Inventory => "CRAFTING",
        Kind::Crafting => "WORKBENCH",
        Kind::Hopper => "HOPPER",
        Kind::Dispenser => "DISPENSER",
        Kind::Shulker => "SHULKER_BOX",
        Kind::Furnace("blasting") => "BLAST_FURNACE",
        Kind::Furnace("smoking") => "SMOKER",
        Kind::Furnace(_) => "FURNACE",
        Kind::Brewing => "BREWING",
        Kind::Crafter | Kind::CrafterBlock => "CRAFTER",
        Kind::Loom => "LOOM",
        Kind::Stonecutter => "STONECUTTER",
        Kind::Grindstone => "GRINDSTONE",
        Kind::Anvil => "ANVIL",
        Kind::Smithing => "SMITHING",
        Kind::Enchanting => "ENCHANTING",
        Kind::Beacon => "BEACON",
        Kind::Cartography => "CARTOGRAPHY",
        Kind::Lectern => "LECTERN",
        Kind::Merchant => "MERCHANT",
        Kind::ItemPicker => "CREATIVE",
        Kind::Chest(_) | Kind::Mount(_) => "CHEST",
    }
}

/// Bukkit's `InventoryType` of a block's container, by the block.
fn type_of_block(game: &Game<'_>, slots: Slots) -> &'static str {
    let pos = match slots {
        Slots::Block(pos) => pos,
        Slots::Ender => return "ENDER_CHEST",
        Slots::Double(..) | Slots::Native => return "CHEST",
    };
    if pos.1 == horizon_client::lan::menu::VEHICLE_KEY_Y {
        return "CHEST";
    }
    let name = &game.res.facts().get(game.world.block(pos.0, pos.1, pos.2)).name;
    match name.trim_start_matches("minecraft:") {
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

/// An open menu as a listener reads it.
struct View {
    kind: &'static str,
    /// The menu's own slots, before the player's.
    own: Vec<Option<ItemStack>>,
    pos: Option<[i32; 3]>,
    /// The recipe a crafting grid holds now.
    recipe: Option<String>,
}

fn find_menu<'a>(host: &'a mut LanHost, conn: u32, window: i32) -> Option<(i32, [i32; 3], &'a mut Menu)> {
    let table = TABLES.get().filter(|find| find(host, conn).is_some_and(|(open, ..)| open == window));
    if let Some(find) = table {
        return find(host, conn);
    }
    let find = STATIONS.get()?;
    find(host, conn).filter(|(open, ..)| *open == window)
}

/// What's open as window `window`. `None` for what isn't a window of the guest's.
fn view_of(game: &Game<'_>, host: &mut LanHost, gi: usize, window: i32) -> Option<View> {
    let conn = host.hub.guests[gi].conn;
    if window == 0 {
        let g = &host.hub.guests[gi];
        let own = (0..46).map(|slot| g.menu_slot(slot)).collect();
        let recipe = game.menu_env().recipes.find(&g.craft_grid, 2).map(|(id, _)| format!("minecraft:{id}"));
        return Some(View { kind: "CRAFTING", own, pos: None, recipe });
    }
    let g = &host.hub.guests[gi];
    if let Some(menu) = g.menu.as_ref().filter(|menu| menu.window == window) {
        let own = menu.slots.read(&game.world, g)?;
        let kind = type_of_block(game, menu.slots);
        return Some(View { kind, own, pos: menu.pos.map(|pos| [pos.0, pos.1, pos.2]), recipe: None });
    }
    if let Some((_, pos, menu)) = find_menu(host, conn, window) {
        let count = menu.slot_count().saturating_sub(36);
        let own: Vec<Option<ItemStack>> = (0..count).map(|slot| menu.slot(slot).cloned()).collect();
        let kind = menu.kind();
        let recipe = (kind == Kind::Crafting).then(|| game.menu_env().recipes.find(&own[1..10], 3).map(|(id, _)| format!("minecraft:{id}"))).flatten();
        return Some(View { kind: type_of_kind(kind), own, pos: Some(pos), recipe });
    }
    if let Some(find) = TRADES.get()
        && let Some((open, slots)) = find(host, conn)
        && open == window
    {
        return Some(View { kind: "MERCHANT", own: slots.to_vec(), pos: None, recipe: None });
    }
    if let Some(find) = LECTERNS.get()
        && let Some((open, key)) = find(host, conn)
        && open == window
    {
        let book = match game.world.block_entities.get(&key) {
            Some(BlockEntity::Lectern(lectern)) => lectern.book.clone(),
            _ => None,
        };
        return Some(View { kind: "LECTERN", own: vec![book], pos: Some([key.0, key.1, key.2]), recipe: None });
    }
    if let Some((find, build, _)) = MOUNTS.get()
        && let Some((open, mob, columns)) = find(host, conn)
        && open == window
    {
        let index = game.mobs.iter().position(|m| m.id == mob)?;
        let menu = build(game, index, columns);
        let own = (0..menu.slot_count().saturating_sub(36)).map(|slot| menu.slot(slot).cloned()).collect();
        return Some(View { kind: "CHEST", own, pos: None, recipe: None });
    }
    None
}

/// The player's 36 slots as a menu shows them: the rows, then the hotbar.
fn rows(g: &horizon_client::lan::guest::Guest) -> Value {
    stacks((9..36).chain(0..9).map(|slot| g.inventory.get(slot).and_then(Option::as_ref)))
}

fn max_stack(game: &Game<'_>, stack: Option<&ItemStack>) -> u32 {
    stack.and_then(|stack| game.res.items().get(&stack.item)).map_or(64, |item| item.max_stack as u32)
}

fn base(kind: &str, g: &horizon_client::lan::guest::Guest, window: i32, view: &View) -> Value {
    json!({
        "kind": kind,
        "uuid": format!("{:032x}", g.uuid),
        "name": g.name,
        "window": window,
        "type": view.kind,
        "size": if view.kind == "CRAFTING" { 5 } else { view.own.len() },
        "items": stacks(view.own.iter().map(Option::as_ref)),
        "rows": rows(g),
        "cursor": stack(g.carried.as_ref()),
        "pos": view.pos,
        "level": g.dimension,
    })
}

/// What a drag would leave where: the raw slots and their new stacks, and the cursor.
fn drag_result(game: &Game<'_>, g: &horizon_client::lan::guest::Guest, view: &View, slots: &[i16], kind: i32) -> (Value, Value) {
    let Some(cursor) = g.carried.as_ref() else {
        return (json!([]), Value::Null);
    };
    let max = max_stack(game, Some(cursor));
    let size = view.own.len();
    let at = |slot: usize| -> Option<ItemStack> {
        if slot < size {
            return view.own[slot].clone();
        }
        let row = slot - size;
        g.inventory.get(if row < 27 { 9 + row } else { row - 27 }).cloned().flatten()
    };
    let each = match kind {
        0 => cursor.count / (slots.len().max(1) as u32),
        1 => 1,
        _ => max,
    };
    let mut left = if kind == 2 { cursor.count } else { cursor.count.saturating_sub(each.saturating_mul(slots.len() as u32)) };
    let mut placed = Vec::new();
    for &slot in slots {
        let have = at(slot as usize);
        let count = have.as_ref().map_or(0, |have| have.count);
        let mut new = have.clone().unwrap_or_else(|| cursor.clone());
        let room = max.saturating_sub(count).min(each);
        new.count = count + room;
        if kind != 2 {
            left += each - room;
        }
        placed.push(json!([slot, crate::game::paper_query::stack_json(&new)]));
    }
    let mut after = cursor.clone();
    after.count = left;
    (Value::Array(placed), if left > 0 { crate::game::paper_query::stack_json(&after) } else { Value::Null })
}

/// The event a click makes, or `None` for one that isn't held.
fn click_payload(game: &Game<'_>, host: &mut LanHost, gi: usize, p: &play::ContainerClick) -> Option<Value> {
    let conn = host.hub.guests[gi].conn;
    if p.input == 5 {
        // A drag: its slots gather as it goes, and only its end asks anything.
        let stage = p.button & 3;
        let slots = DRAGS.with(|drags| {
            let mut drags = drags.borrow_mut();
            let at = drags.iter().position(|(c, _)| *c == conn).unwrap_or_else(|| {
                drags.push((conn, Vec::new()));
                drags.len() - 1
            });
            match stage {
                0 => drags[at].1.clear(),
                1 if !drags[at].1.contains(&p.slot) => drags[at].1.push(p.slot),
                _ => {}
            }
            (stage == 2).then(|| std::mem::take(&mut drags[at].1))
        })?;
        if slots.len() < 2 {
            return None;
        }
        let view = view_of(game, host, gi, p.container)?;
        let g = &host.hub.guests[gi];
        if g.menu.as_ref().is_some_and(|menu| menu.window == p.container && menu.slots == Slots::Native) {
            return None;
        }
        let drag = i32::from(p.button) >> 2;
        let (placed, cursor) = drag_result(game, g, &view, &slots, drag);
        let mut payload = base("drag", g, p.container, &view);
        payload["drag"] = json!(drag);
        payload["slots"] = json!(slots);
        payload["placed"] = placed;
        payload["cursor_after"] = cursor;
        return Some(payload);
    }
    let view = view_of(game, host, gi, p.container)?;
    let g = &host.hub.guests[gi];
    // A native mod's GUI has its own event.
    if g.menu.as_ref().is_some_and(|menu| menu.window == p.container && menu.slots == Slots::Native) {
        return None;
    }
    let mut payload = base("click", g, p.container, &view);
    payload["slot"] = json!(p.slot);
    payload["button"] = json!(p.button);
    payload["input"] = json!(p.input);
    payload["selected"] = json!(g.selected);
    if view.kind == "MERCHANT" {
        let trade = host.play.get(&conn).and_then(|play| play.merchant.as_ref());
        payload["merchant"] = json!(trade.and_then(|trade| trade.merchant.mob));
        let session = trade.map(|trade| &trade.merchant.session);
        payload["offer"] = session.and_then(|session| session.offers.get(session.active.unwrap_or(session.selection_hint))).map_or(Value::Null, offer_value);
        payload["offers"] = session.map_or(Value::Null, |session| Value::Array(session.offers.iter().map(offer_value).collect()));
    }
    payload["recipe"] = json!(view.recipe);
    let raw = usize::try_from(p.slot).ok();
    let own = view.own.len();
    let current = raw.and_then(|raw| if raw < own { view.own[raw].clone() } else { g.inventory.get(if raw - own < 27 { 9 + raw - own } else { raw - own - 27 }).cloned().flatten() });
    payload["current"] = stack(current.as_ref());
    payload["max"] = json!(max_stack(game, current.as_ref().or(g.carried.as_ref())));
    Some(payload)
}

/// The head of `lan_guest_packet`: `Break` keeps the packet for `run_held`.
pub(in crate::game) fn hold_packet(game: &mut Game<'_>, host: &mut LanHost, conn: u32, packet: &S, attack_ticks: PlayTicks, using: PlayUsing) -> ControlFlow<()> {
    install();
    ATTACK_TICKS.set(Some(attack_ticks));
    USING.set(Some(using));
    if let Some(g) = host.hub.guest(conn) {
        SOCKETS.with(|sockets| {
            sockets.borrow_mut().entry(g.uuid).or_insert_with(|| g.address.to_string());
        });
    }
    if PASSING.get() {
        return ControlFlow::Continue(());
    }
    let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn) else {
        return ControlFlow::Continue(());
    };
    if matches!(packet, S::ClientCommand(p) if p.action == 0) && host.hub.guests[gi].dead && runtime::listens("pl_respawn_ask") {
        HELD.with(|held| held.borrow_mut().push(Held { conn, packet: packet.clone(), payload: json!({ "kind": "respawn_ask" }), picked: None }));
        host.commands.push(GuestCommand { conn, command: MARK.to_owned(), permission: 0, integrated: false, quiet: true });
        return ControlFlow::Break(());
    }
    // These are heard from anyone, in any mode.
    let toggled = matches!(packet, S::PlayerInput(_) | S::PlayerAbilities(_) | S::RecipeBookChangeSettings(_)) || matches!(packet, S::PlayerCommand(p) if matches!(p.action, 1 | 2));
    let first = if toggled { toggle_payload(host, gi, packet) } else { heard_first(game, host, gi, conn, packet) };
    if let Some(payload) = first {
        HELD.with(|held| held.borrow_mut().push(Held { conn, packet: packet.clone(), payload, picked: None }));
        host.commands.push(GuestCommand { conn, command: MARK.to_owned(), permission: 0, integrated: false, quiet: true });
        return ControlFlow::Break(());
    }
    if toggled {
        host.hub.toggle_packet(conn, packet.clone());
        return ControlFlow::Break(());
    }
    if host.hub.guests[gi].dead || host.hub.guests[gi].spectator() {
        return ControlFlow::Continue(());
    }
    let mut picked = None;
    let held = match packet {
        S::ContainerClick(p) => click_payload(game, host, gi, p).map(|payload| (payload, S::ContainerClick(p.clone()))),
        S::ContainerButtonClick(p) => button_payload(game, host, gi, p).map(|payload| (payload, S::ContainerButtonClick(p.clone()))),
        S::RenameItem(p) => {
            let anvil = host.hub.guests[gi].menu.is_none();
            let window = find_open(host, conn);
            (anvil && window.is_some()).then(|| (guest_payload(host, gi, json!({ "kind": "rename", "window": window })), S::RenameItem(p.clone())))
        }
        S::SelectTrade(p) => trade_payload(host, gi, p).map(|payload| (payload, S::SelectTrade(p.clone()))),
        S::PickItemFromBlock(p) => {
            let found = game.as_guest(host, conn, |g| {
                if !g.world.loaded(p.pos[0], p.pos[2]) {
                    return None;
                }
                let eye = g.eye_position()?;
                let d2: f64 = (0..3).map(|a| (eye[a] - eye[a].clamp(f64::from(p.pos[a]), f64::from(p.pos[a]) + 1.0)).powi(2)).sum();
                let range = g.reach() + 1.0;
                if d2 >= range * range {
                    return None;
                }
                g.picked_stack(p.pos, g.creative && p.include_data)
            });
            found.flatten().map(|stack| {
                let mut payload = guest_payload(host, gi, json!({ "kind": "pick_block", "pos": p.pos, "include_data": p.include_data }));
                pick_facts(&mut payload, &host.hub.guests[gi], &stack);
                picked = Some(stack);
                (payload, S::PickItemFromBlock(p.clone()))
            })
        }
        S::PickItemFromEntity(p) => {
            let found = game.as_guest(host, conn, |g| g.within_entity_range(p.id).then(|| g.entity_pick_result(p.id)).flatten());
            found.flatten().map(|stack| {
                let mut payload = guest_payload(host, gi, json!({ "kind": "pick_entity", "entity": p.id, "include_data": p.include_data }));
                pick_facts(&mut payload, &host.hub.guests[gi], &stack);
                picked = Some(stack);
                (payload, S::PickItemFromEntity(p.clone()))
            })
        }
        S::PlaceRecipe(p) => recipe_payload(game, host, gi, p).map(|payload| (payload, S::PlaceRecipe(p.clone()))),
        S::SetBeacon(p) => beacon_payload(host, gi, p).map(|payload| (payload, S::SetBeacon(p.clone()))),
        S::SetCreativeModeSlot(p) => creative_payload(host, gi, p).map(|payload| (payload, S::SetCreativeModeSlot(p.clone()))),
        S::Interact(p) => unknown_payload(game, host, gi, p.id, false, p.hand, Some(p.location)).map(|payload| (payload, S::Interact(p.clone()))),
        S::Attack(p) => attack_payload(game, host, gi, conn, p.id).map(|payload| (payload, S::Attack(p.clone()))),
        S::CustomPayload(p) if matches!(p.channel.as_str(), "minecraft:register" | "minecraft:unregister") => {
            let channels: Vec<String> = p.data.split(|byte| *byte == 0).filter(|name| !name.is_empty()).map(|name| String::from_utf8_lossy(name).into_owned()).collect();
            Some((guest_payload(host, gi, json!({ "kind": "channel", "register": p.channel == "minecraft:register", "channels": channels })), S::CustomPayload(p.clone())))
        }
        S::EditBook(p) => book_payload(host, gi, p).map(|payload| (payload, S::EditBook(p.clone()))),
        S::SignUpdate(p) => {
            let sign = matches!(game.world.block_entities.get(&(p.pos[0], p.pos[1], p.pos[2])), Some(BlockEntity::Sign(_)));
            (!sign).then(|| (guest_payload(host, gi, json!({ "kind": "unchecked_sign", "pos": p.pos, "lines": p.lines, "front": p.front })), S::SignUpdate(p.clone())))
        }
        S::CustomClickAction(p) => Some((guest_payload(host, gi, json!({ "kind": "custom_click", "id": p.id, "tag": p.payload.as_ref().map(|tag| crate::game::commands::paper_entities::tag_json(tag)) })), S::CustomClickAction(p.clone()))),
        S::ContainerClose(p) => close_payload(game, host, gi, p).map(|payload| (payload, S::ContainerClose(p.clone()))),
        S::PlayerAction(a) if a.action == 6 => {
            let g = &host.hub.guests[gi];
            let held = g.inventory.get(g.selected).and_then(Option::as_ref);
            Some((guest_payload(host, gi, json!({ "kind": "swap", "stack": stack(held), "off_hand": stack(g.offhand.as_ref()) })), S::PlayerAction(a.clone())))
        }
        S::PlayerAction(a) if a.action == 5 => {
            let in_use = host.play.get(&conn).and_then(|play| USING.get().and_then(|using| using(play)));
            let g = &host.hub.guests[gi];
            in_use.map(|(slot, ticks)| {
                let item = if slot < 36 { g.inventory.get(slot).cloned().flatten() } else { g.offhand.clone() };
                (guest_payload(host, gi, json!({ "kind": "stop_use", "stack": stack(item.as_ref()), "ticks": ticks, "hand": if slot < 36 { "HAND" } else { "OFF_HAND" } })), S::PlayerAction(a.clone()))
            })
        }
        S::SelectBundleItem(p) => {
            let g = &host.hub.guests[gi];
            Some((guest_payload(host, gi, json!({ "kind": "bundle", "slot": p.slot, "index": p.index, "bundle": stack(usize::try_from(p.slot).ok().and_then(|slot| g.menu_slot(slot)).as_ref()) })), S::SelectBundleItem(p.clone())))
        }
        _ => None,
    };
    let Some((payload, packet)) = held else {
        return ControlFlow::Continue(());
    };
    HELD.with(|held| held.borrow_mut().push(Held { conn, packet, payload, picked }));
    host.commands.push(GuestCommand { conn, command: MARK.to_owned(), permission: 0, integrated: false, quiet: true });
    ControlFlow::Break(())
}

/// A packet refused: the client is told where it stands.
fn refused_back(game: &Game<'_>, conn: u32, packet: &S) {
    let Some(g) = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.conn == conn)) else {
        return;
    };
    match packet {
        S::SetCarriedItem(_) => g.send(&play::Clientbound::SetHeldSlot(play::SetHeldSlot { slot: g.selected as i32 })),
        S::PlayerAbilities(_) => g.send(&play::Clientbound::PlayerAbilities(play::PlayerAbilities { flags: g.ability_flags(), flying_speed: 0.05, walking_speed: 0.1 })),
        _ => {}
    }
}

/// What a click leaves to do: `firedInteract` for the item's packet behind it, the block a left click
/// was stopped on, the slot of an item that wasn't used.
fn click_after(game: &mut Game<'_>, conn: u32, on_block: Option<([i32; 3], bool)>, by_item: bool, block: bool, item: bool) {
    let ticks = game.ticks;
    if let Some((pos, true)) = on_block {
        FIRED.set(Some((conn, pos, ticks, item)));
    }
    let state = on_block.map(|(pos, _)| (pos, game.world.block(pos[0], pos[1], pos[2]).0 as i32));
    let Some(host) = game.lan.as_mut() else {
        return;
    };
    if let (Some((pos, state)), Some((_, false)), true) = (state, on_block, block || item) {
        host.hub.send_to(conn, &play::Clientbound::BlockUpdate(play::BlockUpdate { pos, state }));
    }
    if by_item && item && let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn) {
        host.hub.send_inventory(gi);
    }
}

/// `PlayerFailMoveEvent` for each move the hub held back this tick, and what comes of it.
fn fails_check(game: &mut Game<'_>) {
    let rows = std::mem::take(&mut *horizon_client::paper_hub::FAILED_MOVES.lock().unwrap_or_else(|e| e.into_inner()));
    for row in rows {
        let Some(g) = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.uuid == row.uuid)) else {
            continue;
        };
        let conn = g.conn;
        let payload = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension, "reason": row.reason, "from": row.from, "to": row.to, "yaw": row.rot.0, "pitch": row.rot.1 });
        MOVE_WORD.set(None);
        fire(game, "pl_fail_move", payload);
        let (allowed, log) = MOVE_WORD.take().unwrap_or((false, true));
        if log {
            game.lan_log_event(horizon_client::lan::hub::HostEvent::Warn(row.line));
        }
        if row.refused
            && let Some(host) = game.lan.as_mut()
        {
            host.hub.move_answered(conn, allowed, row.back, row.to, row.rot, row.flags);
        }
    }
}

/// `PlayerToggleSneakEvent`, `PlayerToggleSprintEvent` or `PlayerToggleFlightEvent` for a packet the
/// hub sent on because it changes a flag.
fn toggle_payload(host: &LanHost, gi: usize, packet: &S) -> Option<Value> {
    if let S::RecipeBookChangeSettings(p) = packet {
        return Some(guest_payload(host, gi, json!({ "kind": "book_settings", "book": p.book_type, "open": p.open, "filtering": p.filtering })));
    }
    let g = &host.hub.guests[gi];
    let (what, now, before) = match packet {
        S::PlayerInput(p) => ("sneak", p.flags & 32 != 0, g.sneaking),
        S::PlayerCommand(p) => ("sprint", p.action == 1, g.sprinting),
        S::PlayerAbilities(p) => ("flight", p.flags & 2 != 0, g.flying),
        _ => return None,
    };
    (now != before).then(|| guest_payload(host, gi, json!({ "kind": "toggle", "what": what, "now": now })))
}

/// The direction a packet's number stands for.
const FACES: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];

/// `PlayerInteractEvent`'s facts of a click. A block denied from the start is the `cancelledBlock`
/// of `callPlayerInteractEvent`, and an event that starts cancelled.
fn click(host: &LanHost, gi: usize, action: &str, hand: i32, on: Option<([i32; 3], [i32; 3])>, cursor: Option<[f64; 3]>, denied: bool) -> Value {
    let g = &host.hub.guests[gi];
    let held = if hand == 1 { g.offhand.as_ref() } else { g.inventory.get(g.selected).and_then(Option::as_ref) };
    let (pos, face) = (on.map(|on| on.0), on.map_or([0, 0, 1], |on| on.1));
    let word = if denied { "DENY" } else { "ALLOW" };
    let mut payload = guest_payload(host, gi, json!({ "kind": "interact", "action": action, "pos": pos, "face": face, "cursor": cursor, "stack": stack(held), "hand": if hand == 1 { "OFF_HAND" } else { "HAND" }, "use_block": word, "use_item": "DEFAULT" }));
    if denied {
        payload["__answer"] = json!(2);
    }
    payload
}

/// What's heard from a packet whoever sends it: a click, a click on an entity, the held slot.
fn heard_first(game: &mut Game<'_>, host: &mut LanHost, gi: usize, conn: u32, packet: &S) -> Option<Value> {
    if host.hub.guests[gi].dead {
        return None;
    }
    let (min_y, max_y) = (game.world.shape.min_y, game.world.shape.max_y() - 1);
    match packet {
        S::SetCarriedItem(p) => {
            let old = host.hub.guests[gi].selected;
            ((0..9).contains(&p.slot) && usize::try_from(p.slot).ok() != Some(old)).then(|| guest_payload(host, gi, json!({ "kind": "held_slot", "old": old, "new": p.slot })))
        }
        S::UseItemOn(p) => {
            let inside = p.cursor.iter().all(|&c| (f64::from(c) - 0.5).abs() < 1.0000001);
            if !inside || !host.hub.guests[gi].may_reach_block(p.pos) || p.pos[1] < min_y || p.pos[1] > max_y || game.under_spawn_protection(host, gi, p.pos) {
                return None;
            }
            let face = *FACES.get(usize::try_from(p.face).ok()?)?;
            // A spectator's click only counts on what opens a menu.
            let denied = host.hub.guests[gi].spectator() && !game.world.block_entities.contains_key(&(p.pos[0], p.pos[1], p.pos[2]));
            Some(click(host, gi, "RIGHT_CLICK_BLOCK", p.hand, Some((p.pos, face)), Some(p.cursor.map(f64::from)), denied))
        }
        S::PlayerAction(a) if a.action == 0 => {
            if host.hub.guests[gi].spectator() || !host.hub.guests[gi].may_reach_block(a.pos) || game.under_spawn_protection(host, gi, a.pos) || !game.border.contains(f64::from(a.pos[0]), f64::from(a.pos[2])) {
                return None;
            }
            let face = *FACES.get(usize::try_from(i32::from(a.direction)).ok()?)?;
            Some(click(host, gi, "LEFT_CLICK_BLOCK", 0, Some((a.pos, face)), None, false))
        }
        S::UseItem(p) => {
            let g = &mut host.hub.guests[gi];
            (g.yaw, g.pitch) = (p.yaw, p.pitch);
            let Some(hit) = game.as_guest(host, conn, |g| g.pick()).flatten() else {
                return Some(click(host, gi, "RIGHT_CLICK_AIR", p.hand, None, None, true));
            };
            if let Some((who, pos, tick, denied)) = FIRED.get()
                && (who, pos, tick) == (conn, hit.pos, game.ticks)
            {
                // The same click as the block's packet: answered there.
                return denied.then(|| guest_payload(host, gi, json!({ "kind": "interact_again", "use_item": "DENY" })));
            }
            let cursor = [hit.point[0] - f64::from(hit.pos[0]), hit.point[1] - f64::from(hit.pos[1]), hit.point[2] - f64::from(hit.pos[2])];
            Some(click(host, gi, "RIGHT_CLICK_BLOCK", p.hand, Some((hit.pos, hit.face)), Some(cursor), true))
        }
        S::Interact(p) => entity_click(game, host, gi, conn, p),
        S::Punch(_) => {
            let air = game.as_guest(host, conn, |g| g.pick().is_none()) == Some(true);
            let g = &host.hub.guests[gi];
            let held = stack(g.inventory.get(g.selected).and_then(Option::as_ref));
            Some(guest_payload(host, gi, json!({ "kind": "swing", "air": air, "stack": held, "hand": "HAND", "face": [0, 0, 1], "use_block": "DENY", "use_item": "DEFAULT" })))
        }
        _ => None,
    }
}

/// `PlayerInteractEntityEvent`, for an entity the click reaches.
fn entity_click(game: &mut Game<'_>, host: &mut LanHost, gi: usize, conn: u32, p: &play::Interact) -> Option<Value> {
    let target = game.guest_target_box(host, gi, p.id)?;
    if !game.inside_border(&target) || game.as_guest(host, conn, |g| g.within_entity_interaction_range(&target, 3.0)) != Some(true) {
        return None;
    }
    let other = host.hub.guests.iter().any(|g| g.entity_id == p.id);
    let level = host.hub.guests[gi].dimension.clone();
    let kind = if other { json!("minecraft:player") } else { game.native_entity(&json!({ "dimension": level, "id": p.id, "data": false }))["kind"].clone() };
    Some(guest_payload(host, gi, json!({ "kind": "use_entity", "entity": p.id, "type": kind, "hand": if p.hand == 1 { "OFF_HAND" } else { "HAND" }, "at": p.location })))
}

/// The block and the item, as the click running now may use them.
pub(in crate::game) fn block_denied() -> bool {
    USES.get().0
}

pub(in crate::game) fn item_denied() -> bool {
    USES.get().1
}

fn book_payload(host: &LanHost, gi: usize, p: &play::EditBook) -> Option<Value> {
    let g = &host.hub.guests[gi];
    let held = if p.slot == 40 { g.offhand.as_ref() } else { g.inventory.get(usize::try_from(p.slot).ok()?)?.as_ref() };
    Some(guest_payload(host, gi, json!({ "kind": "edit_book", "slot": p.slot, "pages": p.pages, "title": p.title, "previous": stack(held) })))
}

fn close_payload(game: &Game<'_>, host: &mut LanHost, gi: usize, p: &play::ContainerClose) -> Option<Value> {
    let g = &host.hub.guests[gi];
    if p.container == 0 || g.menu.as_ref().is_some_and(|menu| menu.window == p.container && menu.slots == Slots::Native) {
        return None;
    }
    let view = view_of(game, host, gi, p.container)?;
    Some(base("close", &host.hub.guests[gi], p.container, &view))
}

/// What `setNewBookMeta` said past the pages and the title.
struct BookMeta {
    stack: ItemStack,
    author: Option<String>,
    generation: Option<u8>,
    tags: Vec<horizon_world::nbt::Tag>,
}

/// True if the book in `slot` is one the packet can write.
fn book_writable(game: &Game<'_>, conn: u32, slot: i32) -> bool {
    let Some(g) = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.conn == conn)) else {
        return false;
    };
    let held = if slot == 40 { g.offhand.as_ref() } else { usize::try_from(slot).ok().and_then(|slot| g.inventory.get(slot)).and_then(Option::as_ref) };
    held.is_some_and(|held| &*held.item == "minecraft:writable_book")
}

/// `CraftItemStack.setItemMeta` on the book the game just wrote: the plugin's meta, on the pages and
/// the title that went through the game.
fn book_meta_on(game: &mut Game<'_>, conn: u32, slot: i32) {
    let Some(BookMeta { mut stack, author, generation, tags }) = BOOK_META.take() else {
        return;
    };
    let Some(g) = game.lan.as_mut().and_then(|host| host.hub.guests.iter_mut().find(|g| g.conn == conn)) else {
        return;
    };
    let held = if slot == 40 { Some(&mut g.offhand) } else { usize::try_from(slot).ok().and_then(|slot| g.inventory.get_mut(slot)) };
    let Some(book) = held.and_then(Option::as_mut) else {
        return;
    };
    let (mut written, pages) = book.data().map_or((None, Vec::new()), |data| (data.written.clone(), data.writable_pages.clone()));
    if let Some(written) = written.as_mut() {
        written.author = author.unwrap_or_else(|| std::mem::take(&mut written.author));
        written.generation = generation.unwrap_or(written.generation);
        if tags.len() == written.pages.len() {
            written.page_tags = tags;
        }
    }
    stack.item = book.item.clone();
    stack.count = book.count;
    let data = stack.data_mut();
    data.written = written;
    data.writable_pages = pages;
    stack.tidy();
    *book = stack;
    resync(game, conn, 0);
}

/// What `pick_item::pick` would do with `picked`, tried on a copy.
fn pick_facts(payload: &mut Value, g: &horizon_client::lan::guest::Guest, picked: &ItemStack) {
    let (mut slots, mut selected) = (g.inventory[..36].to_vec(), g.selected);
    horizon_client::pick_item::pick(&mut slots, &mut selected, picked, g.creative());
    let source = g.inventory.iter().take(36).position(|held| held.as_ref().is_some_and(|held| held.same_item_same_components(picked)));
    payload["item"] = stack(Some(picked));
    payload["target"] = json!(selected);
    payload["source"] = json!(source.map_or(-1, |slot| slot as i32));
}

/// `tryPickItem` with the slots `PlayerPickItemEvent` left: `Inventory.pickSlot`, or
/// `addAndPickItem` for a creative player when it comes from nowhere.
fn pick_to(game: &mut Game<'_>, conn: u32, picked: ItemStack, source: i32, target: i32) {
    let Some(host) = game.lan.as_mut() else {
        return;
    };
    let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn) else {
        return;
    };
    let g = &mut host.hub.guests[gi];
    let target = target.clamp(0, 8) as usize;
    if let Ok(source) = usize::try_from(source) {
        g.selected = target;
        if source >= 9 || target >= 9 {
            g.inventory.swap(target, source.min(35));
        }
    } else if g.creative() {
        g.selected = target;
        if g.inventory[target].is_some()
            && let Some(free) = g.inventory.iter().take(36).position(Option::is_none)
        {
            g.inventory[free] = g.inventory[target].take();
        }
        g.inventory[target] = Some(picked);
    }
    let slot = g.selected as i32;
    host.hub.send_inventory(gi);
    host.hub.send_to(conn, &play::Clientbound::SetHeldSlot(play::SetHeldSlot { slot }));
}

/// `extra` with whose it is.
fn guest_payload(host: &LanHost, gi: usize, mut extra: Value) -> Value {
    let g = &host.hub.guests[gi];
    extra["uuid"] = json!(format!("{:032x}", g.uuid));
    extra["name"] = json!(g.name);
    extra["level"] = json!(g.dimension);
    extra
}

fn trade_payload(host: &mut LanHost, gi: usize, p: &play::SelectTrade) -> Option<Value> {
    let conn = host.hub.guests[gi].conn;
    let trade = host.play.get(&conn)?.merchant.as_ref()?;
    let (window, mob) = (trade.window, trade.merchant.mob);
    let items = TRADES.get().and_then(|find| find(host, conn).map(|(_, slots)| stacks(slots.iter().map(Option::as_ref)))).unwrap_or(Value::Null);
    let mut payload = guest_payload(host, gi, json!({ "kind": "trade_select", "type": "MERCHANT", "window": window, "size": 3, "index": p.item, "entity": mob, "items": items }));
    payload["cursor"] = stack(host.hub.guests[gi].carried.as_ref());
    Some(payload)
}

fn recipe_payload(game: &Game<'_>, host: &LanHost, gi: usize, p: &horizon_protocol::packets::recipes::PlaceRecipe) -> Option<Value> {
    let id = game.res.stations().recipe_index.recipe_of_display(p.recipe)?;
    Some(guest_payload(host, gi, json!({ "kind": "recipe_click", "recipe": id, "shift": p.use_max, "window": p.container })))
}

fn beacon_payload(host: &mut LanHost, gi: usize, p: &play::SetBeacon) -> Option<Value> {
    let conn = host.hub.guests[gi].conn;
    let window = find_open(host, conn)?;
    let (_, pos, menu) = find_menu(host, conn, window)?;
    if menu.kind() != Kind::Beacon {
        return None;
    }
    let paid = menu.slot(0).is_some();
    let name = |id: Option<i32>| id.and_then(|id| host.hub.registry_name("minecraft:mob_effect", id).map(str::to_owned));
    let (primary, secondary) = (name(p.primary), name(p.secondary));
    Some(guest_payload(host, gi, json!({ "kind": "beacon", "window": window, "pos": pos, "primary": primary, "secondary": secondary, "paid": paid })))
}

fn creative_payload(host: &mut LanHost, gi: usize, p: &play::SetCreativeModeSlot) -> Option<Value> {
    let item = p.item.as_ref().map(|stack| host.hub.stack_from_network(stack));
    let g = &host.hub.guests[gi];
    let view = View { kind: "CREATIVE", own: (0..46).map(|slot| g.menu_slot(slot)).collect(), pos: None, recipe: None };
    let mut payload = base("creative", g, 0, &view);
    payload["slot"] = json!(p.slot);
    payload["cursor"] = item.as_ref().map_or(Value::Null, stack_value);
    Some(payload)
}

/// An attack on anything: what `PrePlayerAttackEntityEvent` and the cooldown event read, or a fake entity's.
fn attack_payload(game: &mut Game<'_>, host: &mut LanHost, gi: usize, conn: u32, id: i32) -> Option<Value> {
    if let Some(unknown) = unknown_payload(game, host, gi, id, true, 0, None) {
        return Some(unknown);
    }
    let strength = game.as_guest(host, conn, |g| g.attack_strength_scale()).unwrap_or(1.0);
    let target = if host.hub.guests.iter().any(|g| g.entity_id == id) { "minecraft:player".to_owned() } else {
        let level = host.hub.guests[gi].dimension.clone();
        game.native_entity(&json!({ "dimension": level, "id": id, "data": false }))["kind"].as_str().unwrap_or_default().to_owned()
    };
    Some(guest_payload(host, gi, json!({ "kind": "attack", "entity": id, "target": target, "strength": strength })))
}

/// `PlayerUseUnknownEntityEvent`: an id nothing in the world has, as a plugin's fake entity's.
fn unknown_payload(game: &mut Game<'_>, host: &LanHost, gi: usize, id: i32, attack: bool, hand: i32, at: Option<[f64; 3]>) -> Option<Value> {
    if host.hub.guests.iter().any(|g| g.entity_id == id) {
        return None;
    }
    let level = host.hub.guests[gi].dimension.clone();
    if !game.native_entity(&json!({ "dimension": level, "id": id, "data": false })).is_null() {
        return None;
    }
    Some(guest_payload(host, gi, json!({ "kind": "unknown_entity", "entity": id, "attack": attack, "hand": if hand == 1 { "OFF_HAND" } else { "HAND" }, "at": at })))
}

/// A menu button: which offer, recipe or pattern. Each is a held event of its own.
fn button_payload(game: &Game<'_>, host: &mut LanHost, gi: usize, p: &play::ContainerButtonClick) -> Option<Value> {
    let conn = host.hub.guests[gi].conn;
    if let Some(find) = LECTERNS.get()
        && let Some((open, key)) = find(host, conn)
        && open == p.container
    {
        let (book, page) = match game.world.block_entities.get(&key) {
            Some(BlockEntity::Lectern(lectern)) => (lectern.book.clone(), lectern.page),
            _ => return None,
        };
        let kind = if p.button == 3 { "lectern_take" } else { "lectern_page" };
        return Some(guest_payload(host, gi, json!({ "kind": kind, "button": p.button, "window": p.container, "type": "LECTERN", "pos": [key.0, key.1, key.2], "book": stack(book.as_ref()), "page": page })));
    }
    let (_, pos, menu) = find_menu(host, conn, p.container)?;
    let kind = menu.kind();
    let button = usize::try_from(p.button).ok();
    let mut payload = json!({ "kind": "button", "type": type_of_kind(kind), "button": p.button, "window": p.container, "pos": pos });
    match kind {
        Kind::Enchanting => {
            let i = button.filter(|i| *i < 3)?;
            let item = menu.slot(0).cloned()?;
            let cost = menu.data[i];
            let lapis = menu.slot(1).map_or(0, |lapis| lapis.count as i32);
            let g = &host.hub.guests.iter().find(|g| g.conn == conn)?;
            let need = i as i32 + 1;
            // EnchantmentMenu.clickMenuButton's own checks: what they refuse never gets this far.
            if cost <= 0 || (!g.creative() && (lapis < need || g.xp_level < need || g.xp_level < cost)) {
                return None;
            }
            let env = game.menu_env();
            let done = horizon_client::stations::apply_enchantment(env.enchantments, env.items, g.xp_seed, i, cost, &item)?;
            let offers = horizon_client::stations::enchant_offers(env.enchantments, env.items, g.xp_seed, game.count_bookshelves(pos), Some(&item));
            payload["cost"] = json!(cost);
            payload["item"] = crate::game::paper_query::stack_json(&item);
            payload["done"] = crate::game::paper_query::stack_json(&done);
            payload["hint"] = json!(offers.clues[i].as_ref().map(|(enchantment, level)| json!({ "enchantment": &**enchantment, "level": level })));
            payload["levels"] = json!(g.xp_level);
        }
        Kind::Stonecutter => {
            let recipe = menu.stone.recipes.get(button?).copied()?;
            let env = game.menu_env();
            let id = env.station_recipes.stonecutting[recipe].id.clone();
            payload["recipe"] = json!(id);
            payload["recipes"] = json!(menu.stone.recipes.iter().map(|&at| env.station_recipes.stonecutting[at].id.clone()).collect::<Vec<_>>());
            payload["input"] = stack(menu.slot(0));
        }
        Kind::Loom => {
            payload["pattern"] = json!(menu.loom_patterns.get(button?).map(|pattern| &**pattern));
            payload["patterns"] = json!(menu.loom_patterns.iter().map(|pattern| &**pattern).collect::<Vec<_>>());
            payload["items"] = stacks((0..3).map(|slot| menu.slot(slot)));
        }
        Kind::Beacon => return None,
        _ => return None,
    }
    let g = host.hub.guests.iter().find(|g| g.conn == conn)?;
    payload["uuid"] = json!(format!("{:032x}", g.uuid));
    payload["name"] = json!(g.name);
    payload["level"] = json!(g.dimension);
    Some(payload)
}

fn pass(game: &mut Game<'_>, conn: u32, packet: S) {
    let Some(mut host) = game.lan.take() else {
        return;
    };
    PASSING.set(true);
    game.lan_guest_packet(&mut host, conn, packet);
    PASSING.set(false);
    game.lan = Some(host);
}

/// Takes back what the client did in window `window`: the whole of it is sent again.
fn resync(game: &mut Game<'_>, conn: u32, window: i32) {
    let Some(host) = game.lan.as_mut() else {
        return;
    };
    let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn) else {
        return;
    };
    if window == 0 {
        host.hub.send_inventory(gi);
        return;
    }
    if host.hub.guests[gi].menu.as_ref().is_some_and(|menu| menu.window == window) {
        host.hub.resend_menu(conn, window, &game.world);
        return;
    }
    // The menus the game runs itself: a click that does nothing, on a state the client can't have,
    // gets the whole menu back from their own code.
    let click = play::ContainerClick { container: window, state: -1, slot: -1, button: 2, input: 3, changed: Vec::new(), carried: None };
    pass(game, conn, S::ContainerClick(click));
}

/// The grid of whichever crafting menu is open, and what a listener may change of it.
fn prepare_view(game: &Game<'_>, host: &mut LanHost, conn: u32) -> Option<(i32, View)> {
    let gi = host.hub.guests.iter().position(|g| g.conn == conn)?;
    let window = find_open(host, conn)?;
    Some((window, view_of(game, host, gi, window)?))
}

/// `PrepareItemCraftEvent` for a grid something other than its menu touched, changed or not.
pub(in crate::game) fn grid_touched(game: &mut Game<'_>, conn: u32) {
    PREPARED.with(|seen| seen.borrow_mut().retain(|(c, _)| *c != conn));
    prepare_grid(game, conn);
}

/// `PrepareItemCraftEvent` for the player's own grid, which no menu of ours holds.
fn prepare_grid(game: &mut Game<'_>, conn: u32) {
    let Some(host) = game.lan.as_ref() else {
        return;
    };
    let Some(g) = host.hub.guests.iter().find(|g| g.conn == conn) else {
        return;
    };
    let grid = g.craft_grid.clone();
    let own = (0..46).map(|slot| g.menu_slot(slot)).collect();
    let view = View { kind: "CRAFTING", own, pos: None, recipe: game.menu_env().recipes.find(&grid, 2).map(|(id, _)| format!("minecraft:{id}")) };
    let key = json!([stacks(grid.iter().map(Option::as_ref))]);
    let changed = PREPARED.with(|seen| {
        let mut seen = seen.borrow_mut();
        match seen.iter_mut().find(|(c, _)| *c == conn) {
            Some((_, last)) if *last == key => false,
            Some((_, last)) => {
                *last = key;
                true
            }
            None => {
                seen.push((conn, key));
                true
            }
        }
    });
    if !changed {
        return;
    }
    let mut payload = base("prepare", g, 0, &view);
    payload["recipe"] = json!(view.recipe);
    paper_bridge::on_refusable(game, "pl_packet", payload);
}

fn find_open(host: &mut LanHost, conn: u32) -> Option<i32> {
    TABLES.get().and_then(|find| find(host, conn).map(|(window, ..)| window)).or_else(|| STATIONS.get().and_then(|find| find(host, conn).map(|(window, ..)| window)))
}

/// `Prepare...Event`, once a menu's inputs have changed: the listener's result goes in at once.
fn prepare(game: &mut Game<'_>, conn: u32) {
    let Some(mut host) = game.lan.take() else {
        return;
    };
    let extra = find_open(&mut host, conn).and_then(|window| {
        let (_, pos, menu) = find_menu(&mut host, conn, window)?;
        let name = menu.anvil.name.clone();
        let (costs, clues) = (menu.data.get(0..3).map(<[i32]>::to_vec), menu.data.get(7..10).map(<[i32]>::to_vec));
        let cost = menu.data.first().copied();
        Some((pos, name, costs, clues, cost))
    });
    let view = prepare_view(game, &mut host, conn);
    let guest = host.hub.guests.iter().find(|g| g.conn == conn).map(|g| (g.uuid, g.name.clone(), g.dimension.clone(), g.xp_seed));
    let mut payload = None;
    if let (Some((window, view)), Some((uuid, name, level, seed)), Some((pos, rename, costs, levels, cost))) = (view, guest, extra) {
        let items = stacks(view.own.iter().map(Option::as_ref));
        let changed = PREPARED.with(|seen| {
            let mut seen = seen.borrow_mut();
            let key = json!([items, rename]);
            match seen.iter_mut().find(|(c, _)| *c == conn) {
                Some((_, last)) if *last == key => false,
                Some((_, last)) => {
                    *last = key;
                    true
                }
                None => {
                    seen.push((conn, key));
                    true
                }
            }
        });
        if changed {
            let mut value = json!({
                "kind": "prepare", "uuid": format!("{uuid:032x}"), "name": name, "window": window, "type": view.kind,
                "size": view.own.len(), "items": items, "pos": pos, "rename": rename, "recipe": view.recipe, "level": level,
                "repair_cost": cost,
            });
            if view.kind == "ENCHANTING" {
                let env = game.menu_env();
                let shelves = game.count_bookshelves(pos);
                let offers = horizon_client::stations::enchant_offers(env.enchantments, env.items, seed, shelves, view.own.first().and_then(Option::as_ref));
                value["bookshelves"] = json!(shelves);
                value["costs"] = json!(costs);
                value["clue_levels"] = json!(levels);
                value["clues"] = json!(offers.clues.iter().map(|clue| clue.as_ref().map(|(enchantment, level)| json!([&**enchantment, level]))).collect::<Vec<_>>());
            }
            payload = Some(value);
        }
    }
    game.lan = Some(host);
    if let Some(payload) = payload {
        let window = payload["window"].as_i64().unwrap_or(0) as i32;
        WROTE.set(false);
        let enchanting = payload["type"] == "ENCHANTING";
        // PrepareItemEnchantEvent: a cancel leaves nothing on offer.
        if paper_bridge::on_refusable(game, "pl_packet", payload) && enchanting {
            if let Some(mut host) = game.lan.take() {
                if let Some((_, _, menu)) = find_menu(&mut host, conn, window) {
                    menu.data[0..3].fill(0);
                    menu.data[4..10].fill(-1);
                }
                game.lan = Some(host);
            }
            WROTE.set(true);
        }
        if WROTE.take() {
            resync(game, conn, window);
        }
    }
}

/// The head of `run_guest_commands`, the hub back in `game.lan`: every held packet, in order.
pub(in crate::game) fn run_held(game: &mut Game<'_>) {
    if let Some(host) = game.lan.as_mut() {
        host.commands.retain(|command| command.command != MARK);
    }
    for later in LATER.with(|later| std::mem::take(&mut *later.borrow_mut())) {
        open_now(game, later);
    }
    let held = HELD.with(|held| std::mem::take(&mut *held.borrow_mut()));
    for Held { conn, packet, payload, picked } in held {
        if payload["kind"] == "respawn_ask" {
            respawn_ask(game, conn);
            pass(game, conn, packet);
            continue;
        }
        let window = payload["window"].as_i64().unwrap_or(0) as i32;
        let menu = matches!(payload["kind"].as_str(), Some("click" | "drag" | "rename" | "button"));
        let creative = payload["kind"] == "creative";
        // A click goes through whatever's said of it: it has a sequence to answer and blocks to send
        // back, and `USES` says what of it may run.
        let clicked = matches!(payload["kind"].as_str(), Some("interact" | "interact_again")).then(|| (payload["use_block"] == "DENY", payload["use_item"] == "DENY"));
        USE_WORD.set(None);
        let refused = paper_bridge::on_refusable(game, "pl_packet", payload) && clicked.is_none();
        let too_dear = TOO_DEAR.take();
        if refused || too_dear {
            AFTER.with(|after| after.borrow_mut().clear());
            BOOK.take();
            BOOK_META.take();
            if menu || creative {
                resync(game, conn, window);
            }
            refused_back(game, conn, &packet);
            continue;
        }
        if matches!(packet, S::PlayerInput(_) | S::PlayerAbilities(_) | S::RecipeBookChangeSettings(_)) || matches!(&packet, S::PlayerCommand(p) if matches!(p.action, 1 | 2)) {
            if let Some(host) = game.lan.as_mut() {
                host.hub.toggle_packet(conn, packet);
            }
            continue;
        }
        let clicked = clicked.map(|said| USE_WORD.take().unwrap_or(said));
        let packet = match (packet, BOOK.take()) {
            (S::EditBook(mut book), Some((pages, title))) => {
                book.pages = pages;
                book.title = title;
                S::EditBook(book)
            }
            (packet, _) => packet,
        };
        if let (true, S::ContainerClick(click), Some(result)) = (window == 0, &packet, grid_result(game, conn))
            && click.slot == 0
        {
            take_grid_result(game, conn, click, result);
            resync(game, conn, 0);
            prepare_grid(game, conn);
            continue;
        }
        if let (S::PlayerAction(_), Some((main, off))) = (&packet, HANDS.take()) {
            if let Some(host) = game.lan.as_mut()
                && let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn)
            {
                let g = &mut host.hub.guests[gi];
                let selected = g.selected;
                g.inventory[selected] = main;
                g.offhand = off;
                host.hub.send_inventory(gi);
            }
            continue;
        }
        if let (Some((source, target)), Some(picked)) = (PICK_TO.take(), picked) {
            pick_to(game, conn, picked, source, target);
            continue;
        }
        let packet = rewritten(game, packet);
        let keep_ticks = KEEP_ATTACK.take().then(|| {
            let (ticks, host) = (ATTACK_TICKS.get()?, game.lan.as_mut()?);
            Some((ticks, *ticks(host.play.entry(conn).or_default())))
        }).flatten();
        let keep_payment = KEEP_PAYMENT.take().then(|| game.lan.as_mut().and_then(|host| find_menu(host, conn, window)).and_then(|(_, _, menu)| menu.slot(0).cloned())).flatten();
        let trade = TRADE.take();
        if let Some((offer, reward, _)) = &trade {
            trade_before(game, conn, offer.as_ref(), *reward);
        }
        let new_page = NEW_PAGE.take();
        let book_slot = if let S::EditBook(book) = &packet { Some(book.slot) } else { None };
        if book_slot.is_some_and(|slot| !book_writable(game, conn, slot)) {
            BOOK_META.take();
        }
        let on_block = match &packet {
            S::UseItemOn(p) => Some((p.pos, true)),
            S::PlayerAction(a) if a.action == 0 => Some((a.pos, false)),
            _ => None,
        };
        let by_item = matches!(packet, S::UseItem(_));
        if let Some(uses) = clicked {
            USES.set(uses);
        }
        pass(game, conn, packet);
        USES.set((false, false));
        if let Some((block, item)) = clicked {
            click_after(game, conn, on_block, by_item, block, item);
        }
        if let Some(slot) = book_slot {
            book_meta_on(game, conn, slot);
        }
        if let Some((_, _, false)) = &trade {
            trade_after(game, conn);
        }
        if let (Some(page), Some((_, _, Some(key)))) = (new_page, lectern_of(game, conn))
            && let Some(BlockEntity::Lectern(lectern)) = game.world.block_entities.get_mut(&key)
        {
            lectern.page = page;
            resync(game, conn, window);
        }
        if let (Some((ticks, old)), Some(host)) = (keep_ticks, game.lan.as_mut()) {
            *ticks(host.play.entry(conn).or_default()) = old;
        }
        if let Some(stack) = keep_payment {
            write_slot(game, conn, window, 0, Some(stack));
            resync(game, conn, window);
        }
        let after = AFTER.with(|after| std::mem::take(&mut *after.borrow_mut()));
        if !after.is_empty() {
            for (slot, stack) in after {
                write_slot(game, conn, window, slot, stack);
            }
            resync(game, conn, window);
        }
        if menu && window == 0 {
            // The game has worked the grid's own result out again.
            if let Some(result) = grid_result(game, conn)
                && let Some(host) = game.lan.as_mut()
                && let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn)
            {
                host.hub.guests[gi].craft_result = Some(result);
                host.hub.send_inventory(gi);
            }
            prepare_grid(game, conn);
        } else if menu {
            prepare(game, conn);
        }
    }
}

/// A result a listener left in the 2x2 grid, while the grid is still what it was for.
fn grid_result(game: &Game<'_>, conn: u32) -> Option<ItemStack> {
    let grid = game.lan.as_ref()?.hub.guest(conn)?.craft_grid.clone();
    GRID_RESULTS.with(|results| results.borrow().iter().find(|(c, for_grid, _)| *c == conn && *for_grid == grid).map(|(.., stack)| stack.clone()))
}

/// A click on such a result. The game works its own out again, so taking this one is done here, as
/// `ResultSlot` does: to the cursor or the inventory, with one of each ingredient gone.
fn take_grid_result(game: &mut Game<'_>, conn: u32, p: &play::ContainerClick, result: ItemStack) {
    let Some(mut host) = game.lan.take() else {
        return;
    };
    if let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn) {
        let room = host.hub.guests[gi].carried.as_ref().is_none_or(|held| held.same_item_same_components(&result) && held.count + result.count <= max_stack(game, Some(held)));
        let taken = match p.input {
            0 if room => {
                let g = &mut host.hub.guests[gi];
                match g.carried.as_mut() {
                    Some(held) => held.count += result.count,
                    None => g.carried = Some(result),
                }
                true
            }
            1 => {
                game.lan_place_back(&mut host, gi, vec![result]);
                true
            }
            _ => false,
        };
        if taken {
            GRID_RESULTS.with(|results| results.borrow_mut().retain(|(c, ..)| *c != conn));
            let g = &mut host.hub.guests[gi];
            for slot in &mut g.craft_grid {
                if let Some(stack) = slot {
                    stack.count -= 1;
                    if stack.count == 0 {
                        *slot = None;
                    }
                }
            }
            g.craft_result = None;
        }
    }
    game.lan = Some(host);
}

/// A held packet with what a listener put in its place.
fn rewritten(game: &Game<'_>, packet: S) -> S {
    match packet {
        S::ContainerButtonClick(mut p) => {
            if let Some(button) = NEW_BUTTON.take() {
                p.button = button;
            }
            S::ContainerButtonClick(p)
        }
        S::PlaceRecipe(mut p) => {
            if let Some((recipe, shift)) = NEW_RECIPE.take() {
                p.recipe = recipe.unwrap_or(p.recipe);
                p.use_max = shift.unwrap_or(p.use_max);
            }
            S::PlaceRecipe(p)
        }
        S::SetBeacon(mut p) => {
            if let Some((primary, secondary)) = NEW_BEACON.take() {
                p.primary = primary.or(p.primary);
                p.secondary = secondary.or(p.secondary);
            }
            let _ = game;
            S::SetBeacon(p)
        }
        packet => packet,
    }
}

fn lectern_of(game: &Game<'_>, conn: u32) -> Option<(i32, (), Option<(i32, i32, i32)>)> {
    let (find, host) = (LECTERNS.get()?, game.lan.as_ref()?);
    let (window, key) = find(host, conn)?;
    Some((window, (), Some(key)))
}

/// The offer under the result, as a listener reads it: `MerchantRecipe`'s JSON.
fn offer_value(offer: &horizon_client::merchant::MerchantOffer) -> Value {
    let cost = |cost: &horizon_client::merchant::ItemCost| stack_value(&cost.stack);
    json!({ "sell": stack_value(&offer.result), "buy": cost(&offer.base_cost_a), "buyB": offer.cost_b.as_ref().map(cost), "uses": offer.uses, "maxUses": offer.max_uses, "rewardExp": offer.reward_exp, "xp": offer.xp, "priceMultiplier": offer.price_multiplier, "specialPrice": offer.special_price_diff, "demand": offer.demand })
}

fn active_offer<'a>(game: &'a mut Game<'_>, conn: u32) -> Option<&'a mut horizon_client::merchant::MerchantOffer> {
    let host = game.lan.as_mut()?;
    let session = &mut host.play.get_mut(&conn)?.merchant.as_mut()?.merchant.session;
    let at = session.active.unwrap_or(session.selection_hint);
    session.offers.get_mut(at)
}

/// `PlayerPurchaseEvent.setTrade` and `setRewardExp`, on the offer about to be bought.
fn trade_before(game: &mut Game<'_>, conn: u32, offer: Option<&Value>, reward: Option<bool>) {
    let result = offer.and_then(|offer| stack_of(&offer["sell"]));
    let uses = offer.and_then(|offer| offer["uses"].as_i64());
    let max = offer.and_then(|offer| offer["maxUses"].as_i64());
    let cost = offer.and_then(|offer| stack_of(&offer["buy"]));
    let second = offer.map(|offer| stack_of(&offer["buyB"]));
    if let Some(active) = active_offer(game, conn) {
        if let Some(result) = result {
            active.result = result;
        }
        if let Some(cost) = cost {
            active.base_cost_a = horizon_client::merchant::ItemCost { stack: cost };
        }
        if let Some(second) = second {
            active.cost_b = second.map(|stack| horizon_client::merchant::ItemCost { stack });
        }
        if let Some(uses) = uses {
            active.uses = uses as i32;
        }
        if let Some(max) = max {
            active.max_uses = max as i32;
        }
        if let Some(reward) = reward {
            active.reward_exp = reward;
        }
    }
}

/// `PlayerPurchaseEvent.setIncreaseTradeUses(false)`: the use the game just counted goes back.
fn trade_after(game: &mut Game<'_>, conn: u32) {
    if let Some(active) = active_offer(game, conn) {
        active.uses = (active.uses - 1).max(0);
    }
}

/// Puts `item` in a raw slot of window `window`; -1 is the cursor.
fn write_slot(game: &mut Game<'_>, conn: u32, window: i32, slot: i32, item: Option<ItemStack>) -> bool {
    let Some(mut host) = game.lan.take() else {
        return false;
    };
    let done = write_in(game, &mut host, conn, window, slot, item);
    game.lan = Some(host);
    WROTE.set(WROTE.get() || done);
    done
}

fn write_in(game: &mut Game<'_>, host: &mut LanHost, conn: u32, window: i32, slot: i32, item: Option<ItemStack>) -> bool {
    let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn) else {
        return false;
    };
    if slot == -1 {
        host.hub.guests[gi].carried = item;
        return true;
    }
    let Ok(raw) = usize::try_from(slot) else {
        return false;
    };
    let player = |g: &mut horizon_client::lan::guest::Guest, row: usize, item| {
        if let Some(held) = g.inventory.get_mut(if row < 27 { 9 + row } else { row - 27 }) {
            *held = item;
        }
    };
    if window == 0 {
        if raw == 0 {
            let grid = host.hub.guests[gi].craft_grid.clone();
            GRID_RESULTS.with(|results| {
                let mut results = results.borrow_mut();
                results.retain(|(c, ..)| *c != conn);
                if let Some(item) = &item {
                    results.push((conn, grid, item.clone()));
                }
            });
        }
        host.hub.guests[gi].set_menu_slot(raw, item);
        return true;
    }
    if let Some((slots, size)) = host.hub.guests[gi].menu.as_ref().filter(|menu| menu.window == window).map(|menu| (menu.slots, menu.size)) {
        if raw >= size {
            player(&mut host.hub.guests[gi], raw - size, item);
            return true;
        }
        let Some(mut all) = slots.read(&game.world, &host.hub.guests[gi]) else {
            return false;
        };
        if let Some(place) = all.get_mut(raw) {
            *place = item;
        }
        slots.write(&mut game.world, &mut host.hub.guests[gi], all);
        if let Slots::Block(key) | Slots::Double(key, _) = slots {
            game.container_changed(key);
        }
        return true;
    }
    if let Some((_, _, menu)) = find_menu(host, conn, window) {
        let own = menu.slot_count().saturating_sub(36);
        if raw < own {
            menu.set_slot(&game.menu_env(), raw, item);
            return true;
        }
        player(&mut host.hub.guests[gi], raw - own, item);
        return true;
    }
    if let Some(find) = TRADES.get()
        && let Some((open, slots)) = find(host, conn)
        && open == window
    {
        if raw < 3 {
            slots[raw] = item;
        } else {
            player(&mut host.hub.guests[gi], raw - 3, item);
        }
        return true;
    }
    if let Some((find, build, store)) = MOUNTS.get()
        && let Some((open, mob, columns)) = find(host, conn)
        && open == window
        && let Some(index) = game.mobs.iter().position(|m| m.id == mob)
    {
        let mut menu = build(game, index, columns);
        let own = menu.slot_count().saturating_sub(36);
        if raw >= own {
            player(&mut host.hub.guests[gi], raw - own, item);
            return true;
        }
        menu.set_slot(&game.menu_env(), raw, item);
        store(game, index, columns, &menu);
        return true;
    }
    false
}

fn conn_of(game: &Game<'_>, query: &Value) -> Option<u32> {
    let uuid = u128::from_str_radix(query["uuid"].as_str()?, 16).ok()?;
    game.lan.as_ref()?.hub.guests.iter().find(|g| g.uuid == uuid).map(|g| g.conn)
}

/// `lan_open_crafting`, `lan_open_station` and `lan_open_merchant` say how to reach what they open.
pub(in crate::game) fn tables_are(find: FindMenu) {
    TABLES.set(Some(find));
}

pub(in crate::game) fn stations_are(find: FindMenu) {
    STATIONS.set(Some(find));
}

pub(in crate::game) fn trades_are(find: FindTrade) {
    TRADES.set(Some(find));
}

pub(in crate::game) fn lecterns_are(find: FindLectern) {
    LECTERNS.set(Some(find));
}

pub(in crate::game) fn mounts_are(find: FindMount, build: MountMenu, store: MountStore) {
    MOUNTS.set(Some((find, build, store)));
}

/// `None` for an op that isn't this family's.
pub(in crate::game) fn query(game: &mut Game<'_>, op: &str, query: &Value) -> Option<Value> {
    Some(match op {
        // `PlayerGameConnection.reenterConfiguration`: `reconfigure_turn` sends them there at the end of the tick.
        "ev_player_reconfigure" => {
            let wanted = query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
            if let Some(uuid) = wanted {
                RECONFIGURES.with(|all| all.borrow_mut().push(uuid));
            }
            json!(wanted.is_some())
        }
        // A slot of what's open: at once, or with `after` once the packet has gone through.
        "ev_player_slot_set" => {
            let (Some(conn), Some(slot)) = (conn_of(game, query), query["slot"].as_i64()) else {
                return Some(json!(false));
            };
            let item = stack_of(&query["stack"]);
            if query["after"].as_bool() == Some(true) {
                AFTER.with(|after| after.borrow_mut().push((slot as i32, item)));
                return Some(json!(true));
            }
            json!(write_slot(game, conn, query["window"].as_i64().unwrap_or(0) as i32, slot as i32, item))
        }
        "ev_player_exp" => {
            EXP.set(query["amount"].as_u64().map(|amount| amount as u32));
            json!(true)
        }
        "ev_player_book" => {
            let pages = query["pages"].as_array().into_iter().flatten().filter_map(|page| page.as_str().map(str::to_owned)).collect();
            BOOK.set(Some((pages, query["title"].as_str().map(str::to_owned))));
            json!(true)
        }
        "ev_player_burn" => {
            let (burn, cook, keep) = BURN.get();
            let number = |key: &str| query[key].as_i64().map(|value| value as i32);
            BURN.set((number("burn").or(burn), number("cook").or(cook), keep || query["keep"] == true));
            json!(true)
        }
        "ev_player_acting" => acting(game),
        "ev_player_move_to" => {
            let at = |i: usize| query["to"][i].as_f64().unwrap_or(0.0);
            MOVE_TO.set(Some([at(0), at(1), at(2), at(3), at(4)]));
            json!(true)
        }
        "ev_player_tp_to" => {
            let at = |i: usize| query["to"][i].as_f64().unwrap_or(0.0);
            TP_TO.set(Some([at(0), at(1), at(2), at(3), at(4)]));
            TP_LEVEL.with(|level| *level.borrow_mut() = query["level"].as_str().map(str::to_owned));
            json!(true)
        }
        "ev_player_award_line" => {
            AWARD_LINE.set(Some(query["line"].as_str().map(str::to_owned)));
            json!(true)
        }
        "ev_player_map" => {
            NEW_MAP.set(stack_of(&query["stack"]));
            json!(true)
        }
        "ev_player_velocity" => {
            let at = |i: usize| query["velocity"][i].as_f64().unwrap_or(0.0);
            NEW_VELOCITY.set(Some([at(0), at(1), at(2)]));
            json!(true)
        }
        "ev_player_name" => {
            NAME.set(Some((query["new_name"].as_str().map(str::to_owned), query["persistent"].as_bool(), query["entity"].as_u64().map(|id| id as u32))));
            json!(true)
        }
        "ev_player_book_in" => {
            NEW_BOOK.set(stack_of(&query["stack"]));
            json!(true)
        }
        "ev_player_frame" => {
            NEW_FRAME.set(Some(stack_of(&query["stack"])));
            json!(true)
        }
        "ev_player_left" => {
            let wanted = query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
            LEFT.with(|left| left.borrow().iter().find(|(uuid, _)| Some(*uuid) == wanted).map_or(Value::Null, |(_, reason)| json!(reason)))
        }
        "ev_player_equip" => json!(game.res.items().get(query["item"].as_str().unwrap_or_default()).and_then(|item| item.equip_slot.clone())),
        "ev_player_equip_items" => {
            let wanted = query["slot"].as_str().unwrap_or_default();
            Value::Array(game.res.items().items.iter().filter(|(_, item)| item.equip_slot.as_deref() == Some(wanted)).map(|(id, _)| json!(&**id)).collect())
        }
        // Who has loaded in since last asked, and whether their client said so or time ran out.
        "ev_player_loaded" => {
            let Some(host) = game.lan.as_ref() else {
                return Some(Value::Null);
            };
            let fresh: Vec<Value> = LOADED.with(|loaded| {
                let mut loaded = loaded.borrow_mut();
                loaded.retain(|uuid| host.hub.guests.iter().any(|g| g.uuid == *uuid));
                let mut fresh = Vec::new();
                for g in host.hub.guests.iter().filter(|g| g.placed && g.client_loaded()) {
                    if !loaded.contains(&g.uuid) {
                        loaded.push(g.uuid);
                        fresh.push(json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "timeout": !g.loaded }));
                    }
                }
                fresh
            });
            Value::Array(fresh)
        }
        // What's changed of each guest's respawn point since last asked.
        "ev_player_fish_exp" => {
            let exp = query["exp"].as_u64().unwrap_or(0) as u32;
            FISH_EXP.set(Some((exp, game.ticks)));
            json!(true)
        }
        "ev_player_mend" => {
            MEND.set(Some((query["repair"].as_u64().unwrap_or(0) as u32, query["used"].as_u64().unwrap_or(0) as u32)));
            json!(true)
        }
        "ev_player_orb_cooldown" => {
            ORB_COOLDOWN.set(query["ticks"].as_u64().map(|ticks| ticks as u32));
            json!(true)
        }
        "ev_player_exp_cooldown" => {
            let ticks = query["ticks"].as_i64().unwrap_or(0);
            let level = game.dim.id().to_owned();
            ORB_COOLDOWN.set(None);
            fire(game, "pl_orb", json!({ "kind": "cooldown", "uuid": query["uuid"], "name": query["name"], "level": level, "new": ticks, "reason": "PLUGIN" }));
            json!(ORB_COOLDOWN.take().map_or(ticks, i64::from))
        }
        "ev_player_use" => {
            USE_WORD.set(Some((query["block"] == true, query["item"] == true)));
            json!(true)
        }
        "ev_player_lan_acting" => json!(game.lan_acting.is_some()),
        "ev_player_mode_plugin" => {
            MODE_BY.with(|by| by.borrow_mut().push(query["name"].as_str().unwrap_or_default().to_owned()));
            json!(true)
        }
        "ev_player_mode_message" => {
            MODE_MESSAGE.with(|message| *message.borrow_mut() = Some(query["text"].clone()));
            json!(true)
        }
        "ev_player_tp_plugin" => {
            TP_BY_PLUGIN.set(TP_BY_PLUGIN.get() + 1);
            json!(true)
        }
        "ev_player_respawn_plugin" => {
            if let Some(uuid) = query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok()) {
                RESPAWN_BY_PLUGIN.with(|by| by.borrow_mut().insert(uuid));
            }
            json!(true)
        }
        "ev_player_command_as" => {
            let number = |key: &str| query[key].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
            if let (Some(by), Some(other)) = (number("by"), number("as")) {
                let first = COMMAND_AS.get().filter(|row| row.2 == game.ticks).map_or(by, |row| row.0);
                COMMAND_AS.set(Some((first, other, game.ticks)));
            }
            json!(true)
        }
        "ev_player_hide_event" => {
            let wanted = query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
            let level = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| Some(g.uuid) == wanted)).map(|g| json!(g.dimension));
            if let Some(level) = level {
                fire(game, "pl_hide", json!({ "uuid": query["uuid"], "name": query["name"], "level": level, "entity": query["entity"], "hidden": query["hidden"] }));
            }
            json!(true)
        }
        "ev_player_move_word" => {
            MOVE_WORD.set(Some((query["allowed"] == true, query["log"] == true)));
            json!(true)
        }
        "ev_player_gateway_to" => {
            let at = |i: usize| query["to"][i].as_f64().unwrap_or(0.0);
            GATEWAY_TO.set(Some([at(0), at(1), at(2)]));
            json!(true)
        }
        "ev_player_hopper_moved" => {
            let (pos, pull) = (query["pos"].clone(), query["pull"] == true);
            let (from, to) = if pull { (Value::Null, pos) } else { (pos, Value::Null) };
            let level = game.dim.id().to_owned();
            MOVED.with(|moved| moved.borrow_mut().take());
            let refused = fire(game, "pl_move_item", json!({ "level": level, "pull": pull, "from": from, "to": to, "stack": query["shown"], "inventory": query["inventory"] }));
            let moved = MOVED.with(|moved| moved.borrow_mut().take());
            HOPPER_DONE.set(!refused || moved.is_some());
            json!({ "refused": refused && moved.is_none(), "stack": moved.as_ref().map_or(Value::Null, stack_value) })
        }
        "ev_player_respawn_to" => {
            let at = |i: usize| query["to"][i].as_f64().unwrap_or(0.0);
            let level = query["level"].as_str().unwrap_or_default().to_owned();
            RESPAWN_TO.with(|to| *to.borrow_mut() = Some((level, [at(0), at(1), at(2)], at(3) as f32, at(4) as f32)));
            json!(true)
        }
        "ev_player_respawn_asked" => {
            let wanted = query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
            let ticks = game.ticks;
            json!(RESPAWN_ASKED.with(|asked| wanted.and_then(|uuid| asked.borrow().get(&uuid).copied()).is_some_and(|tick| ticks.saturating_sub(tick) <= 2)))
        }
        "ev_player_guest" => {
            let wanted = query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
            json!(game.lan.as_ref().is_some_and(|host| host.hub.guests.iter().any(|g| Some(g.uuid) == wanted)))
        }
        "ev_player_plain_kill" => {
            PLAIN_KILL.set(true);
            json!(true)
        }
        "ev_player_chat" => {
            let wanted = query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
            let line = query["line"].as_str().unwrap_or_default();
            let made = wanted.and_then(|uuid| Some((uuid, game.lan.as_ref()?.hub.chat_compelled(uuid, line)?)));
            let found = made.is_some();
            if let Some((uuid, (payload, held))) = made {
                horizon_client::paper_hub::chat_compelled(uuid, payload, held);
            }
            json!(found)
        }
        "ev_player_unleash_drop" => {
            UNLEASH_DROP.set(query["drop"].as_bool());
            json!(true)
        }
        "ev_player_bucket_result" => {
            BUCKET_RESULT.set(Some(stack_of(&query["stack"])));
            json!(true)
        }
        "ev_player_book_meta" => {
            let tags = query["tags"].as_array().into_iter().flatten().map(|page| horizon_client::lan::data::component_nbt(&horizon_client_render::packs::component(page).component())).collect();
            let generation = query["generation"].as_u64().and_then(|generation| u8::try_from(generation).ok());
            BOOK_META.set(stack_of(&query["stack"]).map(|stack| BookMeta { stack, author: query["author"].as_str().map(str::to_owned), generation, tags }));
            json!(true)
        }
        "ev_player_cooldown" => {
            COOLDOWN.set(query["ticks"].as_i64().map(|ticks| ticks as i32));
            json!(true)
        }
        "ev_player_shield" => {
            SHIELD_TICKS.set(query["ticks"].as_u64().map(|ticks| ticks as u32));
            json!(true)
        }
        "ev_player_eat" => {
            EATEN.set(stack_of(&query["stack"]));
            json!(true)
        }
        "ev_player_keep_throw" => {
            THROW_KEEP.set(query["keep"] == true);
            json!(true)
        }
        "ev_player_hatch" => {
            HATCH.set(Some((query["hatching"] == true, query["hatches"].as_i64().unwrap_or(1) as i32, query["type"].as_str().unwrap_or("minecraft:chicken").to_owned())));
            json!(true)
        }
        "ev_player_button" => {
            NEW_BUTTON.set(query["button"].as_i64().map(|button| button as i32));
            json!(true)
        }
        "ev_player_recipe" => {
            let recipe = query["recipe"].as_str().and_then(|id| Some(game.res.stations().recipe_index.display_ids(id).start).filter(|_| game.res.stations().recipe_index.displays(id) > 0));
            NEW_RECIPE.set(Some((recipe, query["shift"].as_bool())));
            json!(true)
        }
        "ev_player_beacon" => {
            let id = |key: &str| query[key].as_str().and_then(|name| game.lan.as_ref()?.hub.registry_id("minecraft:mob_effect", name));
            NEW_BEACON.set(Some((id("primary"), id("secondary"))));
            KEEP_PAYMENT.set(query["keep"] == true);
            json!(true)
        }
        "ev_player_page" => {
            NEW_PAGE.set(query["page"].as_i64().map(|page| page as i32));
            json!(true)
        }
        "ev_player_keep_attack" => {
            KEEP_ATTACK.set(query["keep"] == true);
            json!(true)
        }
        "ev_player_trade" => {
            let offer = (!query["offer"].is_null()).then(|| query["offer"].clone());
            TRADE.set(Some((offer, query["reward"].as_bool(), query["uses"] != false)));
            json!(true)
        }
        "ev_player_socket" => {
            let wanted = query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
            let live = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| Some(g.uuid) == wanted)).map(|g| g.address.to_string());
            json!(live.or_else(|| wanted.and_then(|uuid| SOCKETS.with(|sockets| sockets.borrow().get(&uuid).cloned()))))
        }
        // What each player has been sent, and what the event hasn't told yet.
        "ev_player_chunks" => {
            let Some(host) = game.lan.as_ref() else {
                return Some(Value::Null);
            };
            let mut out = Vec::new();
            CHUNKS.with(|seen| {
                let mut seen = seen.borrow_mut();
                seen.retain(|conn, _| host.hub.guests.iter().any(|g| g.conn == *conn));
                for g in &host.hub.guests {
                    let known = seen.entry(g.conn).or_default();
                    let loaded: Vec<(i32, i32)> = g.sent_chunks.iter().filter(|chunk| !known.contains(chunk)).copied().collect();
                    let unloaded: Vec<(i32, i32)> = known.iter().filter(|chunk| !g.sent_chunks.contains(chunk)).copied().collect();
                    if !loaded.is_empty() || !unloaded.is_empty() {
                        out.push(json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension, "loaded": loaded, "unloaded": unloaded }));
                        known.clone_from(&g.sent_chunks);
                    }
                }
            });
            Value::Array(out)
        }
        "ev_player_refuse" => {
            TOO_DEAR.set(true);
            json!(true)
        }
        "ev_player_replacement" => {
            REPLACED.set(stack_of(&query["stack"]));
            json!(true)
        }
        "ev_player_boost_keep" => {
            BOOST_KEEP.set(query["keep"] == true);
            json!(true)
        }
        "ev_player_brewed" => {
            BREWED.set(Some(query["stacks"].as_array().into_iter().flatten().map(stack_of).collect()));
            json!(true)
        }
        "ev_player_bed_allow" => {
            BED_ALLOW.set(query["allow"] == true);
            json!(true)
        }
        "ev_player_pick_to" => {
            PICK_TO.set(Some((query["source"].as_i64().unwrap_or(-1) as i32, query["target"].as_i64().unwrap_or(0) as i32)));
            json!(true)
        }
        // `RespawnConfig.forced`: an overworld point of the command has no config, and holds anyway.
        "ev_player_spawn_forced" => {
            let wanted = query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
            let Some(host) = game.lan.as_mut() else {
                return Some(json!(false));
            };
            let Some(g) = host.hub.guests.iter_mut().find(|g| Some(g.uuid) == wanted) else {
                return Some(json!(false));
            };
            let Some(config) = g.respawn_elsewhere.as_mut() else {
                return Some(json!(false));
            };
            config.3 = query["forced"] == true;
            json!(true)
        }
        "ev_player_commands_left" => {
            let left = query["left"].as_array().map(|left| left.iter().filter_map(Value::as_str).map(str::to_owned).collect());
            COMMANDS_LEFT.with(|commands| *commands.borrow_mut() = left);
            json!(true)
        }
        "ev_player_portal" => {
            let at = &query["at"];
            let key = [at[0].as_i64().unwrap_or(0) as i32, at[1].as_i64().unwrap_or(0) as i32, at[2].as_i64().unwrap_or(0) as i32];
            let back = query["back"].as_array().filter(|at| at.len() == 3).map(|at| (Dim::from_id(query["from"].as_str().unwrap_or_default()), [at[0].as_f64().unwrap_or(0.0), at[1].as_f64().unwrap_or(0.0), at[2].as_f64().unwrap_or(0.0)]));
            PORTALS.with(|portals| portals.borrow_mut().insert(key, (query["search"].as_i64().map(|radius| radius as i32), query["create"] != false, query["creation"].as_i64().map(|radius| radius as i32), back)));
            json!(true)
        }
        "ev_player_bed_blast" => {
            BED_BLAST.set(query["explode"] == true);
            json!(true)
        }
        "ev_player_said" => {
            SAID.with(|said| *said.borrow_mut() = query["text"].as_str().map(str::to_owned));
            json!(true)
        }
        "ev_player_spawn_answer" => {
            SPAWN_SAID.with(|said| {
                let mut said = said.borrow_mut();
                if let Some(at) = query["at"].as_array() {
                    said.at = Some([at[0].as_i64().unwrap_or(0) as i32, at[1].as_i64().unwrap_or(0) as i32, at[2].as_i64().unwrap_or(0) as i32]);
                    said.dim = query["level"].as_str().map(Dim::from_id);
                }
                if let Some(notify) = query["notify"].as_bool() {
                    said.notify = Some(notify);
                }
                if let Some(line) = query["line"].as_str() {
                    said.line = Some(line.to_owned());
                }
                if let Some(forced) = query["forced"].as_bool() {
                    said.forced = Some(forced);
                }
            });
            json!(true)
        }
        "ev_player_bed_keep" => {
            BED_KEEP.set(query["keep"] == true);
            json!(true)
        }
        "ev_player_slot_trigger" => {
            SLOT_TRIGGER.set(query["trigger"] == true);
            json!(true)
        }
        "ev_player_shear_set" => {
            let stacks = query["drops"].as_array().into_iter().flatten().filter_map(stack_of).collect();
            SHEAR_SET.with(|set| *set.borrow_mut() = Some(stacks));
            json!(true)
        }
        // The berries a click on this block will give, rolled once.
        "ev_player_harvest" => {
            let at = &query["pos"];
            let key = [at[0].as_i64().unwrap_or(0) as i32, at[1].as_i64().unwrap_or(0) as i32, at[2].as_i64().unwrap_or(0) as i32];
            if let Some(known) = HARVESTS.with(|all| all.borrow().get(&key).cloned()) {
                return Some(stacks(known.iter().map(Some)));
            }
            let state = game.world.block(key[0], key[1], key[2]);
            let table = if game.res.facts().get(state).name.contains("sweet_berry") { "minecraft:harvest/sweet_berry_bush" } else { "minecraft:harvest/cave_vine" };
            let seed = game.level.seed;
            let registry = game.pack.assets.registry;
            let rolled = world_items!(game).interact_loot(table, registry, state, &mut game.world.random, &mut game.rand_sequences, seed);
            let out = stacks(rolled.iter().map(Some));
            HARVESTS.with(|all| all.borrow_mut().insert(key, rolled));
            out
        }
        "ev_player_harvest_set" => {
            let at = &query["pos"];
            let key = [at[0].as_i64().unwrap_or(0) as i32, at[1].as_i64().unwrap_or(0) as i32, at[2].as_i64().unwrap_or(0) as i32];
            match query["drops"].as_array() {
                Some(drops) => {
                    let stacks = drops.iter().filter_map(stack_of).collect();
                    HARVESTS.with(|all| all.borrow_mut().insert(key, stacks));
                }
                None => {
                    HARVESTS.with(|all| all.borrow_mut().remove(&key));
                }
            }
            json!(true)
        }
        "ev_player_kick_plugin" => {
            KICK_PLUGIN.set(true);
            json!(true)
        }
        "ev_player_kick_reason" => {
            KICK_REASON.with(|said| *said.borrow_mut() = query["line"].as_str().map(str::to_owned));
            json!(true)
        }
        "ev_player_login_line" => {
            LOGIN_LINE.with(|line| *line.borrow_mut() = query["line"].as_str().map(str::to_owned));
            json!(true)
        }
        "ev_player_fly" => {
            FLY.set(query["fly"] == true);
            json!(true)
        }
        "ev_player_open_title" => {
            OPEN_TITLE.with(|title| *title.borrow_mut() = query["title"].as_str().map(str::to_owned));
            json!(true)
        }
        "ev_player_hopper_to" => {
            let at = &query["at"];
            HOPPER_TO.set(Some(at.as_array().map(|_| (at[0].as_i64().unwrap_or(0) as i32, at[1].as_i64().unwrap_or(0) as i32, at[2].as_i64().unwrap_or(0) as i32))));
            json!(true)
        }
        "ev_player_hopper_slots" => {
            HOPPER_SLOTS.set(query["closed"].as_u64().unwrap_or(0));
            json!(true)
        }
        // What the respawn the game just worked out came to.
        "ev_player_respawn" => {
            let wanted = query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
            RESPAWNS.with(|seen| wanted.and_then(|uuid| seen.borrow().get(&uuid).cloned())).unwrap_or(Value::Null)
        }
        "ev_player_bed_line" => {
            BED_LINE.set(query["line"].as_str().map(str::to_owned));
            json!(true)
        }
        "ev_player_hands" => {
            HANDS.set(Some((stack_of(&query["main"]), stack_of(&query["off"]))));
            json!(true)
        }
        "ev_player_loss" => {
            LOSS.set(query["loss"].as_u64().unwrap_or(0) as u32);
            json!(true)
        }
        "ev_player_moved" => {
            MOVED.set(stack_of(&query["stack"]));
            json!(true)
        }
        // The client took a stack we kept from it: it gets its inventory again.
        "ev_player_resend" => {
            let (Some(conn), Some(host)) = (conn_of(game, query), game.lan.as_mut()) else {
                return Some(json!(false));
            };
            let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn) else {
                return Some(json!(false));
            };
            host.hub.send_inventory(gi);
            json!(true)
        }
        // Where the player's last click landed on a block or an entity.
        "ev_player_clicked" => {
            let uuid = query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
            CLICKED.with(|clicked| clicked.borrow().iter().find(|(player, _)| Some(*player) == uuid).map_or(Value::Null, |(_, at)| json!(at)))
        }
        "ev_player_is_block" => json!(game.pack.assets.registry.default_state(query["name"].as_str().unwrap_or_default()).is_some()),
        _ => return None,
    })
}

// ---- Picking things up ----

/// `Inventory.canHold`, of the 36 slots.
fn room_for(game: &Game<'_>, slots: &[Option<ItemStack>], stack: &ItemStack) -> u32 {
    let max = world_items!(game).get(&stack.item).map_or(64, |item| item.max_stack).max(1);
    let room: u32 = slots.iter().take(36).map(|slot| match slot {
        None => max,
        Some(held) if held.same_item_same_components(stack) => max.saturating_sub(held.count as u32),
        Some(_) => 0,
    }).sum();
    room.min(stack.count as u32)
}

/// `PlayerAttemptPickupItemEvent`, then `PlayerPickupItemEvent` when any of it fits, for each stack
/// touching a guest; `PlayerPickupArrowEvent` for each arrow that would go to them. This is before
/// the game's own pass over them, with the hub back so that a listener can ask anything. A refused
/// stack isn't offered for 10 ticks, and a refused arrow is kept out of that pass (`arrow_gate`).
fn pickups_check(game: &mut Game<'_>) {
    use horizon_client::arrows::Touch;
    let (items, arrows) = (runtime::listens("pl_pickup"), runtime::listens("pl_pickup_arrow"));
    ARROWS_LEFT.with(|left| left.borrow_mut().clear());
    if !items && !arrows {
        return;
    }
    let mut stacks_near = Vec::new();
    let mut arrows_near = Vec::new();
    let Some(host) = game.lan.as_ref() else {
        return;
    };
    for g in host.hub.guests.iter().filter(|g| g.placed && !g.spectator() && !g.dead && g.health > 0.0) {
        let facts = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension });
        let words = horizon_client::lan::playerdata::uuid_ints(g.uuid);
        let at = g.position;
        if items {
            for d in game.dropped.iter().filter(|d| {
                d.pickup_delay == 0
                    && d.target.is_none_or(|t| t == words)
                    && (d.position.x - at[0]).abs() < 1.5
                    && (d.position.z - at[2]).abs() < 1.5
                    && d.position.y > at[1] - 0.6
                    && d.position.y < at[1] + 2.4
            }) {
                let room = room_for(game, &g.inventory, &d.stack);
                stacks_near.push((facts.clone(), d.id, g.entity_id, at, d.stack.clone(), room));
            }
        }
        if arrows {
            let dims = g.dimensions();
            let w = f64::from(dims.width) / 2.0 + 1.0;
            let top = at[1] + f64::from(dims.height) + 0.5;
            for shot in &game.creatures.projectiles {
                let Some(a) = shot.arrow.as_ref().filter(|a| !a.removed) else {
                    continue;
                };
                let p = shot.position;
                let touches = p.x - 0.25 < at[0] + w && p.x + 0.25 > at[0] - w && p.y < top && p.y + 0.5 > at[1] - 0.5 && p.z - 0.25 < at[2] + w && p.z + 0.25 > at[2] - w;
                if let (true, Touch::Give(stack)) = (touches, a.player_touch(g.creative())) {
                    let room = room_for(game, &g.inventory, &stack);
                    arrows_near.push((facts.clone(), g.conn, shot.id, stack, room));
                }
            }
        }
    }
    for (mut payload, id, player, at, stack, room) in stacks_near.into_iter().rev() {
        payload["kind"] = json!("attempt");
        payload["entity"] = json!(id);
        payload["stack"] = stack_value(&stack);
        payload["remaining"] = json!(stack.count as u32 - room);
        FLY.set(false);
        let mut refused = fire(game, "pl_pickup", payload.clone());
        if !refused && room > 0 {
            payload["kind"] = json!("pickup");
            refused = fire(game, "pl_pickup", payload);
        }
        if refused {
            if let Some(dropped) = game.dropped.iter_mut().find(|dropped| dropped.id == id) {
                dropped.pickup_delay = 10;
            }
            if let (true, Some(host)) = (FLY.take(), game.lan.as_ref()) {
                host.hub.take_item(id as i32, player, stack.count as i32, at);
            }
        }
    }
    for (mut payload, conn, id, stack, room) in arrows_near {
        payload["entity"] = json!(id);
        payload["stack"] = stack_value(&stack);
        payload["remaining"] = json!(stack.count as u32 - room);
        if room > 0 && fire(game, "pl_pickup_arrow", payload) {
            ARROWS_LEFT.with(|left| left.borrow_mut().push((conn, id)));
        }
    }
}

/// The head of `guest_arrows`: the arrows a listener refused this guest stay out of the game's pass.
pub(in crate::game) fn arrow_gate(game: &mut Game<'_>, host: &mut LanHost, gi: usize, rest: impl FnOnce(&mut Game<'_>, &mut LanHost)) -> ControlFlow<()> {
    if ARROW_PASS.get() {
        return ControlFlow::Continue(());
    }
    let conn = host.hub.guests[gi].conn;
    let refused: Vec<u32> = ARROWS_LEFT.with(|left| left.borrow().iter().filter(|row| row.0 == conn).map(|row| row.1).collect());
    if refused.is_empty() {
        return ControlFlow::Continue(());
    }
    let mut kept = Vec::new();
    let mut shots = std::mem::take(&mut game.creatures.projectiles);
    shots.retain(|shot| {
        let out = refused.contains(&shot.id);
        if out {
            kept.push(shot.clone());
        }
        !out
    });
    game.creatures.projectiles = shots;
    ARROW_PASS.set(true);
    rest(game, host);
    ARROW_PASS.set(false);
    game.creatures.projectiles.extend(kept);
    ControlFlow::Break(())
}

// ---- Menus opening ----

/// The menus of Bukkit's `InventoryType`s, for sending one again.
const MENUS: [(&str, &str); 16] = [
    ("HOPPER", "minecraft:hopper"),
    ("DISPENSER", "minecraft:generic_3x3"),
    ("DROPPER", "minecraft:generic_3x3"),
    ("FURNACE", "minecraft:furnace"),
    ("BLAST_FURNACE", "minecraft:blast_furnace"),
    ("SMOKER", "minecraft:smoker"),
    ("BREWING", "minecraft:brewing_stand"),
    ("CRAFTER", "minecraft:crafter_3x3"),
    ("SHULKER_BOX", "minecraft:shulker_box"),
    ("WORKBENCH", "minecraft:crafting"),
    ("ANVIL", "minecraft:anvil"),
    ("SMITHING", "minecraft:smithing"),
    ("GRINDSTONE", "minecraft:grindstone"),
    ("STONECUTTER", "minecraft:stonecutter"),
    ("LOOM", "minecraft:loom"),
    ("CARTOGRAPHY", "minecraft:cartography_table"),
];

/// The title a listener set: the same window again, which empties it on the client, then its slots.
/// `open_window` takes the next id and closes what's open, so it's told which id, with nothing open.
fn retitle(game: &mut Game<'_>, conn: u32, window: i32, menu: &str, title: &str) {
    let Some(host) = game.lan.as_mut() else {
        return;
    };
    let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn) else {
        return;
    };
    let kept = host.hub.guests[gi].menu.take();
    let next = host.hub.guests[gi].next_window;
    host.hub.guests[gi].next_window = window - 1;
    host.hub.open_window(conn, menu, horizon_protocol::text::literal(title));
    host.hub.guests[gi].next_window = next;
    host.hub.guests[gi].menu = kept;
    resync(game, conn, window);
}

/// The head of `native_open_menu`: a plugin's own menu opening.
pub(in crate::game) fn open_native(game: &mut Game<'_>, uuid: u128, rows: u8, items: &[Option<ItemStack>], open: impl FnOnce(&mut Game<'_>, Option<String>) -> Option<i32>) -> ControlFlow<Option<i32>> {
    if OPENING.get() {
        return ControlFlow::Continue(());
    }
    let payload = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.uuid == uuid)).map(|g| base("open", g, -1, &View { kind: "CHEST", own: items.to_vec(), pos: None, recipe: None }));
    let Some(mut payload) = payload else {
        return ControlFlow::Continue(());
    };
    payload["size"] = json!(usize::from(rows) * 9);
    OPEN_TITLE.with(|title| title.borrow_mut().take());
    if fire(game, "pl_open", payload) {
        return ControlFlow::Break(None);
    }
    let title = OPEN_TITLE.with(|title| title.borrow_mut().take());
    OPENING.set(true);
    let done = open(game, title);
    OPENING.set(false);
    ControlFlow::Break(done)
}

// ---- Hoppers ----

/// The head of `hopper_eject` and `hopper_suck`: `HopperInventorySearchEvent` for the block it
/// looks at, `PrepareHopperTransferEvent` for the two inventories, then the move, with `at` the
/// block the listeners left. `take_one` skips the slots they closed.
pub(in crate::game) fn hopper_gate(
    search: Option<(i32, i32, i32)>,
    game: &mut Game<'_>,
    pos: (i32, i32, i32),
    pull: bool,
    read: impl Fn(&mut Game<'_>, (i32, i32, i32)) -> Option<Vec<Option<ItemStack>>>,
    run: impl FnOnce(&mut Game<'_>, Option<(i32, i32, i32)>) -> bool,
) -> ControlFlow<bool> {
    if HOPPER_PASS.get() {
        return ControlFlow::Continue(());
    }
    let Some(at) = search else {
        return ControlFlow::Continue(());
    };
    let hook = if pull { "pl_hopper_in" } else { "pl_hopper_out" };
    let level = game.dim.id().to_owned();
    let inventory = |game: &mut Game<'_>, spot: (i32, i32, i32)| {
        read(game, spot).map(|own| {
            let kind = type_of_block(game, Slots::Block(spot));
            json!({ "type": kind, "size": own.len(), "items": stacks(own.iter().map(Option::as_ref)), "pos": [spot.0, spot.1, spot.2], "level": level })
        })
    };
    let seen = inventory(game, at);
    HOPPER_TO.set(None);
    fire(game, hook, json!({ "kind": "search", "pull": pull, "pos": [pos.0, pos.1, pos.2], "at": [at.0, at.1, at.2], "level": level, "inventory": seen }));
    let at = HOPPER_TO.take().unwrap_or(Some(at));
    if HOPPER_DONE.take() {
        return ControlFlow::Break(true);
    }
    HOPPER_SLOTS.set(0);
    if let Some(at) = at
        && let (Some(own), Some(other)) = (inventory(game, pos), inventory(game, at))
    {
        let (source, destination) = if pull { (other, own) } else { (own, other) };
        fire(game, hook, json!({ "kind": "prepare", "pull": pull, "source": source, "destination": destination, "level": level }));
    }
    HOPPER_PASS.set(true);
    let done = run(game, at);
    HOPPER_PASS.set(false);
    HOPPER_SLOTS.set(0);
    ControlFlow::Break(done)
}

/// The head of `take_one`: whether the hopper may take from this slot.
pub(in crate::game) fn hopper_slot(slot: usize) -> bool {
    HOPPER_SLOTS.get() & 1u64.checked_shl(slot as u32).unwrap_or(0) != 0
}

// ---- Recipes, shears ----

/// The head of `recipe_unlocked`: `PlayerRecipeDiscoverEvent`, one by one. A refused recipe comes
/// out of the book before anything is said about it.
pub(in crate::game) fn recipe_gate(game: &mut Game<'_>, unlocked: &[String], rest: impl FnOnce(&mut Game<'_>, &[String])) -> ControlFlow<()> {
    if RECIPE_PASS.get() || unlocked.is_empty() {
        return ControlFlow::Continue(());
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let (mut kept, mut refused) = (Vec::new(), Vec::new());
    for recipe in unlocked {
        if fire(game, "pl_recipes", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "recipe": recipe })) {
            refused.push(recipe.clone());
        } else {
            kept.push(recipe.clone());
        }
    }
    if refused.is_empty() {
        return ControlFlow::Continue(());
    }
    let index = &world_stations!(game).recipe_index;
    game.level.recipe_book.remove_recipes(index, &refused);
    RECIPE_PASS.set(true);
    rest(game, &kept);
    RECIPE_PASS.set(false);
    ControlFlow::Break(())
}

/// The head of `interact_mob`: `PlayerShearEntityEvent` for what the game shears right there, a
/// sheep or a mooshroom.
pub(in crate::game) fn shear_gate<R>(game: &mut Game<'_>, named: Option<i32>, roll: impl FnOnce(&mut Game<'_>, usize) -> Option<Vec<ItemStack>>, refused: R) -> ControlFlow<R> {
    if game.held().is_none_or(|held| &*held.item != "minecraft:shears") {
        return ControlFlow::Continue(());
    }
    let index = match named {
        Some(id) => game.mobs.iter().position(|mob| mob.id as i32 == id),
        None => game.mob_hit().filter(|(_, distance)| {
            let (hit, eye) = (game.pick_unfiltered(), game.eye_position());
            !matches!((hit, eye), (Some(hit), Some(eye)) if (0..3).map(|a| (hit.point[a] - eye[a]).powi(2)).sum::<f64>().sqrt() < *distance)
        }).map(|hit| hit.0),
    };
    let Some(index) = index else {
        return ControlFlow::Continue(());
    };
    let entity = game.mobs[index].id;
    let Some(rolled) = roll(game, index) else {
        return ControlFlow::Continue(());
    };
    if shear_ask(game, entity, "minecraft:shears", rolled) {
        return ControlFlow::Break(refused);
    }
    ControlFlow::Continue(())
}

// ---- The commands a player's client is told about ----

/// From `game::lan`, which holds it.
pub(in crate::game) fn sync_is(sync: fn() -> Option<std::sync::Arc<horizon_client::lan::data::SyncData>>) {
    SYNC_DATA.set(Some(sync));
}

/// Server mods' commands, as the hub's tree is made from them.
pub(in crate::game) fn mod_commands_are(commands: &[crate::game::native_server::NativeCommand]) {
    MOD_COMMANDS.with(|mods| *mods.borrow_mut() = commands.to_vec());
}

/// `PlayerCommandSendEvent`: the hub builds and sends the tree itself, and has no way to ask a
/// plugin. So it says whose it sent, and at the next turn the game builds the same tree from the
/// same data, asks, and sends what's left in its place.
fn commands_check(game: &mut Game<'_>) {
    use std::sync::atomic::Ordering;
    let sync = SYNC_DATA.get().and_then(|sync| sync());
    let listening = runtime::listens("pl_commands") && sync.as_ref().is_some_and(|sync| sync.commands.is_some());
    horizon_client::paper_hub::WATCH_COMMANDS.store(listening, Ordering::Relaxed);
    let sent = horizon_client::paper_hub::take_commands_sent();
    if sent.is_empty() {
        return;
    }
    let (Some(sync), Some(host)) = (sync, game.lan.as_ref()) else {
        return;
    };
    let Some(tree) = sync.commands.as_ref() else {
        return;
    };
    let dedicated = host.hub.config.dedicated && !host.hub.config.integrated;
    // A copy: `command_tree` tells `mod_commands_are` what it was given, which writes this list.
    let held = MOD_COMMANDS.with(|mods| mods.borrow().clone());
    let mods = crate::game::native_server::command_tree(&held, |name| host.hub.vanilla_command(name)).0;
    let due: Vec<_> = host
        .hub
        .guests
        .iter()
        .filter(|g| g.placed && sent.contains(&g.uuid))
        .map(|g| (g.conn, json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension }), tree.packet_with(g.permission, dedicated, &sync.builtin, &mods)))
        .collect();
    for (conn, mut payload, mut packet) in due {
        let root = packet.root as usize;
        let name_of = |nodes: &[play::CommandNode], child: i32| nodes.get(child as usize).map_or(String::new(), |node| node.name.clone());
        payload["commands"] = json!(packet.nodes[root].children.iter().map(|child| name_of(&packet.nodes, *child)).collect::<Vec<_>>());
        COMMANDS_LEFT.with(|left| left.borrow_mut().take());
        fire(game, "pl_commands", payload);
        if let Some(left) = COMMANDS_LEFT.with(|left| left.borrow_mut().take()) {
            let names: Vec<String> = packet.nodes[root].children.iter().map(|child| name_of(&packet.nodes, *child)).collect();
            let mut keep = names.iter().map(|name| left.contains(name));
            packet.nodes[root].children.retain(|_| keep.next().unwrap_or(false));
        }
        if let Some(host) = game.lan.as_ref() {
            host.hub.send_to(conn, &play::Clientbound::Commands(packet));
        }
    }
}

// ---- Portals ----

/// What `PlayerPortalEvent` said of a trip: how far to look, whether one may be built, how far from
/// there, and where the traveller came from.
type PortalRule = (Option<i32>, bool, Option<i32>, Option<(Dim, [f64; 3])>);

/// Where the traveller goes if the event closed this exit: only the level's own chunks can say
/// there's no portal, and those load around them as they arrive.
pub(in crate::game) fn portal_back() -> Option<(Dim, [f64; 3])> {
    PORTAL_RETURN.with(|portal| portal.borrow_mut().take())
}

/// The head of `nether_exit_portal`: what `PlayerPortalEvent` said of this exit. `found` looks
/// for a portal within a radius; with none there and none allowed, nothing is built.
pub(in crate::game) fn portal_gate<R>(at: [i32; 3], game: &Game<'_>, found: impl FnOnce(&Game<'_>, Option<i32>) -> bool, refused: R) -> ControlFlow<R> {
    use horizon_world::level::portal_travel::CREATION_RADIUS;
    let rules = PORTALS.with(|portals| portals.borrow_mut().remove(&at));
    CREATION_RADIUS.store(rules.and_then(|rules| rules.2).unwrap_or(16), std::sync::atomic::Ordering::Relaxed);
    let Some((search, create, _, back)) = rules else {
        return ControlFlow::Continue(());
    };
    if !create && !found(game, search) {
        PORTAL_RETURN.with(|portal| *portal.borrow_mut() = back);
        return ControlFlow::Break(refused);
    }
    PORTAL_RADIUS.set(search);
    ControlFlow::Continue(())
}

/// True if `PlayerPortalEvent` said no portal may be built here and there's none to come out of: the
/// player stays where they are. The level they'd go to is the open one, and `found` reads it from disk.
pub(in crate::game) fn no_exit(game: &Game<'_>, at: [i32; 3], found: impl FnOnce(&Game<'_>, Option<i32>) -> bool) -> bool {
    let Some((search, false)) = PORTALS.with(|portals| portals.borrow().get(&at).map(|rule| (rule.0, rule.1))) else {
        return false;
    };
    if found(game, search) {
        return false;
    }
    PORTALS.with(|portals| portals.borrow_mut().remove(&at));
    true
}

/// `PlayerTeleportEndGatewayEvent`, with the gateway's way worked out: where they go, and whether a
/// listener chose it. `None` if refused.
pub(in crate::game) fn gateway_asked(game: &mut Game<'_>, host: &LanHost, conn: u32, gateway: [i32; 3], to: horizon_core::geometry::Vec3) -> Option<(horizon_core::geometry::Vec3, bool)> {
    let g = host.hub.guest(conn)?;
    let payload = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": game.dim.id(), "pos": gateway, "to": [to.x, to.y, to.z] });
    GATEWAY_TO.set(None);
    if fire(game, "pl_gateway", payload) {
        return None;
    }
    Some(GATEWAY_TO.take().map_or((to, false), |at| (horizon_core::geometry::Vec3::new(at[0], at[1], at[2]), true)))
}

/// The head of `portal_blocks`: the radius a listener set, once.
pub(in crate::game) fn portal_radius() -> Option<i32> {
    PORTAL_RADIUS.take()
}

/// `setWillExplode(true)` on a refusal the game wouldn't have blown up.
fn refusal_blast(game: &mut Game<'_>, pos: [i32; 3], refused: bool) {
    if BED_BLAST.replace(true) && !refused {
        BED_PASS.set(true);
        if let Err(e) = game.destroy_bed_on_use(pos) {
            log::warn!("could not blow up the bed at {pos:?}: {e}");
        }
        BED_PASS.set(false);
    }
}

/// The head of `destroy_bed_on_use`: `PlayerBedFailEnterEvent`, where Paper asks it, before the
/// bed goes up. A cancel means nothing happens at all.
pub(in crate::game) fn bed_blast<R>(game: &mut Game<'_>, head: [i32; 3], kept: R) -> ControlFlow<R> {
    if BED_PASS.get() {
        return ControlFlow::Continue(());
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    BED_LINE.set(None);
    BED_BLAST.set(true);
    let refused = fire(game, "pl_bed_blast", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "pos": head, "result": "NOT_POSSIBLE_HERE" }));
    let line = BED_LINE.take();
    let blast = BED_BLAST.replace(true);
    if refused {
        return ControlFlow::Break(kept);
    }
    if let Some(line) = line {
        game.native_tell(uuid, horizon_client::text::Text::literal(line));
    }
    if blast { ControlFlow::Continue(()) } else { ControlFlow::Break(kept) }
}

/// Whether a plugin or mod owns `word`: as in Bukkit's command map, it comes before vanilla's, and
/// `minecraft:word` still reaches the game's.
pub(in crate::game) fn plugin_word(word: &str) -> bool {
    MOD_COMMANDS.with(|mods| mods.borrow().iter().any(|command| command.usage.split_whitespace().next() == Some(word)))
}

/// The head of `guest_message_command`: `AsyncChatCommandDecorateEvent` for `/say` and `/me`. A
/// changed message runs as that command, unsigned, as a decorated message is in Paper.
pub(in crate::game) fn say_gate(game: &mut Game<'_>, host: &mut LanHost, conn: u32, command: &str, run: impl FnOnce(&mut Game<'_>, &mut LanHost, &str) -> bool) -> ControlFlow<bool> {
    if SAY_PASS.get() {
        return ControlFlow::Continue(());
    }
    let (Some((word @ ("say" | "me"), text)), Some(g)) = (command.split_once(' '), host.hub.guest(conn)) else {
        return ControlFlow::Continue(());
    };
    let payload = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension, "message": text });
    SAID.with(|said| said.borrow_mut().take());
    fire(game, "pl_say", payload);
    let Some(said) = SAID.with(|said| said.borrow_mut().take()) else {
        return ControlFlow::Continue(());
    };
    SAY_PASS.set(true);
    let done = run(game, host, &format!("{word} {said}"));
    SAY_PASS.set(false);
    ControlFlow::Break(done)
}

/// The head of `trident_taken`: `PlayerPickupArrowEvent` for a guest's trident. A cancel leaves it.
pub(in crate::game) fn trident_gate(game: &mut Game<'_>, guest: Option<u32>, id: u32, stack: &ItemStack, creative_only: bool) -> ControlFlow<bool> {
    let (Some(conn), false) = (guest, creative_only) else {
        return ControlFlow::Continue(());
    };
    let Some(g) = game.lan.as_ref().and_then(|host| host.hub.guest(conn)) else {
        return ControlFlow::Continue(());
    };
    let room = room_for(game, &g.inventory, stack);
    let payload = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension, "kind": "arrow", "entity": id, "stack": stack_value(stack), "remaining": stack.count as u32 - room });
    if let Some(payload) = Some(payload).filter(|_| room > 0)
        && fire(game, "pl_pickup_arrow", payload)
    {
        return ControlFlow::Break(false);
    }
    ControlFlow::Continue(())
}

// ---- Spawn points ----

/// What a listener said of a spawn point about to be set.
#[derive(Default)]
struct SpawnSaid {
    at: Option<[i32; 3]>,
    dim: Option<Dim>,
    notify: Option<bool>,
    line: Option<String>,
    forced: Option<bool>,
}

/// What comes of a spawn point about to be set.
pub(in crate::game) struct SpawnAnswer {
    pub refused: bool,
    pub at: Option<[i32; 3]>,
    pub dim: Option<Dim>,
    pub notify: bool,
    pub line: Option<String>,
    pub forced: Option<bool>,
}

impl SpawnAnswer {
    pub(in crate::game) fn applied(&self, mut config: crate::game::respawn_anchor::RespawnConfig) -> crate::game::respawn_anchor::RespawnConfig {
        if let Some(at) = self.at {
            config.pos = at;
        }
        if let Some(dim) = self.dim {
            config.dim = dim;
        }
        if let Some(forced) = self.forced {
            config.forced = forced;
        }
        config
    }
}

/// `PlayerSetSpawnEvent` and `PlayerSpawnChangeEvent` as a bed, an anchor or `/spawnpoint` is about
/// to set the spawn point of the player in place. A bed's old point is kept for `PlayerBedLeaveEvent`.
pub(in crate::game) fn spawn_set(game: &mut Game<'_>, at: [i32; 3], notify: bool, cause: &'static str) -> SpawnAnswer {
    let quiet = SpawnAnswer { refused: false, at: None, dim: None, notify, line: None, forced: None };
    let uuid = game.native_acting_id();
    if cause == "BED" {
        let old: SpawnPoint = (game.spawn.map(|at| [at.x, at.y, at.z]), game.respawn_config.map(|config| config.saved()));
        BED_BEFORE.with(|all| all.borrow_mut().insert(uuid, old));
    }
    if !runtime::listens("pl_spawn_set") {
        return quiet;
    }
    let (name, level) = (game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let forced = cause == "COMMAND";
    SPAWN_SAID.with(|said| *said.borrow_mut() = SpawnSaid::default());
    let refused = fire(game, "pl_spawn_set", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "at": at.map(f64::from), "cause": cause, "notify": notify, "forced": forced }));
    let said = SPAWN_SAID.with(|said| std::mem::take(&mut *said.borrow_mut()));
    SpawnAnswer { refused, at: said.at, dim: said.dim, notify: said.notify.unwrap_or(notify) && said.line.is_none(), line: said.line, forced: said.forced }
}

/// A plugin's line in place of "Respawn point set".
pub(in crate::game) fn spawn_line(game: &mut Game<'_>, line: &str) {
    if game.lan_acting.is_some() {
        game.guest_packets.push(play::Clientbound::SystemChat(play::SystemChat { content: horizon_protocol::text::literal(line), overlay: false }));
    } else {
        game.system_message(horizon_client::text::Text::literal(line.to_owned()));
    }
}

// ---- Getting out of bed ----

/// `PlayerBedLeaveEvent` for those `Hub::wake` held back: a cancel leaves them in bed. A sleeper who
/// has died gets up without being asked.
fn wakes_check(game: &mut Game<'_>) {
    use std::sync::atomic::Ordering;
    let listening = runtime::listens("pl_bed_leave");
    horizon_client::paper_hub::HOLD_WAKES.store(listening, Ordering::Relaxed);
    for uuid in horizon_client::paper_hub::take_wakes() {
        let Some(g) = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.uuid == uuid)) else {
            continue;
        };
        let dead = g.dead || g.health <= 0.0;
        let payload = json!({ "uuid": format!("{uuid:032x}"), "name": g.name, "level": g.dimension, "pos": g.bed });
        BED_KEEP.set(true);
        if !dead && listening && fire(game, "pl_bed_leave", payload) {
            continue;
        }
        let keep = BED_KEEP.replace(true);
        let Some(mut host) = game.lan.take() else {
            return;
        };
        if let Some(gi) = host.hub.guests.iter().position(|g| g.uuid == uuid) {
            if !keep && let Some(old) = BED_BEFORE.with(|all| all.borrow_mut().remove(&uuid)) {
                let g = &mut host.hub.guests[gi];
                g.spawn = old.0;
                g.respawn_elsewhere = old.1;
            }
            horizon_client::paper_hub::WAKING.store(true, Ordering::Relaxed);
            game.wake_guest(&mut host, gi);
            horizon_client::paper_hub::WAKING.store(false, Ordering::Relaxed);
        }
        game.lan = Some(host);
    }
}

// ---- Tracking ----

// ---- Slots, shears, harvests ----

/// The head of what `check_inventory_trigger` does with a change: `PlayerInventorySlotChangeEvent`
/// for each of the 36 slots that differs. Whether the trigger still runs: if one slot wants it to.
pub(in crate::game) fn slots_changed(game: &mut Game<'_>) -> bool {
    if !runtime::listens("pl_slots") {
        return true;
    }
    let uuid = game.native_acting_id();
    let now: Vec<Value> = game.inventory.iter().take(36).map(|slot| stack(slot.as_ref())).collect();
    let before = SLOTS_SEEN.with(|seen| seen.borrow_mut().insert(uuid, now.clone()));
    let Some(before) = before else {
        return true;
    };
    let (name, level) = (game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let (mut changed, mut trigger) = (false, false);
    for (slot, (old, new)) in before.iter().zip(&now).enumerate() {
        if old != new {
            changed = true;
            SLOT_TRIGGER.set(true);
            fire(game, "pl_slots", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "slot": slot, "old": old, "new": new }));
            trigger |= SLOT_TRIGGER.replace(true);
        }
    }
    !changed || trigger
}

/// `PlayerShearEntityEvent`, with what's been rolled: `Err` for a refusal.
fn shear_ask(game: &mut Game<'_>, entity: u32, item: &str, rolled: Vec<ItemStack>) -> bool {
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let payload = json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "entity": entity, "item": item, "stack": game.held().map(stack_value), "hand": hand(game), "drops": stacks(rolled.iter().map(Some)) });
    SHEAR_SET.with(|set| set.borrow_mut().take());
    if fire(game, "pl_shear_mob", payload) {
        return true;
    }
    let drops = SHEAR_SET.with(|set| set.borrow_mut().take()).unwrap_or(rolled);
    SHEAR_OUT.with(|out| *out.borrow_mut() = Some(drops));
    false
}

/// What `PlayerShearEntityEvent` left, for the shearing it's about.
pub(in crate::game) fn shear_drops() -> Option<Vec<ItemStack>> {
    SHEAR_OUT.with(|out| out.borrow_mut().take())
}

/// The head of `interact_shear_mob`: `roll` is the table's drops, if this one can be sheared.
pub(in crate::game) fn shear_golem(game: &mut Game<'_>, index: usize, item: &str, roll: impl FnOnce(&mut Game<'_>) -> Option<Vec<ItemStack>>) -> ControlFlow<bool> {
    if item != "minecraft:shears" {
        return ControlFlow::Continue(());
    }
    let (Some(entity), Some(rolled)) = (game.mobs.get(index).map(|mob| mob.id), roll(game)) else {
        return ControlFlow::Continue(());
    };
    if shear_ask(game, entity, item, rolled) {
        return ControlFlow::Break(false);
    }
    ControlFlow::Continue(())
}

/// The berries `PlayerHarvestBlockEvent` left, for the harvest it's about.
pub(in crate::game) fn harvest_drops(pos: [i32; 3]) -> Option<Vec<ItemStack>> {
    HARVESTS.with(|all| all.borrow_mut().remove(&pos))
}

/// The id `launch_thrown` gives a potion: the one `PlayerLaunchProjectileEvent` was told, or 0 for one
/// nobody asked about, which gets its id as it first moves.
pub(in crate::game) fn thrown_id() -> u32 {
    THROWN_ID.take()
}

/// What `guest_command` runs: `PlayerCommandPreprocessEvent` may have changed it, this tick.
pub(in crate::game) fn command_to(game: &Game<'_>, command: &str) -> Option<String> {
    let row = COMMAND_TO.with(|row| row.borrow_mut().take())?;
    (row.2 == game.ticks && row.0 == command.trim_start_matches('/')).then_some(row.1)
}

/// Whether `PlayerBedEnterEvent` let this try past what the game would refuse.
pub(in crate::game) fn bed_forced() -> bool {
    BED_FORCE.get()
}

// ---- Fishing, moment by moment ----

/// `PlayerFishEvent` for what's in the water: whether a listener refused it.
pub(in crate::game) fn fish_state(game: &mut Game<'_>, hook: u32, state: &str) -> bool {
    if !runtime::listens("pl_fish_tick") {
        return false;
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let hand = hand(game);
    fire(game, "pl_fish_tick", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "state": state, "hook": hook, "hand": hand }))
}

/// `PlayerFishEvent`'s `CAUGHT_FISH`, for the item just made: a refusal takes it back.
pub(in crate::game) fn fish_caught(game: &mut Game<'_>, hook: u32) -> bool {
    if !runtime::listens("pl_fish_reel") {
        return false;
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let (caught, hand) = (game.dropped.last().map(|item| item.id), hand(game));
    let exp = fish_exp(true, game);
    let refused = fire(game, "pl_fish_reel", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "state": "CAUGHT_FISH", "caught": caught, "hook": hook, "hand": hand, "exp": exp }));
    if refused {
        game.dropped.pop();
        FISH_EXP.set(None);
    }
    refused
}

// ---- Ammunition, statistics, criteria ----

/// `Player.getProjectile` with `PlayerReadyArrowEvent`: the first the listeners let through.
pub(in crate::game) fn ready_slot(game: &mut Game<'_>, weapon: &str) -> Option<usize> {
    if !runtime::listens("pl_ready_arrow") {
        return game.projectile_slot(weapon);
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    let bow = game.held().map(stack_value);
    for slot in game.projectile_slots(weapon) {
        let arrow = game.stack_in(slot).map(stack_value);
        if !fire(game, "pl_ready_arrow", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "bow": bow, "arrow": arrow })) {
            return Some(slot);
        }
    }
    None
}

/// The head of a guest's turn: from here to its end, what a listener could stop waits for it.
pub(in crate::game) fn stats_on() {
    STAT_ASKING.set(true);
}

/// `CraftEventFactory.handleStatisticsIncrease`: these are left out there as too many.
const QUIET_STATS: [&str; 20] = ["fall_one_cm", "boat_one_cm", "climb_one_cm", "walk_on_water_one_cm", "walk_under_water_one_cm", "fly_one_cm", "horse_one_cm", "minecart_one_cm", "pig_one_cm", "play_time", "swim_one_cm", "walk_one_cm", "sprint_one_cm", "crouch_one_cm", "time_since_death", "sneak_time", "total_world_time", "time_since_rest", "aviate_one_cm", "strider_one_cm"];

/// Whether this count waits for `PlayerStatisticIncrementEvent`.
pub(in crate::game) fn stat_asked(n: i32, key: &str) -> bool {
    STAT_ASKING.get() && n > 0 && !QUIET_STATS.contains(&key.trim_start_matches("minecraft:"))
}

/// The end of a guest's turn: each count that waited, if a listener lets it.
pub(in crate::game) fn stat_events(game: &mut Game<'_>) {
    STAT_ASKING.set(false);
    let asked = std::mem::take(&mut game.stats.asked);
    if asked.is_empty() {
        return;
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    for (kind, key, amount) in asked {
        let short = kind.trim_start_matches("minecraft:").to_owned();
        let total = game.stats.get(&short, &key) + amount;
        if !fire(game, "pl_stats", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "kind": kind, "key": key, "amount": amount, "total": total })) {
            game.stats.add(&short, &key, amount);
        }
    }
}

/// `PlayerAdvancementCriterionGrantEvent`, before the criterion goes in.
pub(in crate::game) fn criterion_refused(game: &mut Game<'_>, advancement: &str, criterion: &str) -> bool {
    if !runtime::listens("pl_criterion") {
        return false;
    }
    let (uuid, name, level) = (game.native_acting_id(), game.acting_player_name().to_owned(), game.dim.id().to_owned());
    fire(game, "pl_criterion", json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": level, "advancement": advancement, "criterion": criterion }))
}

// ---- Menus waiting for their event ----

type LaterRun = Box<dyn for<'a, 'b> FnOnce(&'a mut Game<'b>, bool, Option<String>)>;

/// What a waiting menu shows.
enum Shown {
    Block([i32; 3]),
    Ready(View, Option<&'static str>),
}

/// A menu a guest's use would open. The hub is out of the game's hands then and a listener could
/// reach nothing, so the use stops here, and `InventoryOpenEvent` fires once the hub's back, right
/// after the packet. Nothing has happened to the chest yet: the menu then opens in a turn of its own.
struct Later {
    conn: u32,
    shown: Shown,
    run: LaterRun,
}

fn hold_menu(game: &Game<'_>, shown: Shown, run: LaterRun) -> Option<LaterRun> {
    let Some(conn) = game.lan_acting else {
        return Some(run);
    };
    if OPENING.get() || !runtime::listens("pl_open") {
        return Some(run);
    }
    LATER.with(|later| later.borrow_mut().push(Later { conn, shown, run }));
    None
}

/// The head of `leave_guest`: `run_guest_commands` has something to do.
pub(in crate::game) fn open_mark(host: &mut LanHost) {
    if let Some(conn) = LATER.with(|later| later.borrow().first().map(|later| later.conn)) {
        host.commands.push(GuestCommand { conn, command: MARK.to_owned(), permission: 0, integrated: false, quiet: true });
    }
}

/// The head of `open_menu`, for what's a menu: signs, command blocks and the like aren't.
pub(in crate::game) fn menu_later(game: &mut Game<'_>, pos: [i32; 3]) -> ControlFlow<()> {
    let key = (pos[0], pos[1], pos[2]);
    let name = game.res.facts().get(game.world.block(pos[0], pos[1], pos[2])).name.clone();
    let menu = !game.chest_menu_blocked(key, &name)
        && horizon_client::command_block::Mode::of_block(&name).is_none()
        && horizon_client::signs::sign_kind(&name).is_none()
        && !matches!(&*name, "minecraft:structure_block" | "minecraft:jigsaw" | "minecraft:test_block" | "minecraft:test_instance_block")
        && !(crate::game::storage::is_shulker_box(&name) && !game.shulker_can_open(pos));
    if !menu {
        return ControlFlow::Continue(());
    }
    match hold_menu(game, Shown::Block(pos), Box::new(move |game, refused, _| {
        if !refused {
            game.open_menu(pos);
        }
    })) {
        Some(_) => ControlFlow::Continue(()),
        None => ControlFlow::Break(()),
    }
}

/// The head of `open_mount_menu`.
pub(in crate::game) fn mount_later(game: &mut Game<'_>, index: usize) -> ControlFlow<()> {
    let Some(mob) = game.mobs.get(index) else {
        return ControlFlow::Continue(());
    };
    let id = mob.id;
    let own = mob.extra.items.clone();
    match hold_menu(game, Shown::Ready(View { kind: "CHEST", own, pos: None, recipe: None }, None), Box::new(move |game, refused, _| {
        if let (false, Some(index)) = (refused, game.mobs.iter().position(|mob| mob.id == id)) {
            game.open_mount_menu(index);
        }
    })) {
        Some(_) => ControlFlow::Continue(()),
        None => ControlFlow::Break(()),
    }
}

/// The head of what opens a chest boat or chest cart: `open` finds it again by its id.
pub(in crate::game) fn vehicle_later(game: &mut Game<'_>, id: u32, own: Vec<Option<ItemStack>>, open: fn(&mut Game<'_>, u32)) -> ControlFlow<()> {
    match hold_menu(game, Shown::Ready(View { kind: "CHEST", own, pos: None, recipe: None }, Some("minecraft:generic_9x3")), Box::new(move |game, refused, _| {
        if !refused {
            open(game, id);
        }
    })) {
        Some(_) => ControlFlow::Continue(()),
        None => ControlFlow::Break(()),
    }
}

/// The start of `open_merchant_state`, which takes its state by value: that waits too, and `None`
/// says so. `run` opens it, with a title in place of the merchant's name, or lets it go.
pub(in crate::game) fn merchant_later<S: 'static>(game: &mut Game<'_>, state: S, run: for<'a, 'b> fn(&'a mut Game<'b>, S, bool, Option<String>)) -> Option<S> {
    let Some(conn) = game.lan_acting.filter(|_| !OPENING.get() && runtime::listens("pl_open")) else {
        return Some(state);
    };
    let shown = Shown::Ready(View { kind: "MERCHANT", own: vec![None; 3], pos: None, recipe: None }, None);
    LATER.with(|later| later.borrow_mut().push(Later { conn, shown, run: Box::new(move |game, refused, title| run(game, state, refused, title)) }));
    None
}

/// What a block's menu shows, and the menu's name for sending it again.
fn block_shown(game: &Game<'_>, g: &horizon_client::lan::guest::Guest, pos: [i32; 3]) -> Option<(View, Option<&'static str>)> {
    let key = (pos[0], pos[1], pos[2]);
    let name = game.res.facts().get(game.world.block(pos[0], pos[1], pos[2])).name.clone();
    let station = |kind: &'static str, size: usize, menu: &'static str| Some((View { kind, own: vec![None; size], pos: Some(pos), recipe: None }, Some(menu)));
    match name.trim_start_matches("minecraft:") {
        "crafting_table" => return station("WORKBENCH", 10, "minecraft:crafting"),
        "anvil" | "chipped_anvil" | "damaged_anvil" => return station("ANVIL", 3, "minecraft:anvil"),
        "smithing_table" => return station("SMITHING", 4, "minecraft:smithing"),
        "grindstone" => return station("GRINDSTONE", 3, "minecraft:grindstone"),
        "stonecutter" => return station("STONECUTTER", 2, "minecraft:stonecutter"),
        "loom" => return station("LOOM", 4, "minecraft:loom"),
        "cartography_table" => return station("CARTOGRAPHY", 3, "minecraft:cartography_table"),
        "enchanting_table" => return station("ENCHANTING", 2, "minecraft:enchantment"),
        "beacon" => return station("BEACON", 1, "minecraft:beacon"),
        "lectern" => {
            let book = match game.world.block_entities.get(&key) {
                Some(BlockEntity::Lectern(lectern)) => lectern.book.clone(),
                _ => None,
            };
            return Some((View { kind: "LECTERN", own: vec![book], pos: Some(pos), recipe: None }, Some("minecraft:lectern")));
        }
        _ => {}
    }
    let slots = if &*name == "minecraft:ender_chest" {
        Slots::Ender
    } else {
        match game.chest_partner(key) {
            Some((other, true)) => Slots::Double(key, other),
            Some((other, false)) => Slots::Double(other, key),
            None => Slots::Block(key),
        }
    };
    let own = slots.read(&game.world, g)?;
    let kind = type_of_block(game, slots);
    let menu = match (kind, own.len()) {
        (_, 54) => Some("minecraft:generic_9x6"),
        ("CHEST" | "ENDER_CHEST" | "BARREL", _) => Some("minecraft:generic_9x3"),
        (kind, _) => MENUS.iter().find(|row| row.0 == kind).map(|row| row.1),
    };
    Some((View { kind, own, pos: Some(pos), recipe: None }, menu))
}

/// `InventoryOpenEvent` for a waiting menu, then the menu, in a turn of the guest's own.
fn open_now(game: &mut Game<'_>, later: Later) {
    let Later { conn, shown, run } = later;
    let facts = game.lan.as_ref().and_then(|host| host.hub.guest(conn)).and_then(|g| {
        let (view, menu) = match shown {
            Shown::Block(pos) => block_shown(game, g, pos)?,
            Shown::Ready(view, menu) => (view, menu),
        };
        Some((base("open", g, -1, &view), menu))
    });
    OPEN_TITLE.with(|title| title.borrow_mut().take());
    let (refused, menu) = match facts {
        Some((payload, menu)) => (fire(game, "pl_open", payload), menu),
        None => (false, None),
    };
    let title = OPEN_TITLE.with(|title| title.borrow_mut().take());
    let Some(mut host) = game.lan.take() else {
        return;
    };
    let told = title.clone();
    OPENING.set(true);
    game.as_guest(&mut host, conn, |game| run(game, refused, told));
    OPENING.set(false);
    let window = host.hub.guest(conn).map(|g| g.next_window);
    game.lan = Some(host);
    if let (false, Some(title), Some(menu), Some(window)) = (refused, title, menu, window) {
        retitle(game, conn, window, menu, &title);
    }
}

// ---- Back to configuration ----

/// Asks for `switchToConfig`. False when it's already asked for: a second `reconfigure` is refused.
pub(in crate::game) fn ask_switch_to_config(uuid: u128) -> bool {
    RECONFIGURES.with(|all| {
        let mut all = all.borrow_mut();
        let fresh = !all.contains(&uuid);
        if fresh {
            all.push(uuid);
        }
        fresh
    })
}

/// `ServerGamePacketListenerImpl.switchToConfig` for whoever asked: from this packet on the client reads
/// no play packets. It leaves the game once it has said it understood. The game calls this every tick,
/// as the turn below runs only when a plugin listens for moves.
pub(in crate::game) fn send_switches(host: &crate::game::lan::LanHost) {
    if RECONFIGURES.with(|all| all.borrow().is_empty()) {
        return;
    }
    for uuid in RECONFIGURES.with(|all| std::mem::take(&mut *all.borrow_mut())) {
        if let Some(g) = host.hub.guests.iter().find(|g| g.uuid == uuid) {
            g.send(&play::Clientbound::StartConfiguration(horizon_protocol::packets::Empty));
            g.sender.flush();
            g.sender.paused.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// `ServerGamePacketListenerImpl.switchToConfig`: from this packet on the client reads no play
/// packets, so the game's stop here. It leaves the game once it has said it understood.
pub(in crate::game) fn reconfigure_turn(game: &mut Game<'_>) {
    chat_turn(game);
    spawns_turn(game);
    steps_turn(game);
    modes_check(game);
    fails_check(game);
    inventories_back(game);
    if let Some(host) = game.lan.as_ref() {
        send_switches(host);
    }
}

/// `PlayerTrackEntityEvent` for each pair the tracker would show. Refused, it stays hidden and is
/// asked about again a second later, as Paper asks whenever the entity could be tracked.
fn tracks_check(game: &mut Game<'_>) {
    use std::sync::atomic::Ordering;
    use horizon_client::paper_hub as hub;
    let listening = runtime::listens("pl_track");
    hub::WATCH_TRACKS.store(listening, Ordering::Relaxed);
    let now = u64::from(game.ticks);
    for pair in hub::take_track_asked() {
        if TRACKS.with(|tracks| tracks.borrow().get(&pair).is_some_and(|next| *next > now)) {
            continue;
        }
        let Some(g) = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.entity_id == pair.0)) else {
            hub::set_track_allowed(pair, true);
            continue;
        };
        let payload = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension, "entity": pair.1 });
        if listening && fire(game, "pl_track", payload) {
            TRACKS.with(|tracks| tracks.borrow_mut().insert(pair, now + 20));
        } else {
            TRACKS.with(|tracks| tracks.borrow_mut().remove(&pair));
            hub::set_track_allowed(pair, true);
        }
    }
}

// ---- Kicks ----

/// `PlayerKickEvent.Cause`'s name, from what the player is told.
fn kick_cause(reason: &horizon_world::nbt::Tag) -> &'static str {
    let text = crate::game::commands::paper_entities::tag_json(reason);
    match text["translate"].as_str().unwrap_or_default() {
        "multiplayer.disconnect.flying" => "FLYING_PLAYER",
        "multiplayer.disconnect.invalid_vehicle_movement" => "INVALID_VEHICLE_MOVEMENT",
        "multiplayer.disconnect.invalid_player_movement" => "INVALID_PLAYER_MOVEMENT",
        "multiplayer.disconnect.idling" => "IDLING",
        "multiplayer.disconnect.duplicate_login" => "DUPLICATE_LOGIN",
        "multiplayer.disconnect.illegal_characters" => "ILLEGAL_CHARACTERS",
        "multiplayer.disconnect.chat_validation_failed" => "CHAT_VALIDATION_FAILED",
        "multiplayer.disconnect.expired_public_key" => "EXPIRED_PROFILE_PUBLIC_KEY",
        "multiplayer.disconnect.out_of_order_chat" => "OUT_OF_ORDER_CHAT",
        "multiplayer.disconnect.unsigned_chat" => "UNSIGNED_CHAT",
        "multiplayer.disconnect.banned" => "BANNED",
        "multiplayer.disconnect.not_whitelisted" => "WHITELIST",
        "multiplayer.disconnect.server_shutdown" => "RESTART_COMMAND",
        "disconnect.timeout" => "TIMEOUT",
        "disconnect.exceeded_packet_rate" => "TOO_MANY_PENDING_CHATS",
        "disconnect.spam" => "SPAM",
        _ => "UNKNOWN",
    }
}

/// `PlayerKickEvent` for what `Hub::kick` held back: a cancel keeps them on.
fn kicks_check(game: &mut Game<'_>) {
    use std::sync::atomic::Ordering;
    use horizon_client::paper_hub as hub;
    let listening = runtime::listens("pl_kick");
    hub::HOLD_KICKS.store(listening, Ordering::Relaxed);
    for (conn, reason) in hub::take_kicks() {
        let Some(host) = game.lan.as_ref() else {
            return;
        };
        let Some(g) = host.hub.guest(conn) else {
            continue;
        };
        let cause = match (KICK_BY.take(), kick_cause(&reason)) {
            (Some(by), "UNKNOWN") => by,
            (_, cause) => cause,
        };
        let payload = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension, "reason": hub::english(&host.hub, &reason), "cause": cause });
        KICK_REASON.with(|said| said.borrow_mut().take());
        let refused = listening && paper_bridge::on_refusable(game, "pl_kick", payload);
        let said = KICK_REASON.with(|said| said.borrow_mut().take());
        if !refused && let Some(host) = game.lan.as_mut() {
            hub::KICKING.store(true, Ordering::Relaxed);
            host.hub.kick(conn, said.map_or(reason, |line| horizon_protocol::text::literal(&line)));
            hub::KICKING.store(false, Ordering::Relaxed);
        }
    }
}

/// What `/kill` says of a death.
pub(in crate::game) fn kill_cause() -> &'static str {
    if PLAIN_KILL.take() { "death.attack.generic" } else { "death.attack.genericKill" }
}

/// `/kick` as it runs: whose it is, for the event that follows.
pub(in crate::game) fn kick_note(game: &Game<'_>) {
    if !KICK_PLUGIN.take() {
        KICK_BY.set(Some("KICK_COMMAND"));
    } else {
        KICK_BY.set(Some("PLUGIN"));
    }
    let _ = game;
}

// ---- Logging in ----

/// The head of `lan_guest_joined`: `PlayerLoginEvent`, before the game takes them. A refusal
/// disconnects them from there.
pub(in crate::game) fn login_gate(game: &mut Game<'_>, host: &mut LanHost, conn: u32) -> ControlFlow<()> {
    if !runtime::listens("pl_login") {
        return ControlFlow::Continue(());
    }
    let Some(g) = host.hub.guest(conn) else {
        return ControlFlow::Continue(());
    };
    let handshake = horizon_client::paper_hub::handshake_of(g.address);
    let payload = json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name, "level": g.dimension, "address": g.address.ip().to_string(), "socket": g.address.to_string(), "hostname": handshake.as_ref().map(|handshake| format!("{}:{}", handshake.host, handshake.port)) });
    LOGIN_LINE.with(|line| line.borrow_mut().take());
    if !fire(game, "pl_login", payload) {
        return ControlFlow::Continue(());
    }
    let line = LOGIN_LINE.with(|line| line.borrow_mut().take()).unwrap_or_default();
    host.hub.kick(conn, horizon_protocol::text::literal(&line));
    ControlFlow::Break(())
}
