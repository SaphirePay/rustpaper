// The engine's side of the entity events plugins hear (deaths, conversions, level changes, block use, effects):
// what a mob's own code does has no `Game` in hand, so those arrive through `paper_hub` and carry the mob itself.
// Some steps here are inert (held effects, mob sleep, the asks before a hit): no plugin event hears them. See NOTICE.
use crate::game::Game;
use crate::game::paper_events::block_events as blocks;
use crate::game::paper_events::server_entity_level_change_events as level_change;
use crate::game::paper_events::server_living_entity_events as living;
use crate::game::paper_events::server_player_events as players;
use crate::game::lan::LanHost;
use horizon_client::effects::EffectInstance;
use horizon_client::paper_hub::{self as hub, EffectStep, HeldEffect, MobEffectRow};
use horizon_client::first_person::Hand;
use horizon_client::items::ItemStack;
use horizon_client::mobs::{Mob, Who};
use std::cell::RefCell;
use std::sync::Mutex;

thread_local! {
    /// `MobEffectUtil`'s stack: `/execute run` nests one command in another.
    static COMMANDS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// While it lives, effects happen under this command.
pub(in crate::game) struct CommandScope;

impl Drop for CommandScope {
    fn drop(&mut self) {
        COMMANDS.with(|commands| {
            commands.borrow_mut().pop();
        });
    }
}

/// `CommandQueueEntry.execute`: `id` is the game's name for what runs, `effect.give`; the command is its first word.
pub(in crate::game) fn command_scope(id: &str) -> CommandScope {
    COMMANDS.with(|commands| commands.borrow_mut().push(id.split('.').next().unwrap_or(id).to_owned()));
    CommandScope
}

fn command() -> Option<String> {
    COMMANDS.with(|commands| commands.borrow().last().cloned())
}

/// Whether anything hears effects: no plugin event does here (`EntityPotionEffectEvent` is the ev_entity rows').
fn heard() -> bool {
    false
}

/// Once a tick, as listeners may come with a mod at any time.
pub(in crate::game) fn hear_effects(game: &Game<'_>) {
    // Nothing asks about an effect step, a mob's fatal hit or a hit before it lands: the hub holds none of them.
    hub::hear_effects(false);
    hub::hear_deaths(false);
    hub::hear_hit_asking(false);
    let tags = &game.stations().enchantments.entity_tags;
    hub::effect_tags(tags.get("minecraft:immune_to_infested"), tags.get("minecraft:immune_to_oozing"), tags.get("minecraft:ignores_poison_and_regen"));
    hub::hear_hits(hit_landed, mob_may_die);
}

/// A step of a mob's effect: no plugin event hears one (Bukkit's `EntityPotionEffectEvent` comes from the ev_entity
/// rows), so each goes ahead. The held path that leads here never runs: the hub holds nothing (`hear_effects`).
fn mob_step(_game: &mut Game<'_>, _step: EffectStep, _id: u32, _effect: MobEffectRow, _ambient: bool, _visible: bool) -> bool {
    true
}

fn mob_mut<'a>(game: &'a mut Game<'_>, id: u32) -> Option<&'a mut Mob> {
    game.mobs.iter_mut().find(|m| m.id == id)
}

fn flags_of(game: &Game<'_>, id: u32, effect: &str) -> (bool, bool) {
    game.mobs.iter().find(|m| m.id == id).map_or((false, true), |m| (m.extra.ambient_effects.iter().any(|e| *e == effect), !m.extra.hidden_effects.iter().any(|e| *e == effect)))
}

thread_local! {
    /// Changes whose mob isn't in the list yet, with the drains they've missed.
    static MISSING: RefCell<Vec<(u8, HeldEffect)>> = const { RefCell::new(Vec::new()) };
    /// A listener or the game turned a held change down.
    static REFUSED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the last changes `effects_now` ran were turned down, for a command to say so.
pub(in crate::game) fn effects_refused() -> bool {
    REFUSED.take()
}

fn held_mob(change: &HeldEffect) -> u32 {
    match change {
        HeldEffect::Asked { mob, .. } | HeldEffect::Add { mob, .. } | HeldEffect::Remove { mob, .. } | HeldEffect::Clear { mob, .. } | HeldEffect::Gone { mob, .. } | HeldEffect::Ending { mob, .. } | HeldEffect::Hit { mob, .. } => *mob,
    }
}

fn run_held(game: &mut Game<'_>, change: HeldEffect) {
    match change {
        HeldEffect::Asked { mob, effect } => {
            mob_step(game, EffectStep::AllowAdd, mob, effect, false, true);
        }
        HeldEffect::Add { mob, effect, ambient, visible, cause, from } => {
            if !mob_step(game, EffectStep::AllowAdd, mob, effect, ambient, visible) {
                return REFUSED.set(true);
            }
            let Some(found) = mob_mut(game, mob) else {
                return;
            };
            horizon_client::mobs::ee::effect_cause(cause);
            horizon_client::mobs::ee::put_effect_from(from);
            if !hub::mob_affected(&found.kind.id, effect.0) || horizon_client::mobs::ee::effect_refused(found, effect.0, effect.1, effect.2) {
                return REFUSED.set(true);
            }
            mob_step(game, EffectStep::BeforeAdd, mob, effect, ambient, visible);
            let Some(found) = mob_mut(game, mob) else {
                return;
            };
            let new = !found.has_effect(effect.0);
            hub::passing_effects(|| found.add_effect_with(effect.0, effect.1, effect.2, ambient, visible));
            if new {
                mob_step(game, EffectStep::AfterAdd, mob, effect, ambient, visible);
            }
        }
        HeldEffect::Remove { mob, effect } => {
            let (ambient, visible) = flags_of(game, mob, effect.0);
            if !mob_step(game, EffectStep::AllowRemove, mob, effect, ambient, visible) {
                return REFUSED.set(true);
            }
            mob_step(game, EffectStep::BeforeRemove, mob, effect, ambient, visible);
            if let Some(found) = mob_mut(game, mob) {
                hub::passing_effects(|| found.remove_effect(effect.0));
            }
            mob_step(game, EffectStep::AfterRemove, mob, effect, ambient, visible);
        }
        HeldEffect::Clear { mob, cause } => {
            let Some(list) = game.mobs.iter().find(|m| m.id == mob).map(|m| m.extra.effects.clone()) else {
                return;
            };
            let flags: Vec<_> = list.iter().map(|effect| flags_of(game, mob, effect.0)).collect();
            for (effect, (ambient, visible)) in list.iter().zip(&flags) {
                mob_step(game, EffectStep::BeforeRemove, mob, *effect, *ambient, *visible);
            }
            let mut kept = Vec::new();
            for (effect, (ambient, visible)) in list.iter().zip(&flags) {
                if !mob_step(game, EffectStep::AllowRemove, mob, *effect, *ambient, *visible) {
                    kept.push(*effect);
                }
            }
            if let Some(found) = mob_mut(game, mob) {
                hub::passing_effects(|| horizon_client::mobs::ee::effects_clear_keeping(found, &cause, &kept));
            }
            for (effect, (ambient, visible)) in list.iter().zip(&flags) {
                mob_step(game, EffectStep::AfterRemove, mob, *effect, *ambient, *visible);
            }
        }
        HeldEffect::Hit { mob, amount, source } => {
            if hit_asked(game, mob, amount, source)
                && let Some(found) = mob_mut(game, mob)
            {
                found.damage(amount, source);
            }
        }
        HeldEffect::Gone { mob, effect, ambient, visible } => {
            mob_step(game, EffectStep::AfterRemove, mob, effect, ambient, visible);
        }
        HeldEffect::Ending { mob, effect } => {
            let (ambient, visible) = flags_of(game, mob, effect.0);
            mob_step(game, EffectStep::BeforeRemove, mob, effect, ambient, visible);
            if let Some(found) = mob_mut(game, mob) {
                hub::passing_effects(|| found.remove_effect(effect.0));
            }
            mob_step(game, EffectStep::AfterRemove, mob, effect, ambient, visible);
        }
    }
}

/// What's waited for the game goes through. A change for a mob not in the list yet, one being made, gets
/// a few more turns: it's on its way in.
pub(in crate::game) fn effects_now(game: &mut Game<'_>) {
    let missing = MISSING.with(|missing| std::mem::take(&mut *missing.borrow_mut()));
    for (tries, change) in missing {
        if game.mobs.iter().any(|m| m.id == held_mob(&change)) {
            run_held(game, change);
        } else if tries < 3 {
            MISSING.with(|missing| missing.borrow_mut().push((tries + 1, change)));
        }
    }
    loop {
        landed_now(game);
        let held = hub::take_held();
        if held.is_empty() {
            return;
        }
        for change in held {
            if game.mobs.iter().any(|m| m.id == held_mob(&change)) {
                run_held(game, change);
            } else {
                MISSING.with(|missing| missing.borrow_mut().push((0, change)));
            }
        }
    }
}

/// Whether a mob's coming tick ends one of its effects: `BEFORE_REMOVE` is due first, while the game can be had.
pub(in crate::game) fn effects_ending(mob: &Mob) -> bool {
    hub::effects_heard() && mob.extra.effects.iter().any(|effect| effect.1 != -1 && effect.1 <= 1)
}

/// `tickEffects` as it's about to take what's run out, ahead of the mob's tick, for the game to be in hand.
pub(in crate::game) fn effects_told(game: &mut Game<'_>, id: u32) {
    let Some(list) = game.mobs.iter().find(|m| m.id == id).map(|m| m.extra.effects.clone()) else {
        return;
    };
    for effect in list.into_iter().filter(|effect| effect.1 != -1 && effect.1 <= 1) {
        let (ambient, visible) = flags_of(game, id, effect.0);
        hub::effect_told(id, effect.0);
        mob_step(game, EffectStep::BeforeRemove, id, effect, ambient, visible);
    }
    effects_now(game);
}

/// A mob's turn in its tick has left something for the game.
pub(in crate::game) fn turn_waits(mob: &Mob) -> bool {
    hub::effects_waiting() || hub::mob_sleep_waiting() || mob.extra.death_asked.is_some() || LANDED.with(|landed| !landed.borrow().is_empty())
}

/// A hit on a mob before it lands: no plugin event asks here (`EntityDamageEvent` is the ev_entity rows'), so it goes on.
pub(in crate::game) fn hit_asked(_game: &mut Game<'_>, _id: u32, _amount: f32, _source: horizon_client::mobs::Damage) -> bool {
    true
}

/// The same for the player in place.
pub(in crate::game) fn player_hit_asked(_game: &mut Game<'_>, _damage_type: &str, _causing: i32, _direct: i32, _amount: f32) -> bool {
    true
}

/// `apply_effect_action`'s hit, before the player is taken in hand.
pub(in crate::game) fn effect_hit_asked(game: &mut Game<'_>, action: &horizon_client::effects::EffectAction) -> bool {
    use horizon_client::effects::EffectAction;
    match action {
        EffectAction::Hurt(source, amount) => player_hit_asked(game, source, 0, 0, *amount),
        EffectAction::Poison => player_hit_asked(game, "minecraft:magic", 0, 0, 1.0),
        _ => true,
    }
}

/// Between two mobs' turns, the mobs back in the game.
pub(in crate::game) fn turn_over(game: &mut Game<'_>) {
    effects_now(game);
    deaths_asked(game);
    effects_now(game);
    mob_sleep_now(game);
}

/// A mob's sleep: nothing hears it, so the hub keeps no notes; any left go.
pub(in crate::game) fn mob_sleep_now(_game: &mut Game<'_>) {
    let _ = hub::take_mob_sleep();
}

/// A step of the player's effect: as a mob's, nothing hears it.
fn player_step(_game: &mut Game<'_>, _step: EffectStep, _effect: &EffectInstance) -> bool {
    true
}

/// `canBeAffected`, then `addEffect` as it looks for one already there.
pub(in crate::game) fn player_adding(game: &mut Game<'_>, effect: &EffectInstance) -> bool {
    if !player_step(game, EffectStep::AllowAdd, effect) {
        return false;
    }
    player_step(game, EffectStep::BeforeAdd, effect);
    true
}

/// `onEffectAdded`: it wasn't there before.
pub(in crate::game) fn player_added(game: &mut Game<'_>, effect: &EffectInstance) {
    player_step(game, EffectStep::AfterAdd, effect);
}

/// `LivingEntity.removeEffect`: true if it went.
pub(in crate::game) fn player_remove(game: &mut Game<'_>, id: &str, cause: &str) -> bool {
    let Some(effect) = game.potions.effects.get(id).cloned() else {
        return false;
    };
    if !player_step(game, EffectStep::AllowRemove, &effect) {
        return false;
    }
    player_step(game, EffectStep::BeforeRemove, &effect);
    if super::paper_fam_ev_entity::effect_gone(game, id, cause, "REMOVED") {
        return false;
    }
    game.potions.effects.list.retain(|held| held.id != id);
    game.apply_effect_status();
    player_step(game, EffectStep::AfterRemove, &effect);
    true
}

/// `LivingEntity.removeAllEffects`, in vanilla's order: all are told they're going, each is asked, and all are told
/// they went, those a listener kept too.
pub(in crate::game) fn player_clear(game: &mut Game<'_>, cause: &str) {
    if !heard() {
        return super::paper_fam_ev_entity::effects_clear(game, cause);
    }
    let list = game.potions.effects.list.clone();
    for effect in &list {
        player_step(game, EffectStep::BeforeRemove, effect);
    }
    let kept: Vec<EffectInstance> = list.iter().filter(|effect| !player_step(game, EffectStep::AllowRemove, effect)).cloned().collect();
    super::paper_fam_ev_entity::effects_clear(game, cause);
    for effect in kept {
        if game.potions.effects.get(effect.id).is_none() {
            game.potions.effects.list.push(effect);
        }
    }
    game.apply_effect_status();
    for effect in &list {
        player_step(game, EffectStep::AfterRemove, effect);
    }
}

/// The effects a player had as this effect tick began.
static TICKING: Mutex<Vec<EffectInstance>> = Mutex::new(Vec::new());

/// `tickEffects`, before the list is taken out of the game to tick.
pub(in crate::game) fn effects_expiring(game: &mut Game<'_>) {
    if !heard() || game.player.as_ref().is_none_or(|player| player.health.dead_or_dying()) {
        return;
    }
    *TICKING.lock().unwrap_or_else(|e| e.into_inner()) = game.potions.effects.list.clone();
}

/// `tickEffects` as its iterator is about to take this one, whatever ended it. The list is out of the game while it
/// ticks: for the listener it's back as it stood.
pub(in crate::game) fn effect_ending(game: &mut Game<'_>, effect: &EffectInstance) {
    let had = TICKING.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if had.is_empty() {
        return;
    }
    game.potions.effects.list = had;
    player_step(game, EffectStep::BeforeRemove, effect);
    game.potions.effects.list.clear();
}

/// `onEffectsRemoved` for what the tick took.
pub(in crate::game) fn effects_ticked(game: &mut Game<'_>) {
    let had = std::mem::take(&mut *TICKING.lock().unwrap_or_else(|e| e.into_inner()));
    for effect in had {
        if game.potions.effects.get(effect.id).is_none() {
            player_step(game, EffectStep::AfterRemove, &effect);
        }
    }
}

/// `canBeAffected` where the game asks it before it adds: a mob the game turns away never reaches `add_effect`, and
/// The plugins' say comes before that answer.
pub(in crate::game) fn mob_takes(tags: &std::collections::HashMap<String, std::collections::HashSet<String>>, mob: &Mob, effect: &str, ticks: i32, amplifier: i32) -> bool {
    let able = super::potions::mob_can_be_affected(tags, &mob.kind.id, effect);
    if !able && let Some(name) = super::potions::mob_effect_id(effect) {
        hub::mob_asked(mob, (name, ticks, amplifier));
    }
    able
}

/// `canBeAffected` itself, for a cloud choosing who's in it: `AreaEffectCloud.tick` asks, and `addEffect` asks again.
pub(in crate::game) fn cloud_affects(tags: &std::collections::HashMap<String, std::collections::HashSet<String>>, mob: &Mob, effect: &str, ticks: i32, amplifier: i32) -> bool {
    if let Some(name) = super::potions::mob_effect_id(effect) {
        hub::mob_asked(mob, (name, ticks, amplifier));
    }
    super::potions::mob_can_be_affected(tags, &mob.kind.id, effect)
}

/// Mobs a hit has killed, now: `LivingEntity.die` runs inside the hit on Java. Here the hit has no `Game`, so this runs
/// as soon as the work that dealt it is done.
pub(in crate::game) fn deaths_now(game: &mut Game<'_>) {
    effects_now(game);
    if !hub::take_deaths() {
        return;
    }
    deaths_asked(game);
    effects_now(game);
    let killed: Vec<Mob> = game.mobs.iter().filter(|m| !m.alive() && !m.removed && !m.extra.death_refused && m.extra.death_asked.is_none() && !game.died.contains(&m.id)).cloned().collect();
    for m in killed {
        game.died.insert(m.id);
        game.mob_died(&m);
    }
}

// ---- Hits and deaths ----

/// A hit that got past `ALLOW_DAMAGE`, until it lands.
#[derive(Clone)]
struct Coming {
    entity: i32,
    kind: String,
    damage_type: String,
    causing: i32,
    direct: i32,
    base: f32,
}

thread_local! {
    /// A hit can start another, a thorn's: the innermost is last.
    static HITS: RefCell<Vec<Coming>> = const { RefCell::new(Vec::new()) };
    /// Hits that killed, until the death is told.
    static FATAL: RefCell<Vec<Coming>> = const { RefCell::new(Vec::new()) };
    /// The player whose death comes next, by entity id.
    static DYING: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
    /// A dragon that has just died, with what killed it.
    static DRAGON_DOWN: RefCell<Option<Coming>> = const { RefCell::new(None) };
}

pub(in crate::game) fn hit_coming(entity: i32, kind: &str, damage_type: &str, causing: i32, direct: i32, base: f32) {
    HITS.with(|hits| {
        let mut hits = hits.borrow_mut();
        // One that never landed, a hit the frames stopped, is forgotten here.
        hits.retain(|hit| hit.entity != entity);
        if hits.len() >= 32 {
            hits.remove(0);
        }
        hits.push(Coming { entity, kind: kind.to_owned(), damage_type: damage_type.to_owned(), causing, direct, base });
    });
}

fn hit_of(entity: i32) -> Option<Coming> {
    HITS.with(|hits| hits.borrow().iter().rev().find(|hit| hit.entity == entity).cloned())
}

fn hit_landed(entity: i32, taken: f32, blocked: bool, alive: bool) {
    let Some(hit) = HITS.with(|hits| {
        let mut hits = hits.borrow_mut();
        let at = hits.iter().rposition(|hit| hit.entity == entity)?;
        Some(hits.remove(at))
    }) else {
        return;
    };
    if alive {
        hit_lived(hit, taken, blocked);
    } else {
        FATAL.with(|fatal| {
            let mut fatal = fatal.borrow_mut();
            fatal.retain(|known| known.entity != entity);
            if fatal.len() >= 32 {
                fatal.remove(0);
            }
            fatal.push(hit);
        });
    }
}

thread_local! {
    /// Hits something lived through, for `AFTER_DAMAGE` with the game.
    static LANDED: RefCell<Vec<(Coming, f32, bool)>> = const { RefCell::new(Vec::new()) };
}

/// A hit something lived through: no plugin event hears the end of `hurtServer` here.
fn hit_lived(_hit: Coming, _taken: f32, _blocked: bool) {}

/// The end of `hurtServer`, the game back in hand: nothing waits for it.
fn landed_now(_game: &mut Game<'_>) {
    LANDED.with(|landed| landed.borrow_mut().clear());
}

fn take_fatal(entity: i32) -> Option<Coming> {
    FATAL.with(|fatal| {
        let mut fatal = fatal.borrow_mut();
        let at = fatal.iter().position(|hit| hit.entity == entity)?;
        Some(fatal.remove(at))
    })
}

fn mob_may_die(mob: &mut Mob, taken: f32, bypass: bool) {
    let hit = hit_of(mob.id as i32);
    mob.extra.death_asked = Some(hub::DeathAsked {
        damage_type: hit.as_ref().map_or("minecraft:generic".to_owned(), |hit| hit.damage_type.clone()),
        causing: hit.as_ref().map_or(0, |hit| hit.causing),
        direct: hit.as_ref().map_or(0, |hit| hit.direct),
        amount: taken,
        bypass,
    });
}

/// The rest of the hit that left a mob at no health: `isDeadOrDying` with `ALLOW_DEATH`, the totem, and for what
/// lives through them the end of `hurtServer`. `die` comes after, for those that don't.
fn deaths_asked(game: &mut Game<'_>) {
    let asked: Vec<u32> = game.mobs.iter().filter(|m| m.extra.death_asked.is_some()).map(|m| m.id).collect();
    for id in asked {
        let Some(asked) = mob_mut(game, id).and_then(|m| m.extra.death_asked.take()) else {
            continue;
        };
        // No plugin event refuses a mob's death here: it goes ahead, the totem as the game has it.
        let allowed = true;
        let Some(mob) = mob_mut(game, id) else {
            continue;
        };
        if allowed {
            mob.health = 0.0;
            horizon_client::mobs::ee::totem(mob, asked.bypass);
        } else {
            mob.extra.death_refused = mob.health <= 0.0;
        }
        if mob.alive() {
            mob.death_time = 0;
            mob.extra.death_refused = false;
            if let Some(hit) = take_fatal(id as i32) {
                hit_lived(hit, asked.amount, false);
            }
        }
    }
}

/// The dragon's fatal hit, with the dragon lifted out of the game for it and the game lent: as a mob's, with its
/// health where a listener can raise it.
pub(in crate::game) fn dragon_may_die(_entity: i32, _health: &mut f32, _taken: f32) -> bool {
    true
}

/// What `hurt` said in `Dragon`: its end, where a mob's is.
pub(in crate::game) fn dragon_landed(entity: i32, taken: f32, alive: bool) {
    hit_landed(entity, taken, false, alive);
    if !alive {
        let fatal = take_fatal(entity);
        DRAGON_DOWN.with(|down| *down.borrow_mut() = fatal.or(Some(Coming { entity, kind: String::new(), damage_type: String::new(), causing: 0, direct: 0, base: 0.0 })));
    }
}

/// `die` for the dragon, with the game back in hand.
pub(in crate::game) fn dragon_after(game: &mut Game<'_>) {
    let Some(hit) = DRAGON_DOWN.with(|down| down.borrow_mut().take()) else {
        return;
    };
    const KIND: &str = "minecraft:ender_dragon";
    let damage_type = (!hit.damage_type.is_empty()).then_some(hit.damage_type.as_str());
    let death = living::Death { entity: hit.entity, kind: KIND, player: None, mob: None, damage_type, causing: hit.causing, direct: hit.direct, killer: guest_uuid(game, hit.causing), cause: None };
    living::after_mob_death(game, &death);
}

/// The player with this entity id, if one has it.
fn guest_uuid(game: &Game<'_>, entity: i32) -> Option<u128> {
    if entity == 0 {
        return None;
    }
    game.lan.as_ref()?.hub.guests.iter().find(|guest| guest.entity_id == entity).map(|guest| guest.uuid)
}

/// `damageSource.getEntity()` of a mob's last hit.
fn mob_attacker(game: &Game<'_>, m: &Mob, killer: Option<u128>) -> Option<(i32, Option<u128>)> {
    match m.last_damage?.0.attacker? {
        Who::Mob(id) => Some((id as i32, guest_uuid(game, id as i32))),
        Who::Player => Some((0, killer.or_else(|| super::paper_fam_entity::killer_uuid(game, m)))),
    }
}

/// `LivingEntity.die` as it calls `killedEntity`, before the loot.
pub(in crate::game) fn mob_dying(game: &mut Game<'_>, m: &Mob) {
    let Some((attacker, attacker_player)) = mob_attacker(game, m, None) else {
        return;
    };
    let Some((source, _)) = m.last_damage else {
        return;
    };
}

/// `AFTER_DEATH`, as `die` ends.
pub(in crate::game) fn mob_dead(game: &mut Game<'_>, m: &Mob, killer: Option<u128>) {
    let source = m.last_damage.map(|(source, _)| source);
    let causing = match source.and_then(|source| source.attacker) {
        Some(Who::Mob(id)) => id as i32,
        _ => 0,
    };
    let melee = source.is_some_and(|source| super::creatures::melee(source.kind));
    let killer = killer.or_else(|| mob_attacker(game, m, None).and_then(|(_, player)| player));
    let death = living::Death {
        entity: m.id as i32,
        kind: &m.kind.id,
        player: None,
        mob: Some(m),
        damage_type: source.map(super::creatures::source_damage_type),
        causing,
        direct: if melee { causing } else { 0 },
        killer,
        cause: None,
    };
    living::after_mob_death(game, &death);
}

/// Says whose death comes next: the game's own record of it has only a UUID.
pub(in crate::game) fn dying_player(entity: i32) {
    DYING.set(entity);
}

/// `ServerPlayer.die`: `killedEntity` for the player, then `AFTER_DEATH`.
pub(in crate::game) fn player_dead(game: &mut Game<'_>, player: u128, cause: Option<&str>) {
    let entity = DYING.get();
    let hit = take_fatal(entity);
    let causing = hit.as_ref().map_or(0, |hit| hit.causing);
    let death = living::Death {
        entity,
        kind: "minecraft:player",
        player: Some(player),
        mob: None,
        damage_type: hit.as_ref().map(|hit| hit.damage_type.as_str()),
        causing,
        direct: hit.as_ref().map_or(0, |hit| hit.direct),
        killer: guest_uuid(game, causing),
        cause,
    };
    living::after_player_death(game, &death);
}

// ---- Respawning and changing level ----

thread_local! {
    /// A guest on their way back from the End: `respawn(alive = true)`.
    static RETURNING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn level_id(dim: &str) -> String {
    format!("minecraft:{}", dim.trim_start_matches("minecraft:"))
}

/// `PlayerList.respawn` as it ends.
pub(in crate::game) fn player_respawned(game: &mut Game<'_>, player: u128, name: &str, alive: bool, from: &str, to: &str) {
    players::after_respawn(game, player, name, alive);
    let (from, to) = (level_id(from), level_id(to));
    if from != to {
        level_change::after_player_change_level(game, player, name, &from, &to);
    }
}

/// `triggerDimensionChangeTriggers` as it ends, for a guest. The way back from the End is a respawn on Java, and comes
/// by here on Horizon.
pub(in crate::game) fn guest_changed_level(game: &mut Game<'_>, host: &LanHost, i: usize, from: &str, to: &str) {
    let (player, name) = (host.hub.guests[i].uuid, host.hub.guests[i].name.clone());
    if RETURNING.take() {
        return player_respawned(game, player, &name, true, from, to);
    }
    let (from, to) = (level_id(from), level_id(to));
    // The hub is out here, so the player can't be asked about: listeners get them once it's back.
    super::paper_fam_core::when_hub_back(Box::new(move |game| level_change::after_player_change_level(game, player, &name, &from, &to)));
}

/// A guest leaves the End's credits for the overworld.
pub(in crate::game) fn guest_returning(game: &mut Game<'_>, host: &LanHost, conn: u32) {
    let Some(guest) = host.hub.guest(conn) else {
        return;
    };
    let _ = (game, guest);
    RETURNING.set(true);
}

/// The world's own player: `respawn` as it ends. A level change on the way was told by `enter_dimension`.
pub(in crate::game) fn host_respawned(game: &mut Game<'_>) {
    let (player, name) = (game.native_acting_id(), game.acting_player_name().to_owned());
    players::after_respawn(game, player, &name, false);
}

/// The world's own player, in a new level.
pub(in crate::game) fn host_changed_level(game: &mut Game<'_>, from: &str, to: &str) {
    let (player, name) = (game.native_acting_id(), game.acting_player_name().to_owned());
    let (from, to) = (level_id(from), level_id(to));
    level_change::after_player_change_level(game, player, &name, &from, &to);
}

/// Entities on their way through a portal: where they left, until they join where they're going.
struct Travelling {
    entity: u32,
    /// The riders' old ids, in the order they're read back in.
    riders: Vec<u32>,
    from: String,
    to: String,
    at: [f64; 3],
}

static TRAVELLING: Mutex<Vec<Travelling>> = Mutex::new(Vec::new());
const LEFT: &str = "horizon:portal_from";

/// Says on the entity's saved tree what it was: it waits in that form until its level is there.
pub(in crate::game) fn portal_left(tag: &mut horizon_world::nbt::Tag, entity: u32, from: &str, riders: &[u32]) {
    use horizon_world::nbt::Tag;
    if let Tag::Compound(saved) = tag {
        let riders = Tag::List(riders.iter().map(|&id| Tag::Int(id as i32)).collect());
        saved.insert(LEFT.into(), Tag::compound([("id", Tag::Int(entity as i32)), ("level", Tag::String(level_id(from).as_str().into())), ("riders", riders)]));
    }
}

/// `resolve_arrivals` has its place: it joins the level when the tree is next read.
pub(in crate::game) fn portal_arriving(tag: &mut horizon_world::nbt::Tag, to: &str, at: [f64; 3]) {
    use horizon_world::nbt::Tag;
    let Tag::Compound(saved) = tag else {
        return;
    };
    let Some(left) = saved.remove(&LEFT.into()) else {
        return;
    };
    let (Some(Tag::Int(entity)), Some(Tag::String(from))) = (left.get("id"), left.get("level")) else {
        return;
    };
    let from = String::from_utf16_lossy(&from.0);
    // `setCanCreatePortal(false)` with no portal: it's put back, and in Java it never left.
    if from == level_id(to) {
        return;
    }
    let riders = match left.get("riders") {
        Some(Tag::List(list)) => list.iter().filter_map(|id| if let Tag::Int(id) = id { Some(*id as u32) } else { None }).collect(),
        _ => Vec::new(),
    };
    let mut travelling = TRAVELLING.lock().unwrap_or_else(|e| e.into_inner());
    if travelling.len() >= 256 {
        travelling.remove(0);
    }
    travelling.push(Travelling { entity: *entity as u32, riders, from, to: level_id(to), at });
}

/// The mobs on `id`, as they're written to its saved tree and read back.
pub(in crate::game) fn riders_of(game: &Game<'_>, id: u32) -> Vec<u32> {
    fn walk(game: &Game<'_>, id: u32, out: &mut Vec<u32>) {
        for m in game.mobs.iter().filter(|m| m.extra.mount == Some(id) && m.alive()) {
            out.push(m.id);
            walk(game, m.id, out);
        }
    }
    let mut out = Vec::new();
    walk(game, id, &mut out);
    out
}

fn take_travelling(at: [f64; 3]) -> Option<Travelling> {
    let mut travelling = TRAVELLING.lock().unwrap_or_else(|e| e.into_inner());
    let found = travelling.iter().position(|row| (0..3).all(|i| (row.at[i] - at[i]).abs() < 1.0e-3))?;
    Some(travelling.remove(found))
}

/// A player's vehicle comes along under the same id: `Entity.teleportCrossDimension` on it.
pub(in crate::game) fn vehicle_arrived(game: &mut Game<'_>, id: u32, from: &str) {
    let Some(target) = game.level_entity(id as i32) else {
        return;
    };
    let kind = game.target_type(target).to_string();
    let (from, to) = (level_id(from), level_id(game.dim.id()));
    level_change::after_entity_change_level(game, id, id, &kind, &from, &to);
}

/// The entity being read off its saved tree. Only an item and an orb tell themselves on the way:
/// for the rest, what joined shows once the read is done.
struct Loading {
    before: u32,
    mobs: usize,
    kind: String,
    at: [f64; 3],
    from_disk: bool,
    moved: bool,
}

thread_local! {
    static LOADING: RefCell<Option<Loading>> = const { RefCell::new(None) };
}

pub(in crate::game) fn loading(game: &Game<'_>, tag: &horizon_world::nbt::Tag, kind: &str, from_disk: bool, moved: bool) {
    let at = horizon_client::entity_nbt::position(tag).map_or([0.0; 3], |p| [p.x, p.y, p.z]);
    LOADING.set(Some(Loading { before: game.next_entity, mobs: game.mobs.len(), kind: kind.to_owned(), at, from_disk, moved }));
}

fn joins(game: &mut Game<'_>, id: u32, kind: &str, at: [f64; 3], from_disk: bool) -> bool {
    let joined = match game.mobs.iter().position(|m| m.id == id) {
        Some(index) if !from_disk => crate::game::paper_fam_ev_entity::mob_joins(game, index),
        _ => game.native_entity_joins(id, kind, at, from_disk),
    };
    if !joined {
        game.mobs.retain(|m| m.id != id);
        game.boats.retain(|b| b.id != id);
        game.carts.retain(|c| c.id != id);
        game.armor_stands.retain(|a| a.id != id);
        game.item_frames.retain(|f| f.id != id);
        game.paintings.retain(|p| p.id != id);
        game.others.retain(|o| o.id != id);
        game.crystals.retain(|c| c.id != id);
        game.creatures.projectiles.retain(|p| p.id != id);
        game.forget_entity(id);
    }
    joined
}

/// `PersistentEntitySectionManager.addEntity` for what was just read, with its riders, and for a
/// traveller `Entity.teleport`'s end: the riders', being teleported inside their vehicle's, first.
/// A `/summon` tells its own, once it has put the entity where it asked.
pub(in crate::game) fn load_told(game: &mut Game<'_>) {
    let Some(row) = LOADING.take() else {
        return;
    };
    if row.moved || matches!(&*row.kind, "minecraft:item" | "minecraft:experience_orb") {
        return;
    }
    let travelled = if row.from_disk { None } else { take_travelling(row.at) };
    if !row.from_disk && travelled.is_none() {
        return;
    }
    let mob_root = game.res.mob_types().contains_key(&row.kind);
    let new: Vec<(u32, String, [f64; 3])> = game.mobs.iter().skip(row.mobs).map(|m| (m.id, m.kind.id.to_string(), [m.position.x, m.position.y, m.position.z])).collect();
    let root = if mob_root {
        new.first().map(|m| m.0)
    } else {
        (game.next_entity > row.before).then_some(row.before + 1)
    };
    let root = root.filter(|&id| joins(game, id, &row.kind, row.at, row.from_disk));
    let riders: Vec<(u32, String)> = new
        .iter()
        .skip(usize::from(mob_root))
        .filter(|m| joins(game, m.0, &m.1, m.2, row.from_disk))
        .map(|m| (m.0, m.1.clone()))
        .collect();
    let Some(travelled) = travelled else {
        return;
    };
    for (&old, (id, kind)) in travelled.riders.iter().zip(&riders) {
        level_change::after_entity_change_level(game, old, *id, kind, &travelled.from, &travelled.to);
    }
    if let Some(id) = root {
        level_change::after_entity_change_level(game, travelled.entity, id, &row.kind, &travelled.from, &travelled.to);
    }
}

/// `Entity.teleport`, as the new entity stands in its level.
pub(in crate::game) fn entity_arrived(game: &mut Game<'_>, entity: u32, kind: &str, pos: [f64; 3]) {
    if let Some(row) = take_travelling(pos) {
        level_change::after_entity_change_level(game, row.entity, entity, kind, &row.from, &row.to);
    }
}

// ---- Using a block ----

/// What's left of `ServerPlayerGameMode.useItemOn`'s block part.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::game) enum BlockStep {
    /// The block's own step runs.
    Block,
    /// A listener said `TRY_WITH_EMPTY_HAND`: the block's `useItemOn` is skipped, not its `useWithoutItem`.
    EmptyHand,
    /// The block is done with and the click goes on to the item.
    Skip,
    /// The click ends here.
    Done,
}

struct Turn {
    click: blocks::Click,
    /// `USE_WITHOUT_ITEM`'s answer, once asked.
    answer: Option<blocks::Use>,
    /// A listener skipped `useItemOn`, so nothing stands before `useWithoutItem`.
    forced: bool,
}

thread_local! {
    /// The hand's turn under way: the block's steps are spread over the engine, and one asks once.
    static CLICK: RefCell<Option<Turn>> = const { RefCell::new(None) };
}

fn consumes(answer: blocks::Use) -> bool {
    matches!(answer, blocks::Use::Success | blocks::Use::SuccessServer | blocks::Use::Consume)
}

/// `BlockState.useItemOn`'s HEAD, for either hand and an empty one too.
pub(in crate::game) fn use_item_on(game: &mut Game<'_>, hit: &crate::game::Hit) -> BlockStep {
    use blocks::Use;
    let click = blocks::Click {
        pos: hit.pos,
        face: hit.face,
        point: hit.point,
        off_hand: game.hand == Hand::Off,
    };
    // `usedItemStack`: the advancement sees the stack from before the listeners.
    let used = game.held().cloned().unwrap_or_else(|| ItemStack::new("minecraft:air", 0));
    let mut answer = Use::Default;
    if blocks::heard() && let Some(player) = game.native_acting_player() {
        answer = blocks::use_item_on(game, &player, &used, &click);
    }
    CLICK.set(Some(Turn { click, answer: None, forced: answer == Use::TryEmptyHand }));
    match answer {
        Use::Default => BlockStep::Block,
        Use::TryEmptyHand => BlockStep::EmptyHand,
        Use::Fail | Use::Pass => BlockStep::Skip,
        _ => {
            game.trigger_block("minecraft:item_used_on_block", hit.pos, &used);
            if answer != Use::Consume {
                game.swing_arm();
            }
            BlockStep::Done
        }
    }
}

/// `BlockState.useWithoutItem`'s HEAD, where a block's step sits inside one of the engine's: whether
/// the block's own part must not run. Only the main hand gets that far. Outside a click the caller
/// is something else using the block, and nothing is asked.
pub(in crate::game) fn empty_hand_taken(game: &mut Game<'_>) -> bool {
    use blocks::Use;
    if game.hand != Hand::Main {
        return true;
    }
    let Some((click, asked)) = CLICK.with_borrow(|turn| turn.as_ref().map(|t| (t.click, t.answer))) else {
        return false;
    };
    let answer = asked.unwrap_or_else(|| {
        let mut answer = Use::Default;
        if blocks::heard() && let Some(player) = game.native_acting_player() {
            answer = blocks::use_without_item(game, &player, &click);
        }
        CLICK.with_borrow_mut(|turn| {
            if let Some(turn) = turn {
                turn.answer = Some(answer);
            }
        });
        answer
    });
    answer != Use::Default
}

/// The block's `useItemOn` has nothing to say here, in the engine's terms: what's next of the
/// `TRY_WITH_EMPTY_HAND` branch.
pub(in crate::game) fn empty_hand_at(game: &mut Game<'_>, hit: &crate::game::Hit, held: Option<&ItemStack>) -> BlockStep {
    if game.hand != Hand::Main {
        return BlockStep::Skip;
    }
    let (asked, forced) = CLICK.with_borrow(|turn| turn.as_ref().map_or((None, false), |t| (t.answer, t.forced)));
    if asked.is_none() && !forced && item_on_answered(game, hit, held) {
        return BlockStep::Skip;
    }
    if !empty_hand_taken(game) {
        return BlockStep::Block;
    }
    let answer = CLICK.with_borrow(|turn| turn.as_ref().and_then(|t| t.answer)).unwrap_or(blocks::Use::Default);
    if !consumes(answer) {
        return BlockStep::Skip;
    }
    game.trigger_default_block_use(hit.pos);
    if answer != blocks::Use::Consume {
        game.swing_arm();
    }
    BlockStep::Done
}

/// The blocks whose `useItemOn` doesn't answer `TRY_WITH_EMPTY_HAND` here though the engine's part
/// of them, up to now, took nothing: they're `PASS`, or the engine does them with the item's use.
fn item_on_answered(game: &Game<'_>, hit: &crate::game::Hit, held: Option<&ItemStack>) -> bool {
    let facts = game.res.facts().get(game.world.block(hit.pos[0], hit.pos[1], hit.pos[2]));
    let name = &*facts.name;
    if name.ends_with("_shelf") {
        return true;
    }
    match name {
        "minecraft:redstone_ore" | "minecraft:deepslate_redstone_ore" => true,
        "minecraft:tnt" => held.is_some_and(|s| matches!(&*s.item, "minecraft:flint_and_steel" | "minecraft:fire_charge")),
        "minecraft:campfire" | "minecraft:soul_campfire" => {
            held.is_some_and(|s| game.recipes().cook("minecraft:campfire_cooking", s).is_some())
        }
        "minecraft:lectern" => {
            let key = (hit.pos[0], hit.pos[1], hit.pos[2]);
            held.is_none()
                && !matches!(
                    game.world.block_entities.get(&key),
                    Some(crate::game::BlockEntity::Lectern(l)) if l.book.is_some()
                )
        }
        "minecraft:chiseled_bookshelf" => held.is_some_and(|s| {
            game.res.items().item_tags.get("minecraft:bookshelf_books").is_some_and(|m| m.contains(&*s.item))
        }),
        _ => false,
    }
}

/// `FenceBlock.useWithoutItem` for a guest's click, which the engine does with the leads.
pub(in crate::game) fn knot_used(game: &mut Game<'_>, hit: &crate::game::Hit) -> bool {
    if !matches!(game.guest_use, Some(crate::game::GuestUse::OnBlock(_))) {
        return false;
    }
    let facts = game.res.facts().get(game.world.block(hit.pos[0], hit.pos[1], hit.pos[2]));
    let fence = game.res.items().block_tags.get("minecraft:fences").is_some_and(|s| s.contains(&*facts.name));
    fence && !empty_hand_taken(game) && game.interact_knot()
}

pub(in crate::game) fn click_over() {
    CLICK.set(None);
}

// ---- Mobs turning into others ----

type Params = horizon_client::mobs::conversion::Params;

thread_local! {
    /// `Mob.convertTo` under way, where the new mob comes out of the engine finished.
    static CONVERTING: RefCell<Option<(Mob, Params)>> = const { RefCell::new(None) };
}

fn conversion_told(game: &mut Game<'_>, old: &Mob, params: Params, split: bool, new: u32) {
    effects_now(game);
    let Some(new_kind) = game.mobs.iter().find(|m| m.id == new).map(|m| m.kind.id.to_string()) else {
        return;
    };
    let told = living::Conversion {
        old,
        new,
        new_kind: &new_kind,
        keep_equipment: params.keep_equipment,
        preserve_can_pick_up_loot: params.preserve_can_pick_up_loot,
        split,
    };
    living::mob_conversion(game, &told);
}

pub(in crate::game) fn converting(from: &Mob, params: Params) {
    CONVERTING.set(Some((from.clone(), params)));
}

/// Where a mob joins: the new one of a `converting`, before the level has it.
pub(in crate::game) fn on_converted(game: &mut Game<'_>, index: usize) {
    effects_now(game);
    let (Some((old, params)), Some(id)) = (CONVERTING.take(), game.mobs.get(index).map(|m| m.id)) else {
        return;
    };
    conversion_told(game, &old, params, false, id);
}

/// A conversion that's made through `spawn_mob` and set up after: the new mob stays out until
/// `converted`, as `convertTo` runs its consumer, then the event, then `addFreshEntity`.
pub(in crate::game) fn conversion_ahead() {
    // Effects made in setting a mob up wait for it to be in the list.
    crate::game::paper_fam_ev_entity::hold_join(true);
}

pub(in crate::game) fn conversion_off() {
    crate::game::paper_fam_ev_entity::hold_join(false);
}

pub(in crate::game) fn converted(game: &mut Game<'_>, old: &Mob, params: Params, split: bool, id: u32) {
    use crate::game::paper_fam_ev_entity as ev;
    conversion_told(game, old, params, split, id);
    ev::hold_join(false);
    if let Some(mut i) = game.mobs.iter().position(|m| m.id == id) {
        while i < game.mobs.len() {
            if ev::mob_joins(game, i) {
                i += 1;
            } else {
                game.mobs.remove(i);
            }
        }
    }
}

/// A bolt's conversion, whose joining `zapped` sees to after asking.
pub(in crate::game) fn zap_converted(game: &mut Game<'_>, old: &Mob, params: Params, id: u32) {
    conversion_told(game, old, params, false, id);
}

// ---- A command sending an entity to another level ----

const SENT: &str = "horizon:sent_to";

/// `TeleportCommand` on what isn't a player, for a level that doesn't hold it: true if it's on its way.
pub(in crate::game) fn entity_across(game: &mut Game<'_>, t: crate::game::commands::Target, level: crate::game::Dim, pos: [f64; 3], rotation: Option<[f32; 2]>) -> bool {
    use crate::game::commands::Target;
    let id = match t {
        Target::Mob(id) | Target::Item(id) | Target::Boat(id) | Target::Cart(id) => id,
        _ => return false,
    };
    if !game.in_target_level(t, |game| game.dim != level).unwrap_or(false) {
        return false;
    }
    let Some(pos) = crate::game::paper_fam_ev_entity::command_teleport(game, t, pos) else {
        return true;
    };
    game.in_target_level(t, |game| game.send_to_level(id, level, pos, rotation));
    true
}

/// Says on the saved tree where it lands: the plan's position is a block.
pub(in crate::game) fn sent_to(tag: &mut horizon_world::nbt::Tag, pos: [f64; 3], rotation: Option<[f32; 2]>) {
    use horizon_world::nbt::Tag;
    if let Tag::Compound(saved) = tag {
        let [yaw, pitch] = rotation.unwrap_or([f32::NAN, f32::NAN]);
        saved.insert(SENT.into(), Tag::List(vec![Tag::Double(pos[0].to_bits()), Tag::Double(pos[1].to_bits()), Tag::Double(pos[2].to_bits()), Tag::Float(yaw.to_bits()), Tag::Float(pitch.to_bits())]));
    }
}

/// `TeleportCommand.performTeleport`: where it lands, its fall over.
pub(in crate::game) fn sent_landing(tag: &mut horizon_world::nbt::Tag, at: horizon_core::geometry::Vec3, motion: horizon_core::geometry::Vec3, rotation: [f32; 2]) -> (horizon_core::geometry::Vec3, horizon_core::geometry::Vec3, [f32; 2]) {
    use horizon_world::nbt::Tag;
    let Tag::Compound(saved) = tag else {
        return (at, motion, rotation);
    };
    let Some(Tag::List(sent)) = saved.remove(&SENT.into()) else {
        return (at, motion, rotation);
    };
    let double = |index: usize| match sent.get(index) {
        Some(Tag::Double(bits)) => f64::from_bits(*bits),
        _ => 0.0,
    };
    let float = |index: usize, own: f32| match sent.get(index) {
        Some(Tag::Float(bits)) if !f32::from_bits(*bits).is_nan() => f32::from_bits(*bits),
        _ => own,
    };
    let landing = horizon_core::geometry::Vec3::new(double(0), double(1), double(2));
    (landing, horizon_core::geometry::Vec3::new(motion.x, 0.0, motion.z), [float(3, rotation[0]), float(4, rotation[1])])
}

// ---- A player's block break ----

/// What's in the block about to go.
pub(in crate::game) fn block_was(game: &Game<'_>, pos: [i32; 3]) -> crate::game::paper_events::player_block_break_events::Was {
    let entity = game.world.block_entities.get(&(pos[0], pos[1], pos[2])).cloned();
    let saved = if entity.is_none() { crate::game::spill::saved_entity(&game.world, pos) } else { None };
    crate::game::paper_events::player_block_break_events::Was { entity, saved }
}

/// `ServerPlayerGameMode.destroyBlock` at `Block.destroy`: removed, before the drops.
pub(in crate::game) fn block_broken(game: &mut Game<'_>, pos: [i32; 3], state: horizon_world::registry::BlockStateId, was: &crate::game::paper_events::player_block_break_events::Was) {
    if let Some(player) = game.native_acting_player() {
        let _ = (&player, pos, state, was);
    }
}
