# The entity events' needs of the engine (paper_entity_events.rs). Applied after the mixins and the tables before it.
GAME = 'horizon-client/src/game/'
MOBS = 'horizon-client/src/mobs/'
LIB = 'horizon-client/src/'
HUB = 'horizon_client::paper_hub'

EDITS = [
    # ---- ServerMobEffectEvents on mobs: `Mob` has what happens to its effects, and the hub takes it to the listeners.
    (LIB + 'mobs.rs', '''        if ee::effect_refused(self, effect, ticks, amplifier) {
            return;
        }
        if let Some(e) = self.extra.effects.iter_mut().find(|e| e.0 == effect) {''', '''        // `canBeAffected`: the plugins' say first, and the game refuses after.
        use crate::paper_hub::{EffectStep, mob_effect};
        if !mob_effect(EffectStep::AllowAdd, self, (effect, ticks, amplifier)) || !crate::paper_hub::mob_affected(&self.kind.id, effect) {
            return;
        }
        if ee::effect_refused(self, effect, ticks, amplifier) {
            return;
        }
        mob_effect(EffectStep::BeforeAdd, self, (effect, ticks, amplifier));
        if let Some(e) = self.extra.effects.iter_mut().find(|e| e.0 == effect) {'''),
    (LIB + 'mobs.rs', '''            self.extra.effects.push((effect, ticks, amplifier));
        }
    }
    pub fn speed_bonus(&self) -> f64 {''', '''            self.extra.effects.push((effect, ticks, amplifier));
            mob_effect(EffectStep::AfterAdd, self, (effect, ticks, amplifier));
        }
    }
    /// `LivingEntity.removeEffect`: true if it went.
    pub fn remove_effect(&mut self, effect: &str) -> bool {
        use crate::paper_hub::{EffectStep, mob_effect};
        let Some(found) = self.extra.effects.iter().find(|e| e.0 == effect).copied() else {
            return false;
        };
        if !mob_effect(EffectStep::AllowRemove, self, found) {
            return false;
        }
        mob_effect(EffectStep::BeforeRemove, self, found);
        self.extra.effects.retain(|e| e.0 != effect);
        self.extra.hidden_effects.retain(|h| *h != effect);
        self.extra.ambient_effects.retain(|h| *h != effect);
        mob_effect(EffectStep::AfterRemove, self, found);
        true
    }
    pub fn speed_bonus(&self) -> f64 {'''),
    (LIB + 'mobs.rs', '''        self.add_effect(effect, ticks, amplifier);
        let after = self
''', '''        self.add_effect(effect, ticks, amplifier);
        let after = self
'''),
    (LIB + 'mobs.rs', '''    pub fn effects_clear(m: &mut Mob, cause: &str) {
        if !hears("ee_effect") {
            return m.extra.effects.clear();
        }
''', '''    pub fn effects_clear(m: &mut Mob, cause: &str) {
        use crate::paper_hub::{EffectStep, mob_effect};
        let list = m.extra.effects.clone();
        for effect in &list {
            mob_effect(EffectStep::BeforeRemove, m, *effect);
        }
        let kept: Vec<_> = list.iter().copied().filter(|effect| !mob_effect(EffectStep::AllowRemove, m, *effect)).collect();
        effects_clear_all(m, cause);
        for effect in kept {
            if !m.extra.effects.iter().any(|e| e.0 == effect.0) {
                m.extra.effects.push(effect);
            }
        }
        let left = std::mem::take(&mut m.extra.effects);
        m.extra.hidden_effects.retain(|h| left.iter().any(|e| e.0 == *h));
        m.extra.ambient_effects.retain(|h| left.iter().any(|e| e.0 == *h));
        m.extra.effects = left;
        for effect in &list {
            mob_effect(EffectStep::AfterRemove, m, *effect);
        }
    }
    fn effects_clear_all(m: &mut Mob, cause: &str) {
        if !hears("ee_effect") {
            return m.extra.effects.clear();
        }
'''),
    (MOBS + 'ai.rs', '''    super::ee::effects_ending(m, &mut effects);
    effects.retain(|e| e.1 > 0 || e.1 == -1);
''', '''    super::ee::effects_ending(m, &mut effects);
    // `tickEffects`: told before the iterator takes it, with the mob's effects where a listener looks for them.
    use crate::paper_hub::{EffectStep, mob_effect};
    let ended: Vec<_> = effects.iter().copied().filter(|e| e.1 <= 0 && e.1 != -1).collect();
    if !ended.is_empty() {
        m.extra.effects = effects;
        for effect in &ended {
            mob_effect(EffectStep::BeforeRemove, m, *effect);
        }
        effects = std::mem::take(&mut m.extra.effects);
    }
    effects.retain(|e| e.1 > 0 || e.1 == -1);
'''),
    (MOBS + 'ai.rs', '''        .retain(|h| effects.iter().any(|e| e.0 == *h));
    m.extra.effects = effects;
}
''', '''        .retain(|h| effects.iter().any(|e| e.0 == *h));
    m.extra.effects = effects;
    for effect in &ended {
        mob_effect(EffectStep::AfterRemove, m, *effect);
    }
}
'''),
    (MOBS + 'trader.rs', '''                m.extra.effects.retain(|e| e.0 != INVISIBILITY);
                m.extra.effects.push((INVISIBILITY, 3600, 0));
''', '''                m.add_effect(INVISIBILITY, 3600, 0);
'''),
    (MOBS + 'zombie_villager.rs', '''    m.extra.effects.retain(|e| e.0 != "minecraft:weakness");
''', '''    m.remove_effect("minecraft:weakness");
'''),
    (GAME + 'commands/effects.rs', '''                            let before = m.extra.effects.len();
                            m.extra
                                .effects
                                .retain(|x| e.is_some_and(|e| x.0 != namespaced(e.id)));
                            let left = &m.extra.effects;
                            m.extra
                                .hidden_effects
                                .retain(|h| left.iter().any(|x| x.0 == *h));
                            m.extra
                                .ambient_effects
                                .retain(|h| left.iter().any(|x| x.0 == *h));
                            m.extra.effects.len() != before
''', '''                            match e {
                                Some(e) => m.remove_effect(namespaced(e.id)),
                                None => {
                                    let any = !m.extra.effects.is_empty();
                                    horizon_client::mobs::ee::effects_clear(m, "COMMAND");
                                    any
                                }
                            }
'''),
    # ---- The same on players, at the moment it happens.
    (GAME + 'potions.rs', '''                if crate::game::paper_events::on_effect_add(
                    game,
                    effect.id,
                    effect.amplifier,
                ) {''', '''                if crate::game::paper_entity_events::player_adding(game, effect) {'''),
    (GAME + 'potions.rs', '''        let new = self.potions.effects.get(id).is_none();
        let changed = self.potions.effects.add(effect);
''', '''        let new = self.potions.effects.get(id).is_none();
        let told = new.then(|| effect.clone());
        let changed = self.potions.effects.add(effect);
'''),
    (GAME + 'potions.rs', '''        self.mod_effect_added(id, None, amp, new);
        changed
''', '''        self.mod_effect_added(id, None, amp, new);
        if let Some(effect) = told {
            crate::game::paper_entity_events::player_added(self, &effect);
        }
        changed
'''),
    (GAME + 'potions.rs', '''    pub(super) fn tick_mob_effects(&mut self) {
''', '''    pub(super) fn tick_mob_effects(&mut self) {
        crate::game::paper_entity_events::effects_expiring(self);
'''),
    (GAME + 'commands/effects.rs', '''                                super::super::paper_fam_ev_entity::effects_clear(g, "COMMAND");
''', '''                                crate::game::paper_entity_events::player_clear(g, "COMMAND");
'''),
    (GAME + 'commands/effects.rs', '''                                if !super::super::paper_fam_ev_entity::effect_gone(g, e.id, "COMMAND", "REMOVED") {
                                    g.potions.effects.list.retain(|x| x.id != e.id);
                                }
''', '''                                crate::game::paper_entity_events::player_remove(g, e.id, "COMMAND");
'''),
    (LIB + 'game.rs', '''                paper_fam_ev_entity::effects_clear(self, "MILK");
''', '''                paper_entity_events::player_clear(self, "MILK");
'''),
    (LIB + 'game.rs', '''                    if !paper_fam_ev_entity::effect_gone(self, id, "MILK", "REMOVED") {
                        self.potions.effects.list.retain(|e| e.id != id);
                    }
''', '''                    paper_entity_events::player_remove(self, id, "MILK");
'''),
    (GAME + 'totem.rs', '''        super::paper_fam_ev_entity::effects_clear(self, "TOTEM");
''', '''        super::paper_entity_events::player_clear(self, "TOTEM");
'''),
    (GAME + 'creatures.rs', '''                    self.potions.effects.list.retain(|e| e.id != "mining_fatigue");
''', '''                    crate::game::paper_entity_events::player_remove(self, "mining_fatigue", "UNKNOWN");
'''),
    (LIB + 'game.rs', '''            game.potions.effects.add(ambient(primary, amplifier));
''', '''            game.add_player_effect(ambient(primary, amplifier));
'''),
    (LIB + 'game.rs', '''                game.potions.effects.add(ambient(other, 0));
''', '''                game.add_player_effect(ambient(other, 0));
'''),
    # `EffectEventContext`: what command is running.
    (GAME + 'commands.rs', '''    fn execute_command(&mut self, id: &str, ctx: &Context) -> R<i32> {
        match id {
''', '''    fn execute_command(&mut self, id: &str, ctx: &Context) -> R<i32> {
        let _command = crate::game::paper_entity_events::command_scope(id);
        match id {
'''),
    # ---- The end of a hit and a fatal hit, where they land.
    (LIB + 'health.rs', '    pub fn work(&self, base: f32) -> HitWork {', '''    /// `hurtServer`'s `damage` as it ends: what blocking, freezing and a helmet leave, with armour still to come.
    pub fn before_armor(&self, base: f32) -> f32 {
        let mut now = base.max(0.0);
        let blocked = match &self.blocking {
            Some(b) if now > 0.0 && !b.bypassed && !b.pierce => b.rules.resolve(&mut MatchedSets, now, b.angle).unwrap_or(0.0),
            _ => 0.0,
        };
        now -= blocked;
        if self.freezing {
            now *= 5.0;
        }
        if self.helmet {
            now *= 0.75;
        }
        if now.is_finite() { now } else { f32::MAX }
    }
    pub fn work(&self, base: f32) -> HitWork {'''),
    (LIB + 'mobs.rs', '''        amount = match said {
            Some(said) => (said[0] + steps.frames).max(0.0),''', '''        let taken_damage = steps.before_armor(amount);
        amount = match said {
            Some(said) => (said[0] + steps.frames).max(0.0),'''),
    (LIB + 'mobs.rs', '''        if self.health <= 0.0 {
            self.health = 0.0;
            ee::totem(self, source.kind.bypasses_invulnerability());
        }
''', '''        if self.health <= 0.0 && crate::paper_hub::mob_may_die(self, taken_damage) {
            self.health = 0.0;
            ee::totem(self, source.kind.bypasses_invulnerability());
        }
'''),
    (LIB + 'mobs.rs', '''        if self.species() == Species::GlowSquid {
            aquatic::hurt_server(self);
        }
        true
    }
''', '''        if self.species() == Species::GlowSquid {
            aquatic::hurt_server(self);
        }
        crate::paper_hub::hit_landed(self.id as i32, taken_damage, false, self.alive());
        true
    }
'''),
    (LIB + 'mobs.rs', '    pub hurt_dir: f32,', '    pub hurt_dir: f32,\n    /// `ALLOW_DEATH` refused and nothing raised the health: no `die` for this one.\n    pub death_refused: bool,'),
    (GAME + 'creatures.rs', '.filter(|m| !m.alive() && !m.removed && !self.died.contains(&m.id))', '.filter(|m| !m.alive() && !m.removed && !m.extra.death_refused && !self.died.contains(&m.id))'),
    (GAME + 'creatures.rs', '                if !m.alive() && !m.removed && died.insert(m.id) {', '                if !m.alive() && !m.removed && !m.extra.death_refused && died.insert(m.id) {'),
    (GAME + 'creatures.rs', '    fn mob_died(&mut self, m: &Mob) {\n', '    pub(super) fn mob_died(&mut self, m: &Mob) {\n        crate::game::paper_entity_events::mob_dying(self, m);\n'),
    (GAME + 'creatures.rs', 'crate::game::paper_events::on_mob_died(game, m.id, m.kind.id.as_ref());', 'crate::game::paper_events::on_mob_died(game, m);'),
    (GAME + 'commands.rs', '''                    if m.alive() {
                        m.record_damage(
                            horizon_client::mobs::Damage::of(
                                horizon_client::mobs::DamageKind::GenericKill,
                            ),
                            f32::MAX,
                        );
                    }
                    m.health = 0.0;
                    if m.death_time == 0 {
                        m.death_time = 1;
                    }
''', '''                    // `LivingEntity.kill`: a hit like any other, that listeners can refuse.
                    if m.alive() {
                        m.damage(f32::MAX, horizon_client::mobs::Damage::of(horizon_client::mobs::DamageKind::GenericKill));
                    }
                    if !m.alive() && m.death_time == 0 {
                        m.death_time = 1;
                    }
'''),
    (LIB + 'health.rs', '''                knockback: None,
            });
        Ok(outcome)
    }
    pub fn set_health(''', '''                knockback: None,
            });
        if outcome.hurt || outcome.fully_blocked {
            crate::paper_hub::hit_landed(self.entity_id, amount - outcome.blocked, outcome.blocked > 0.0, self.health.health() > 0.0);
        }
        Ok(outcome)
    }
    pub fn set_health('''),
    (GAME + 'chat.rs', '''        self.native_player_died(player, cause.as_deref());
        self.ui.died_at = Some(std::time::Instant::now());''', '''        crate::game::paper_entity_events::dying_player(self.player.as_ref().map_or(0, |p| p.entity_id));
        self.native_player_died(player, cause.as_deref());
        self.ui.died_at = Some(std::time::Instant::now());'''),
    (GAME + 'lan_actions.rs', '''            self.native_player_died(uuid, cause.as_deref());
            if here {''', '''            crate::game::paper_entity_events::dying_player(id);
            self.native_player_died(uuid, cause.as_deref());
            if here {'''),
    (GAME + 'dragon.rs', '''        if self.health <= 0.0 {
            if self.phase.sitting() {
                self.health = 0.0;
            } else {''', '''        let refused = self.health <= 0.0 && !crate::game::paper_entity_events::dragon_may_die(self.id as i32, &mut self.health, damage);
        crate::game::paper_entity_events::dragon_landed(self.id as i32, damage, self.health > 0.0 || refused);
        if self.health <= 0.0 && !refused {
            if self.phase.sitting() {
                self.health = 0.0;
            } else {'''),
    (GAME + 'dragon.rs', '''    fn hurt_dragon_part(
        &mut self,
        part: usize,
        damage: f32,
        counts: bool,
        by: Option<FightWho>,
        kind: DamageKind,
        direct: Option<&ProjectileKind>,
    ) -> Hurt {
''', '''    fn hurt_dragon_part(
        &mut self,
        part: usize,
        damage: f32,
        counts: bool,
        by: Option<FightWho>,
        kind: DamageKind,
        direct: Option<&ProjectileKind>,
    ) -> Hurt {
        let hurt = self.hurt_dragon_part_now(part, damage, counts, by, kind, direct);
        crate::game::paper_entity_events::dragon_after(self);
        hurt
    }
    fn hurt_dragon_part_now(
        &mut self,
        part: usize,
        damage: f32,
        counts: bool,
        by: Option<FightWho>,
        kind: DamageKind,
        direct: Option<&ProjectileKind>,
    ) -> Hurt {
'''),
    # ---- BEFORE_REMOVE before it goes, whatever ended it: `tick_with` says so as it takes each one.
    ('horizon-client-render/src/effects.rs', '    pub fn tick_with(\n        &mut self,', '    /// `tick_with` passes this for the tick when an effect is about to leave the list, whatever ended it.\n    pub const ENDING: i32 = i32::MIN;\n    pub fn tick_with(\n        &mut self,'),
    ('horizon-client-render/src/effects.rs', '''            if !(e.infinite() || e.duration > 0) {
                self.list.remove(i);
                continue;
            }
            let t = if e.infinite() {
                self.tick_count''', '''            if !(e.infinite() || e.duration > 0) {
                omen(e, Self::ENDING);
                self.list.remove(i);
                continue;
            }
            let t = if e.infinite() {
                self.tick_count'''),
    ('horizon-client-render/src/effects.rs', '''                OmenTick::Remove => {
                    self.list.remove(i);
                    continue;
                }''', '''                OmenTick::Remove => {
                    omen(&self.list[i], Self::ENDING);
                    self.list.remove(i);
                    continue;
                }'''),
    ('horizon-client-render/src/effects.rs', '''                        existing.update(&effect);
                        self.list.remove(i);
                        continue;''', '''                        existing.update(&effect);
                        omen(&self.list[i], Self::ENDING);
                        self.list.remove(i);
                        continue;'''),
    ('horizon-client-render/src/effects.rs', '''            if e.infinite() || e.duration > 0 {
                i += 1;
            } else {
                self.list.remove(i);
            }''', '''            if e.infinite() || e.duration > 0 {
                i += 1;
            } else {
                omen(&self.list[i], Self::ENDING);
                self.list.remove(i);
            }'''),
    (GAME + 'potions.rs', '''                &mut |e, t| {
                    if !brewing::is_mod_effect(e.id) {''', '''                &mut |e, t| {
                    if t == horizon_client::effects::PlayerEffects::ENDING {
                        crate::game::paper_entity_events::effect_ending(self, e);
                        return OmenTick::Continue;
                    }
                    if !brewing::is_mod_effect(e.id) {'''),
    # ---- ALLOW_ADD before the game's own refusal.
    (GAME + 'potions.rs', '''                            if mob_can_be_affected(tags, &m.kind.id, effect)
                                && let Some(name) = mob_effect_id(effect)''', '''                            if crate::game::paper_entity_events::mob_takes(tags, m, effect, duration, amp)
                                && let Some(name) = mob_effect_id(effect)'''),
    (GAME + 'potions.rs', '''                            if mob_can_be_affected(tags, &m.kind.id, id)
                                && let Some(name) = mob_effect_id(id)''', '''                            if crate::game::paper_entity_events::mob_takes(tags, m, id, scaled(*duration), *amp)
                                && let Some(name) = mob_effect_id(id)'''),
    (GAME + 'potions.rs', '''                        .any(|(id, ..)| mob_can_be_affected(tags, &m.kind.id, id))''', '''                        .any(|(id, ticks, amp)| crate::game::paper_entity_events::cloud_affects(tags, m, id, *ticks, *amp))'''),
    (GAME + 'potent_sulfur.rs', '''    if mob_can_be_affected(tags, &mob.kind.id, "nausea")
''', '''    if crate::game::paper_entity_events::mob_takes(tags, mob, "nausea", EFFECT_DURATION_IN_TICKS, 0)
'''),
    (MOBS + 'ai.rs', '''                            if m.invulnerable <= 0
                                && !m.invulnerable_flag
                                && !matches!(m.species(), Species::Wither | Species::WitherSkeleton)
                            {''', '''                            if m.invulnerable <= 0
                                && !m.invulnerable_flag
                            {'''),
    # ---- `die` as soon as the work that dealt the hit is done.
    (GAME + 'creatures.rs', '''        horizon_client::spans::time(
            "tick_creature_entities",
            || self.tick_creature_entities(),
        )
    }''', '''        let ticked = horizon_client::spans::time(
            "tick_creature_entities",
            || self.tick_creature_entities(),
        );
        crate::game::paper_entity_events::deaths_now(self);
        ticked
    }'''),
    (LIB + 'game.rs', '''            horizon_client::spans::time("tick_tnt", || self.tick_tnt())?;
''', '''            horizon_client::spans::time("tick_tnt", || self.tick_tnt())?;
            crate::game::paper_entity_events::deaths_now(self);
'''),
    (LIB + 'game.rs', '''            horizon_client::spans::time("tick_lan_packets", || self.tick_lan_packets());
''', '''            horizon_client::spans::time("tick_lan_packets", || self.tick_lan_packets());
            crate::game::paper_entity_events::deaths_now(self);
'''),
    # ---- Respawning and changing level, where they happen.
    (GAME + 'lan_actions.rs', '''        let back = self.dim;
        let away = server && host.hub.guests[i].dimension != level.id();
''', '''        let back = self.dim;
        let came_from = host.hub.guests[i].dimension.to_string();
        let away = server && host.hub.guests[i].dimension != level.id();
'''),
    (GAME + 'lan_actions.rs', '''        if away && let Err(e) = self.enter_level(Some(host), back) {
            log::error!("could not reopen {}: {e}", back.id());
        }
    }
    /// Per tick, for every guest: dig, use items, pick up drops, die.''', '''        let went_to = host.hub.guests[i].dimension.to_string();
        let (respawned, respawned_name) = (host.hub.guests[i].uuid, host.hub.guests[i].name.clone());
        crate::game::paper_entity_events::player_respawned(self, respawned, &respawned_name, false, &came_from, &went_to);
        if away && let Err(e) = self.enter_level(Some(host), back) {
            log::error!("could not reopen {}: {e}", back.id());
        }
    }
    /// Per tick, for every guest: dig, use items, pick up drops, die.'''),
    (GAME + 'server_travel.rs', '''                play.won_game = false;
                self.start_server_trip(host, conn, Dim::Overworld, None);''', '''                play.won_game = false;
                crate::game::paper_entity_events::guest_returning(self, host, conn);
                self.start_server_trip(host, conn, Dim::Overworld, None);'''),
    (GAME + 'server_travel.rs', '''            |g| { g.trigger_dimension(source, target, [from.x, from.y, from.z]) },
        );
''', '''            |g| { g.trigger_dimension(source, target, [from.x, from.y, from.z]) },
        );
        crate::game::paper_entity_events::guest_changed_level(self, host, i, source.id(), target.id());
'''),
    (GAME + 'server_travel.rs', '''            self.as_guest(host, t.conn, |g| g.trigger_dimension(source, t.dim, origin));
''', '''            self.as_guest(host, t.conn, |g| g.trigger_dimension(source, t.dim, origin));
            crate::game::paper_entity_events::guest_changed_level(self, host, i, source.id(), t.dim.id());
'''),
    (LIB + 'game.rs', '''        self.end_travel = (target.kind() == Dim::End || from.kind() == Dim::End)
            && self.end_portal_travel;
        Ok(())
''', '''        self.end_travel = (target.kind() == Dim::End || from.kind() == Dim::End)
            && self.end_portal_travel;
        crate::game::paper_entity_events::host_changed_level(self, from.id(), target.id());
        Ok(())
'''),
    (GAME + 'screens.rs', '''        if self.remote.is_none() && self.level.hardcore {
            self.set_mode(GameMode::Spectator);
        }
        Ok(())
    }
    pub(super) fn death_click(''', '''        if self.remote.is_none() && self.level.hardcore {
            self.set_mode(GameMode::Spectator);
        }
        if self.remote.is_none() {
            crate::game::paper_entity_events::host_respawned(self);
        }
        Ok(())
    }
    pub(super) fn death_click('''),
    (GAME + 'nether_portal.rs', '''    fn send_through_portal(&mut self, id: u32, entry: BlockPos) {
''', '''    fn send_through_portal(&mut self, id: u32, entry: BlockPos) {
        let (waiting, from) = (self.portals.arrivals.len(), self.dim);
        let riders = crate::game::paper_entity_events::riders_of(self, id);
        self.send_through_portal_now(id, entry);
        if self.portals.arrivals.len() > waiting && let Some(arrival) = self.portals.arrivals.last_mut() {
            crate::game::paper_entity_events::portal_left(&mut arrival.tag, id, from.id(), &riders);
        }
    }
    fn send_through_portal_now(&mut self, id: u32, entry: BlockPos) {
'''),
    (GAME + 'nether_portal.rs', '''            super::persist::mark_fresh(&mut tag);
            let key = ((at.x.floor() as i32) >> 4, (at.z.floor() as i32) >> 4);
            match self.world.loaded_entities.iter_mut().find(|(k, _)| *k == key) {''', '''            super::persist::mark_fresh(&mut tag);
            crate::game::paper_entity_events::portal_arriving(&mut tag, plan.dim.id(), [at.x, at.y, at.z]);
            let key = ((at.x.floor() as i32) >> 4, (at.z.floor() as i32) >> 4);
            match self.world.loaded_entities.iter_mut().find(|(k, _)| *k == key) {'''),
    # ---- BlockEvents: the two steps of `ServerPlayerGameMode.useItemOn`, not the click's start.
    ('horizon-world/src/level/interact.rs', '''    fn block_use_item_on<L: LevelAccess + ?Sized>(''', '''    pub fn block_use_item_on<L: LevelAccess + ?Sized>('''),
    (LIB + 'game.rs', '''        if !(sneaking && hands_full) && !crate::game::paper_fam_ev_player::block_denied() && let Some(hit) = block_hit {
            let held = self.held().cloned();
            if self.use_lectern(hit.pos, held.as_ref()) {''', '''        let asked = if !(sneaking && hands_full) && !crate::game::paper_fam_ev_player::block_denied() && let Some(hit) = block_hit {
            crate::game::paper_entity_events::use_item_on(self, hit)
        } else {
            crate::game::paper_entity_events::BlockStep::Skip
        };
        if asked == crate::game::paper_entity_events::BlockStep::Done {
            return Ok(true);
        }
        let empty_only = asked == crate::game::paper_entity_events::BlockStep::EmptyHand;
        if asked != crate::game::paper_entity_events::BlockStep::Skip && let Some(hit) = block_hit {
            let held = if empty_only { None } else { self.held().cloned() };
            if self.use_lectern(hit.pos, held.as_ref()) {'''),
    (LIB + 'game.rs', '''            if let Some(swings) = self.use_shelf(hit) {
                let used = held.unwrap_or_else(''', '''            if !empty_only && let Some(swings) = self.use_shelf(hit) {
                let used = held.unwrap_or_else('''),
    # A cauldron's use is `AbstractCauldronBlock.useItemOn`: the block's step, not the item's.
    (LIB + 'game.rs', '''            if self.use_redstone_ore(hit.pos, held.as_ref())? {
                self.swing_arm();
                return Ok(true);
            }''', '''            let swings = self.swings_started;
            if !empty_only && self.use_on_cauldron(hit.pos)? {
                if swings == self.swings_started {
                    self.swing_arm();
                }
                return Ok(true);
            }
            if !empty_only && self.use_redstone_ore(hit.pos, held.as_ref())? {
                self.swing_arm();
                return Ok(true);
            }'''),
    (LIB + 'game.rs', '''        if allow_block_actions && let Some(hit) = self.pick()
            && self.use_on_cauldron(hit.pos)?
        {
            return Ok(UseResult::Success { swing: true });
        }
''', ''),
    (LIB + 'game.rs', '''            if !anchor_passed && !chains_sign && self.hand == Hand::Main {
''', '''            if crate::game::paper_entity_events::knot_used(self, hit) {
                return Ok(true);
            }
            if (!anchor_passed || empty_only) && !chains_sign {
'''),
    (LIB + 'game.rs', '''                let (result, branch) = self
                    .with_level(|b, level, ticks| {
                        b.use_item_on_branch(
                            level,
                            ticks,
                            BlockPos::new(hit.pos[0], hit.pos[1], hit.pos[2]),
                            face,
                            at,
                            player,
                            item.as_deref(),
                        )
                    });
                self.process_level_events()?;''', '''                let block_at = BlockPos::new(hit.pos[0], hit.pos[1], hit.pos[2]);
                let took = if empty_only {
                    None
                } else {
                    self.with_level(|b, level, ticks| {
                        b.block_use_item_on(level, ticks, block_at, face, at, player, item.as_deref())
                    })
                };
                let (result, branch) = match took {
                    Some(result) => (result, horizon_world::level::UseBranch::ItemOn),
                    None => {
                        use crate::game::paper_entity_events::{BlockStep, empty_hand_at};
                        let result = match empty_hand_at(self, hit, held.as_ref()) {
                            BlockStep::Done => return Ok(true),
                            BlockStep::Skip => horizon_world::level::InteractionResult::Pass,
                            _ => self.with_level(|b, level, ticks| {
                                b.use_without_item(level, ticks, block_at, face, at, player)
                            }),
                        };
                        (result, horizon_world::level::UseBranch::WithoutItem)
                    }
                };
                self.process_level_events()?;'''),
    # The knot is a fence's `useWithoutItem`, or a held lead's `useOn` when the block was passed over.
    (LIB + 'game.rs', '''            if on_block && !crate::game::paper_fam_ev_player::block_denied() && self.with_hand(hand, Game::interact_knot) {
                return Ok(Some(hand));
            }
            if self.with_hand(hand, |game| game.use_hand(block_hit.as_ref()))? {
                return Ok(Some(hand));
            }''', '''            if on_block && block_hit.is_none() && !crate::game::paper_fam_ev_player::block_denied() && self.with_hand(hand, Game::interact_knot) {
                return Ok(Some(hand));
            }
            let used = self.with_hand(hand, |game| game.use_hand(block_hit.as_ref()));
            crate::game::paper_entity_events::click_over();
            if used? {
                return Ok(Some(hand));
            }'''),
    (LIB + 'game.rs', '''        if !claimed && !on_block {
            self.native_use_item(&stack, &mut claimed);
        }
        if claimed {
            self.swing_arm();
            return Ok(true);
        }
''', '''        if !claimed && !on_block {
            self.native_use_item(&stack, &mut claimed);
        }
        if claimed {
            self.swing_arm();
            return Ok(true);
        }
        if on_block && block_hit.is_some() && &*stack.item == "minecraft:lead" && self.interact_knot() {
            return Ok(true);
        }
'''),
    (GAME + 'jukebox.rs', '''            if self.hand != Hand::Main {
                return false;
            }
            self.eject_record(pos);''', '''            if crate::game::paper_entity_events::empty_hand_taken(self) {
                return false;
            }
            self.eject_record(pos);'''),
    (GAME + 'decorated_pot.rs', '''            if self.hand != Hand::Main {
                return false;
            }
            self.audio.play(
                "minecraft:block.decorated_pot.insert_fail",''', '''            if crate::game::paper_entity_events::empty_hand_taken(self) {
                return false;
            }
            self.audio.play(
                "minecraft:block.decorated_pot.insert_fail",'''),
    (GAME + 'chiseled_bookshelf.rs', '''        if self.hand != horizon_client::first_person::Hand::Main {
            return None;
        }
        if !occupied {''', '''        if crate::game::paper_entity_events::empty_hand_taken(self) {
            return None;
        }
        if !occupied {'''),
    # ---- Everything but an item and an orb joined without a word when read off its tree, a traveller too.
    (GAME + 'persist.rs', '''                let moved = take_moved(&mut tag);
                let kind = entity_nbt::kind(&tag).unwrap_or_default();
''', '''                let moved = take_moved(&mut tag);
                crate::game::paper_entity_events::load_told(self);
                let kind = entity_nbt::kind(&tag).unwrap_or_default();
                crate::game::paper_entity_events::loading(self, &tag, &kind, from_disk, moved);
'''),
    (GAME + 'persist.rs', '''        self.spawn_entities_that_loaded();
        super::paper_fam_ev_world::entities_moved(self, "ew_entities_load", &chunks);
''', '''        self.spawn_entities_that_loaded();
        crate::game::paper_entity_events::load_told(self);
        super::paper_fam_ev_world::entities_moved(self, "ew_entities_load", &chunks);
'''),
    (GAME + 'server_travel.rs', '''        self.move_entity_memory(source, id);
        self.set_portal_cooldown(id);
    }''', '''        self.move_entity_memory(source, id);
        self.set_portal_cooldown(id);
        crate::game::paper_entity_events::vehicle_arrived(self, id, source.id());
    }'''),
    # ---- MOB_CONVERSION: `Mob.convertTo` isn't one function here. `convert_to` tells as its mob joins.
    (GAME + 'creatures.rs', '''                    crate::game::paper_events::on_converting(
                        from.id,
                        from.kind.id.as_ref(),
                        params.keep_equipment,
                    );''', '''                    crate::game::paper_entity_events::converting(from, params);'''),
    (GAME + 'dispenser.rs', '''        let id = mob.id;
        self.mobs[index].removed = true;
        self.mobs.push(mob);
        self.native_mobs_joined(self.mobs.len() - 1, false);
        Some(id)''', '''        let id = mob.id;
        crate::game::paper_entity_events::converting(&self.mobs[index], horizon_client::mobs::conversion::Params { keep_equipment: false, preserve_can_pick_up_loot: false });
        self.mobs[index].removed = true;
        self.mobs.push(mob);
        self.native_mobs_joined(self.mobs.len() - 1, false);
        Some(id)'''),
    (LIB + 'mobs.rs', '''    pub life: Option<i32>,
}
/// A snapshot of another entity mobs can see this tick.''', '''    pub life: Option<i32>,
    /// `Mob.convertTo` by this spawn: the mob as it was, and the `ConversionParams`.
    pub conversion: Option<(Box<Mob>, conversion::Params)>,
}
/// A snapshot of another entity mobs can see this tick.'''),
    (GAME + 'creatures.rs', '''                MobEvent::SpawnKind { kind, at, setup } => {
                    horizon_client::mobs::ee::why(super::paper_fam_ev_entity::spawn_kind_why(&kind));
                    let zap = super::paper_fam_ev_entity::zap_ahead(&kind);
                    if let Some(id) = self.spawn_mob(&kind, at, setup.persistent) {
                        self.apply_setup(id, &setup);
                        super::paper_fam_ev_entity::zapped(self, zap, id, at, &kind);
                    }
                }''', '''                MobEvent::SpawnKind { kind, at, setup } => {
                    use crate::game::paper_entity_events as fab;
                    horizon_client::mobs::ee::why(super::paper_fam_ev_entity::spawn_kind_why(&kind));
                    let zap = super::paper_fam_ev_entity::zap_ahead(&kind);
                    if setup.conversion.is_some() {
                        fab::conversion_ahead();
                    }
                    if let Some(id) = self.spawn_mob(&kind, at, setup.persistent) {
                        self.apply_setup(id, &setup);
                        if let Some((old, params)) = &setup.conversion {
                            if zap {
                                fab::zap_converted(self, old, *params, id);
                            } else {
                                fab::converted(self, old, *params, false, id);
                            }
                        }
                        super::paper_fam_ev_entity::zapped(self, zap, id, at, &kind);
                    } else if setup.conversion.is_some() {
                        fab::conversion_off();
                    }
                }'''),
    (GAME + 'creatures.rs', '''                let setup = SpawnSetup {
                    yaw: m.yaw,
                    persistent: true,
                    ..Default::default()
                };
                let pos = [m.position.x, m.position.y, m.position.z];
                sound = Some("entity.mooshroom.shear");''', '''                let setup = SpawnSetup {
                    yaw: m.yaw,
                    persistent: true,
                    conversion: Some((Box::new(m.clone()), horizon_client::mobs::conversion::Params { keep_equipment: false, preserve_can_pick_up_loot: false })),
                    ..Default::default()
                };
                let pos = [m.position.x, m.position.y, m.position.z];
                sound = Some("entity.mooshroom.shear");'''),
    (GAME + 'lightning.rs', '''                            kind: "minecraft:zombified_piglin".into(),
                            at,
                            setup: setup(m),''', '''                            kind: "minecraft:zombified_piglin".into(),
                            at,
                            setup: SpawnSetup {
                                conversion: Some((Box::new(m.clone()), horizon_client::mobs::conversion::Params { keep_equipment: false, preserve_can_pick_up_loot: true })),
                                ..setup(m)
                            },'''),
    (GAME + 'lightning.rs', '''                            kind: "minecraft:witch".into(),
                            at,
                            setup: setup(m),''', '''                            kind: "minecraft:witch".into(),
                            at,
                            setup: SpawnSetup {
                                conversion: Some((Box::new(m.clone()), horizon_client::mobs::conversion::Params { keep_equipment: false, preserve_can_pick_up_loot: false })),
                                ..setup(m)
                            },'''),
    (GAME + 'zombie_villagers.rs', '''        horizon_client::mobs::ee::why("INFECTION");
        let Some(id) = self.spawn_mob("minecraft:zombie_villager", at, old.persistent) else {
            return;
        };''', '''        horizon_client::mobs::ee::why("INFECTION");
        crate::game::paper_entity_events::conversion_ahead();
        let Some(id) = self.spawn_mob("minecraft:zombie_villager", at, old.persistent) else {
            crate::game::paper_entity_events::conversion_off();
            return;
        };'''),
    (GAME + 'zombie_villagers.rs', '''        let Some(z) = self.mobs.iter_mut().find(|m| m.id == id) else {
            return;
        };
        convert_to_zombie_villager(&old, z);
''', '''        let Some(z) = self.mobs.iter_mut().find(|m| m.id == id) else {
            crate::game::paper_entity_events::conversion_off();
            return;
        };
        convert_to_zombie_villager(&old, z);
        crate::game::paper_entity_events::converted(self, &old, horizon_client::mobs::conversion::Params { keep_equipment: true, preserve_can_pick_up_loot: true }, false, id);
'''),
    (GAME + 'zombie_villagers.rs', '''        horizon_client::mobs::ee::why("CURED");
        let Some(vid) = self.spawn_mob("minecraft:villager", at, old.persistent) else {
            return;
        };
        let Some(v) = self.mobs.iter_mut().find(|m| m.id == vid) else {
            return;
        };
        convert_to_villager(&old, v);
''', '''        horizon_client::mobs::ee::why("CURED");
        crate::game::paper_entity_events::conversion_ahead();
        let Some(vid) = self.spawn_mob("minecraft:villager", at, old.persistent) else {
            crate::game::paper_entity_events::conversion_off();
            return;
        };
        let Some(v) = self.mobs.iter_mut().find(|m| m.id == vid) else {
            crate::game::paper_entity_events::conversion_off();
            return;
        };
        convert_to_villager(&old, v);
        crate::game::paper_entity_events::converted(self, &old, horizon_client::mobs::conversion::Params { keep_equipment: false, preserve_can_pick_up_loot: false }, false, vid);
'''),
    # `AbstractCubeMob.remove` is `convertTo` with `SPLIT_ON_DEATH`, once for each child.
    (GAME + 'creatures.rs', '''                        horizon_client::mobs::ee::why("SLIME_SPLIT");
                        if let Some(id) = self.spawn_mob(&m.kind.id, at, m.persistent)
                            && let Some(c) = self.mobs.iter_mut().find(|c| c.id == id)
                        {
                            horizon_client::mobs::cube::set_size(c, size / 2);
                        }''', '''                        horizon_client::mobs::ee::why("SLIME_SPLIT");
                        crate::game::paper_entity_events::conversion_ahead();
                        match self.spawn_mob(&m.kind.id, at, m.persistent) {
                            Some(id) => {
                                if let Some(c) = self.mobs.iter_mut().find(|c| c.id == id) {
                                    horizon_client::mobs::cube::set_size(c, size / 2);
                                }
                                crate::game::paper_entity_events::converted(self, m, horizon_client::mobs::conversion::Params { keep_equipment: false, preserve_can_pick_up_loot: false }, true, id);
                            }
                            None => crate::game::paper_entity_events::conversion_off(),
                        }'''),
    (GAME + 'sulfur_cubes.rs', '''            horizon_client::mobs::ee::why("SLIME_SPLIT");
            if let Some(id) = self.spawn_mob(&m.kind.id, at, m.persistent)
                && let Some(c) = self.mobs.iter_mut().find(|c| c.id == id)
            {
                c.extra.age = -24000;
                sulfur_cube::set_size(c, size / 2, true);
                c.yaw = yaw;
                c.body_yaw = yaw;
                c.head_yaw = yaw;
            }''', '''            horizon_client::mobs::ee::why("SLIME_SPLIT");
            crate::game::paper_entity_events::conversion_ahead();
            let Some(id) = self.spawn_mob(&m.kind.id, at, m.persistent) else {
                crate::game::paper_entity_events::conversion_off();
                continue;
            };
            if let Some(c) = self.mobs.iter_mut().find(|c| c.id == id) {
                c.extra.age = -24000;
                sulfur_cube::set_size(c, size / 2, true);
                c.yaw = yaw;
                c.body_yaw = yaw;
                c.head_yaw = yaw;
            }
            crate::game::paper_entity_events::converted(self, m, horizon_client::mobs::conversion::Params { keep_equipment: false, preserve_can_pick_up_loot: false }, true, id);'''),
    # ---- A mob's effects and fatal hit with the real game: the change waits for it, see `HeldEffect`.
    (LIB + 'mobs.rs', '''        let effect = horizon_client_core::potions::namespaced(effect);
        let before = self
            .extra
            .effects''', '''        let effect = horizon_client_core::potions::namespaced(effect);
        if crate::paper_hub::effects_heard() {
            return crate::paper_hub::hold_add(self.id, effect, ticks, amplifier, ambient, visible);
        }
        let before = self
            .extra
            .effects'''),
    (LIB + 'mobs.rs', '''        // `canBeAffected`: the plugins' say first, and the game refuses after.
        use crate::paper_hub::{EffectStep, mob_effect};
        if !mob_effect(EffectStep::AllowAdd, self, (effect, ticks, amplifier)) || !crate::paper_hub::mob_affected(&self.kind.id, effect) {
            return;
        }
        if ee::effect_refused(self, effect, ticks, amplifier) {
            return;
        }
        mob_effect(EffectStep::BeforeAdd, self, (effect, ticks, amplifier));
''', '''        if crate::paper_hub::effects_heard() {
            return crate::paper_hub::hold_add(self.id, effect, ticks, amplifier, false, true);
        }
        // `canBeAffected`. What's held has been through this already, the plugins' say first.
        if !crate::paper_hub::passing() {
            if !crate::paper_hub::mob_affected(&self.kind.id, effect) {
                return;
            }
            if ee::effect_refused(self, effect, ticks, amplifier) {
                return;
            }
        }
'''),
    (LIB + 'mobs.rs', '''            self.extra.effects.push((effect, ticks, amplifier));
            mob_effect(EffectStep::AfterAdd, self, (effect, ticks, amplifier));
''', '''            self.extra.effects.push((effect, ticks, amplifier));
'''),
    (LIB + 'mobs.rs', '''        use crate::paper_hub::{EffectStep, mob_effect};
        let Some(found) = self.extra.effects.iter().find(|e| e.0 == effect).copied() else {
            return false;
        };
        if !mob_effect(EffectStep::AllowRemove, self, found) {
            return false;
        }
        mob_effect(EffectStep::BeforeRemove, self, found);
        self.extra.effects.retain(|e| e.0 != effect);
        self.extra.hidden_effects.retain(|h| *h != effect);
        self.extra.ambient_effects.retain(|h| *h != effect);
        mob_effect(EffectStep::AfterRemove, self, found);
        true''', '''        let Some(found) = self.extra.effects.iter().find(|e| e.0 == effect).copied() else {
            return false;
        };
        if crate::paper_hub::effects_heard() {
            crate::paper_hub::hold_effect(crate::paper_hub::HeldEffect::Remove { mob: self.id, effect: found });
            return true;
        }
        self.extra.effects.retain(|e| e.0 != effect);
        self.extra.hidden_effects.retain(|h| *h != effect);
        self.extra.ambient_effects.retain(|h| *h != effect);
        true'''),
    (LIB + 'mobs.rs', '''    pub fn effects_clear(m: &mut Mob, cause: &str) {
        use crate::paper_hub::{EffectStep, mob_effect};
        let list = m.extra.effects.clone();
        for effect in &list {
            mob_effect(EffectStep::BeforeRemove, m, *effect);
        }
        let kept: Vec<_> = list.iter().copied().filter(|effect| !mob_effect(EffectStep::AllowRemove, m, *effect)).collect();
        effects_clear_all(m, cause);
        for effect in kept {
            if !m.extra.effects.iter().any(|e| e.0 == effect.0) {
                m.extra.effects.push(effect);
            }
        }
        let left = std::mem::take(&mut m.extra.effects);
        m.extra.hidden_effects.retain(|h| left.iter().any(|e| e.0 == *h));
        m.extra.ambient_effects.retain(|h| left.iter().any(|e| e.0 == *h));
        m.extra.effects = left;
        for effect in &list {
            mob_effect(EffectStep::AfterRemove, m, *effect);
        }
    }''', '''    pub fn effects_clear(m: &mut Mob, cause: &str) {
        if crate::paper_hub::effects_heard() {
            return crate::paper_hub::hold_effect(crate::paper_hub::HeldEffect::Clear { mob: m.id, cause: cause.to_owned() });
        }
        effects_clear_keeping(m, cause, &[]);
    }
    /// `removeAllEffects`, but for what a listener kept.
    pub fn effects_clear_keeping(m: &mut Mob, cause: &str, kept: &[crate::paper_hub::MobEffectRow]) {
        effects_clear_all(m, cause);
        for effect in kept {
            if !m.extra.effects.iter().any(|e| e.0 == effect.0) {
                m.extra.effects.push(*effect);
            }
        }
        let left = std::mem::take(&mut m.extra.effects);
        m.extra.hidden_effects.retain(|h| left.iter().any(|e| e.0 == *h));
        m.extra.ambient_effects.retain(|h| left.iter().any(|e| e.0 == *h));
        m.extra.effects = left;
    }'''),
    (LIB + 'mobs.rs', '''    pub fn effect_cause(cause: &'static str) {
        EFFECT_CAUSE.set(cause);
''', '''    pub fn effect_cause_now() -> &'static str {
        EFFECT_CAUSE.get()
    }
    pub fn put_effect_from(row: Value) {
        EFFECT_FROM.with(|from| *from.borrow_mut() = row);
    }
    pub fn effect_cause(cause: &'static str) {
        EFFECT_CAUSE.set(cause);
'''),
    (LIB + 'mobs/ai.rs', '''    // `tickEffects`: told before the iterator takes it, with the mob's effects where a listener looks for them.
    use crate::paper_hub::{EffectStep, mob_effect};
    let ended: Vec<_> = effects.iter().copied().filter(|e| e.1 <= 0 && e.1 != -1).collect();
    if !ended.is_empty() {
        m.extra.effects = effects;
        for effect in &ended {
            mob_effect(EffectStep::BeforeRemove, m, *effect);
        }
        effects = std::mem::take(&mut m.extra.effects);
    }
    effects.retain(|e| e.1 > 0 || e.1 == -1);
''', '''    // What the listeners haven't heard of stays until they have: the game has them, after this mob's turn.
    let ended: Vec<_> = effects.iter().copied().filter(|e| e.1 <= 0 && e.1 != -1).collect();
    let stay = crate::paper_hub::effects_ended(m, &ended);
    effects.retain(|e| e.1 > 0 || e.1 == -1 || stay.contains(&e.0));
'''),
    (LIB + 'mobs/ai.rs', '''    m.extra.effects = effects;
    for effect in &ended {
        mob_effect(EffectStep::AfterRemove, m, *effect);
    }
}
''', '''    m.extra.effects = effects;
}
'''),
    (LIB + 'mobs.rs', '''        if self.health <= 0.0 && crate::paper_hub::mob_may_die(self, taken_damage) {''', '''        if self.health <= 0.0 && crate::paper_hub::mob_may_die(self, taken_damage, source.kind.bypasses_invulnerability()) {'''),
    (LIB + 'mobs.rs', '''    pub death_refused: bool,
''', '''    pub death_refused: bool,
    /// A fatal hit waiting for `ALLOW_DEATH`.
    pub death_asked: Option<crate::paper_hub::DeathAsked>,
'''),
    (GAME + 'creatures.rs', '''.filter(|m| !m.alive() && !m.removed && !m.extra.death_refused && !self.died''', '''.filter(|m| !m.alive() && !m.removed && !m.extra.death_refused && m.extra.death_asked.is_none() && !self.died'''),
    (GAME + 'creatures.rs', '''                if !m.alive() && !m.removed && !m.extra.death_refused && died.insert(m.id) {
                    dead.push(m.clone());
                }''', '''                if !m.alive() && !m.removed && !m.extra.death_refused && m.extra.death_asked.is_none() && died.insert(m.id) {
                    dead.push(m.clone());
                }'''),
    # The turns of the mob tick: `self` is only read in them, so between two it can be had.
    (GAME + 'creatures.rs', '''            let base = self.mob_world_base(&[]);
            let path_changes = horizon_client::mobs::collision_changes(''', '''            let mut base = self.mob_world_base(&[]);
            let path_changes = horizon_client::mobs::collision_changes('''),
    (GAME + 'creatures.rs', '''        let ticking = self.entity_ticking_area();
        let guests = self.lan_guest_views();
        let players = self.level_players();''', '''        let mut ticking = self.entity_ticking_area();
        let guests = self.lan_guest_views();
        let players = self.level_players();'''),
    (GAME + 'creatures.rs', '''            for i in 0..mobs.len() {
                let mut w = MobWorld {
                    others: &snapshot,
                    others_grid: Some(&grid),
                    others_index: Some(&index),
                    ..base.clone()
                };
                let m = &mut mobs[i];
''', '''            for i in 0..mobs.len() {
                if i >= mobs.len() {
                    break;
                }
                if crate::game::paper_entity_events::effects_ending(&mobs[i]) {
                    let id = mobs[i].id;
                    let (x, y, z) = (mobs[i].position.x, mobs[i].position.y, mobs[i].position.z);
                    let near = super::natural_spawning::nearest_player(&spawn_players, x, y, z).map(|p| p.position);
                    if self.world.loaded(x.floor() as i32, z.floor() as i32)
                        && ticking(x.floor() as i32, z.floor() as i32)
                        && !Self::at_rest(&mobs[i], near, self.world.game_time)
                    {
                        std::mem::swap(&mut self.mobs, &mut mobs);
                        std::mem::swap(&mut self.died, &mut died);
                        std::mem::swap(&mut self.world.random, &mut scratch);
                        crate::game::paper_entity_events::effects_told(self, id);
                        std::mem::swap(&mut self.world.random, &mut scratch);
                        std::mem::swap(&mut self.died, &mut died);
                        std::mem::swap(&mut self.mobs, &mut mobs);
                        base = self.mob_world_base(&[]);
                        ticking = self.entity_ticking_area();
                        if i >= mobs.len() {
                            break;
                        }
                    }
                }
                let mut w = MobWorld {
                    others: &snapshot,
                    others_grid: Some(&grid),
                    others_index: Some(&index),
                    ..base.clone()
                };
                let m = &mut mobs[i];
'''),
    (GAME + 'creatures.rs', '''                horizon_client::mobs::apply_brain_events(&mut mobs, &mut events, from);
                horizon_client::mobs::ee::mark_actor(&mut events, from, mobs[i].id);
            }
            let mut flags = keep.into_iter();''', '''                horizon_client::mobs::apply_brain_events(&mut mobs, &mut events, from);
                horizon_client::mobs::ee::mark_actor(&mut events, from, mobs[i].id);
                if crate::game::paper_entity_events::turn_waits(&mobs[i]) {
                    std::mem::swap(&mut self.mobs, &mut mobs);
                    std::mem::swap(&mut self.died, &mut died);
                    std::mem::swap(&mut self.world.random, &mut scratch);
                    crate::game::paper_entity_events::turn_over(self);
                    std::mem::swap(&mut self.world.random, &mut scratch);
                    std::mem::swap(&mut self.died, &mut died);
                    std::mem::swap(&mut self.mobs, &mut mobs);
                    base = self.mob_world_base(&[]);
                    ticking = self.entity_ticking_area();
                    if i >= mobs.len() {
                        break;
                    }
                    let m = &mobs[i];
                    if kept && !m.alive() && !m.removed && !m.extra.death_refused && m.extra.death_asked.is_none() && died.insert(m.id) {
                        dead.push(m.clone());
                    }
                    snapshot[i] = seen(m, uuid);
                    snapshot[i].alive &= kept;
                    grid.update(i, snapshot[i].bounds);
                }
            }
            let mut flags = keep.into_iter();'''),
    # Where a change made outside a mob's tick has its game back.
    (GAME + 'creatures.rs', '''    pub(super) fn handle_mob_events(&mut self, events: Vec<MobEvent>) -> Result<()> {
''', '''    pub(super) fn handle_mob_events(&mut self, events: Vec<MobEvent>) -> Result<()> {
        let done = self.handle_mob_events_now(events);
        crate::game::paper_entity_events::effects_now(self);
        done
    }
    fn handle_mob_events_now(&mut self, events: Vec<MobEvent>) -> Result<()> {
'''),
    (GAME + 'creatures.rs', '''    pub(super) fn projectile_hit(&mut self, p: &Projectile, hit: Hit) -> Result<()> {
''', '''    pub(super) fn projectile_hit(&mut self, p: &Projectile, hit: Hit) -> Result<()> {
        let done = self.projectile_hit_now(p, hit);
        crate::game::paper_entity_events::effects_now(self);
        done
    }
    fn projectile_hit_now(&mut self, p: &Projectile, hit: Hit) -> Result<()> {
'''),
    (LIB + 'game.rs', '''    fn use_item(&mut self) -> Result<Option<Hand>> {
        self.with_player_actor(|game| game.use_item_as_player())
    }''', '''    fn use_item(&mut self) -> Result<Option<Hand>> {
        let used = self.with_player_actor(|game| game.use_item_as_player());
        crate::game::paper_entity_events::effects_now(self);
        used
    }'''),
    (GAME + 'block_entity_anims.rs', '''                                m.add_effect("minecraft:glowing", 60, 0);
                            }
                        }
''', '''                                m.add_effect("minecraft:glowing", 60, 0);
                            }
                        }
                        crate::game::paper_entity_events::effects_now(self);
'''),
    (GAME + 'enchant_effects.rs', '''            m.add_effect(id, ticks, amplifier);
            self.mod_effect_added(id, Some(entity), amplifier, new);''', '''            m.add_effect(id, ticks, amplifier);
            crate::game::paper_entity_events::effects_now(self);
            self.mod_effect_added(id, Some(entity), amplifier, new);'''),
    (GAME + 'potions.rs', '''                                m.add_effect(name, duration, amp);
                                self.mod_effect_added(name, Some(id as i32), amp, new);''', '''                                m.add_effect(name, duration, amp);
                                crate::game::paper_entity_events::effects_now(self);
                                self.mod_effect_added(name, Some(id as i32), amp, new);'''),
    (GAME + 'potions.rs', '''                                    !hidden.contains(id),
                                );
                                self.mod_effect_added(name, Some(mob as i32), *amp, new);''', '''                                    !hidden.contains(id),
                                );
                                crate::game::paper_entity_events::effects_now(self);
                                self.mod_effect_added(name, Some(mob as i32), *amp, new);'''),
    (GAME + 'potions.rs', '''                            self.mobs[i].add_effect(name, d, *amp);
''', '''                            self.mobs[i].add_effect(name, d, *amp);
                            crate::game::paper_entity_events::effects_now(self);
'''),
    (GAME + 'raids.rs', '''                m.add_effect("glowing", 1000, 1);
            }
        }
''', '''                m.add_effect("glowing", 1000, 1);
            }
        }
        crate::game::paper_entity_events::effects_now(self);
'''),
    (GAME + 'trading.rs', '''                        m.add_effect("minecraft:regeneration", 200, 0);
                    }
''', '''                        m.add_effect("minecraft:regeneration", 200, 0);
                    }
                    crate::game::paper_entity_events::effects_now(self);
'''),
    (GAME + 'wither.rs', '''                if killed {
                    self.heal_skull_owner(p, live_owner);''', '''                crate::game::paper_entity_events::effects_now(self);
                if killed {
                    self.heal_skull_owner(p, live_owner);'''),
    (GAME + 'commands/effects.rs', '''                                false,
                                particles,
                            );
                            self.mod_effect_added(name, Some(id as i32), amplifier, new);
                            true''', '''                                false,
                                particles,
                            );
                            crate::game::paper_entity_events::effects_now(self);
                            let refused = crate::game::paper_entity_events::effects_refused();
                            self.mod_effect_added(name, Some(id as i32), amplifier, new);
                            !refused'''),
    (GAME + 'commands/effects.rs', '''                _ => continue,
            };
            tracker.track(t, changed);
        }
        let message = match (tracker.changed.as_slice(), effect) {''', '''                _ => continue,
            };
            crate::game::paper_entity_events::effects_now(self);
            let changed = !crate::game::paper_entity_events::effects_refused() && changed;
            tracker.track(t, changed);
        }
        let message = match (tracker.changed.as_slice(), effect) {'''),
    # The dragon has no list to stand in: lifted out, the game can be lent for its hit.
    (GAME + 'dragon.rs', '''        let hurt = d.hurt_part(part, damage, source, kind, direct);
        if counts && matches!(hurt, Hurt::Landed { .. })''', '''        let Some(mut lifted) = self.dragon.take() else {
            return Hurt::Refused;
        };
        let hurt = horizon_mod_runtime::lend(self, || lifted.hurt_part(part, damage, source, kind, direct));
        self.dragon = Some(lifted);
        let Some(d) = self.dragon.as_mut() else {
            return hurt;
        };
        if counts && matches!(hurt, Hurt::Landed { .. })'''),
    # ---- ALLOW_DAMAGE with the game. Where the result is read, it's asked before the hit; where it isn't, or in a
    # mob's tick, the hit waits for the game. `reaches_hurt_server` is the part of `hurtServer` before the plugins are asked.
    (LIB + 'mobs.rs', '''    pub fn damage(&mut self, amount: f32, source: Damage) -> bool {
        self.damage_with(amount, source, None)
    }''', '''    pub fn damage(&mut self, amount: f32, source: Damage) -> bool {
        if crate::paper_hub::hit_held(self.id, amount, source) {
            return true;
        }
        self.damage_with(amount, source, None)
    }
    /// `damage` where nothing reads the result and the game can be had right after.
    pub fn damage_held(&mut self, amount: f32, source: Damage) {
        crate::paper_hub::holding_hits(true);
        self.damage(amount, source);
        crate::paper_hub::holding_hits(false);
    }
    /// Whether a hit gets as far as `hurtServer`'s `isSleeping`, what it asked and refused on the way left undone.
    pub fn reaches_hurt_server(&self, amount: f32, source: &Damage) -> bool {
        let mut mob = self.clone();
        if mob.species() == Species::SulfurCube && !sulfur_cube::accepts_damage(&mut mob, source, amount) {
            return false;
        }
        if !mob.alive() || mob.removed || (mob.invulnerable_flag && !source.kind.bypasses_invulnerability()) {
            return false;
        }
        let reflected = &*mob.kind.id == "minecraft:ghast" && source.kind == DamageKind::Fireball && source.attacker == Some(Who::Player);
        if source.is_fire() && (mob.kind.flags.fire_immune || mob.has_effect("minecraft:fire_resistance")) && !reflected {
            return false;
        }
        ai::accepts_damage(&mut mob, source, amount)
    }'''),
    (GAME + 'native_hooks.rs', '''fn mob_source(damage: &Damage) -> HurtSource<'static> {''', '''pub(super) fn mob_source(damage: &Damage) -> HurtSource<'static> {'''),
    (GAME + 'creatures.rs', '''                let kept = m.tick(&w, &mut scratch, &mut events);
''', '''                horizon_client::paper_hub::holding_hits(true);
                let kept = m.tick(&w, &mut scratch, &mut events);
'''),
    (GAME + 'creatures.rs', '''                horizon_client::mobs::apply_brain_events(&mut mobs, &mut events, from);
                horizon_client::mobs::ee::mark_actor(&mut events, from, mobs[i].id);
                if crate::game::paper_entity_events::turn_waits(&mobs[i]) {''', '''                horizon_client::mobs::apply_brain_events(&mut mobs, &mut events, from);
                horizon_client::paper_hub::holding_hits(false);
                horizon_client::mobs::ee::mark_actor(&mut events, from, mobs[i].id);
                if crate::game::paper_entity_events::turn_waits(&mobs[i]) {'''),
    (GAME + 'block_entity_anims.rs', '''        m.damage(4.0, Damage::of(DamageKind::Magic));
        self.audio.play("minecraft:block.conduit.attack.target", Some(at), 1.0, 1.0);''', '''        m.damage_held(4.0, Damage::of(DamageKind::Magic));
        crate::game::paper_entity_events::effects_now(self);
        self.audio.play("minecraft:block.conduit.attack.target", Some(at), 1.0, 1.0);'''),
    (GAME + 'commands/world_cmds.rs', '''                let hurt = self
                    .mobs
                    .iter_mut()
                    .find(|m| m.id == id)
                    .is_some_and(|m| m.damage(amount, damage));
''', '''                let hurt = crate::game::paper_entity_events::hit_asked(self, id, amount, damage)
                    && self
                        .mobs
                        .iter_mut()
                        .find(|m| m.id == id)
                        .is_some_and(|m| m.damage(amount, damage));
                crate::game::paper_entity_events::effects_now(self);
'''),
    (GAME + 'commands.rs', '''                        m.damage(f32::MAX, horizon_client::mobs::Damage::of(horizon_client::mobs::DamageKind::GenericKill));
''', '''                        m.damage_held(f32::MAX, horizon_client::mobs::Damage::of(horizon_client::mobs::DamageKind::GenericKill));
'''),
    (GAME + 'creatures.rs', '''                        m.damage(amount, horizon_client::mobs::Damage::of(kind));
                    }
                }
                MobEvent::SetTarget { mob, target } => {''', '''                        m.damage_held(amount, horizon_client::mobs::Damage::of(kind));
                    }
                }
                MobEvent::SetTarget { mob, target } => {'''),
    (GAME + 'enchant_effects.rs', '''            let m = &mut self.mobs[index];
            m.hurt(damage, None, &mut self.world.random);
            let center = [m.position.x, m.position.y, m.position.z];''', '''            let m = &mut self.mobs[index];
            let center = [m.position.x, m.position.y, m.position.z];
            m.damage_held(damage, horizon_client::mobs::Damage::of(horizon_client::mobs::DamageKind::Generic));
            crate::game::paper_entity_events::effects_now(self);
            if index >= self.mobs.len() {
                return;
            }'''),
    (GAME + 'explosion.rs', '''    if hurtable && !shielded {
        m.damage(amount, source);
    }''', '''    if hurtable && !shielded {
        m.damage_held(amount, source);
    }'''),
    (GAME + 'fangs.rs', '''                m.damage(
                    6.0,
                    horizon_client::mobs::Damage {''', '''                m.damage_held(
                    6.0,
                    horizon_client::mobs::Damage {'''),
    (GAME + 'fireworks.rs', '''            if let Some(m) = self.mobs.iter_mut().find(|m| m.id == id) {
                let hurt = m.damage(amount, Damage::fireworks(attacker, at));''', '''            if !crate::game::paper_entity_events::hit_asked(self, id, amount, Damage::fireworks(attacker, at)) {
                continue;
            }
            if let Some(m) = self.mobs.iter_mut().find(|m| m.id == id) {
                let hurt = m.damage(amount, Damage::fireworks(attacker, at));'''),
    (GAME + 'lightning.rs', '''                    m.damage(f32::MAX, damage);
                    continue;''', '''                    m.damage_held(f32::MAX, damage);
                    continue;'''),
    (GAME + 'lightning.rs', '''            m.damage(5.0, damage);
        }
        if !events.is_empty() {''', '''            m.damage_held(5.0, damage);
        }
        crate::game::paper_entity_events::effects_now(self);
        if !events.is_empty() {'''),
    (GAME + 'mace.rs', '''        let ctx = EffectContext {
            this: Some(&kind),
            attacker: Some("minecraft:player"),
            direct_attacker: Some("minecraft:player"),''', '''        let asked = crate::game::paper_entity_events::hit_asked(self, self.mobs[index].id, amount, source);
        let ctx = EffectContext {
            this: Some(&kind),
            attacker: Some("minecraft:player"),
            direct_attacker: Some("minecraft:player"),'''),
    (GAME + 'mace.rs', '''        let hurt = self.mobs[index].damage_with(amount, source, Some(&mut breach));''', '''        let hurt = asked && self.mobs[index].damage_with(amount, source, Some(&mut breach));'''),
    (GAME + 'potions.rs', '''                                m.damage(
                                    (6 << amp.min(30)) as f32,
                                    Damage::of(DamageKind::Magic),
                                );''', '''                                m.damage_held(
                                    (6 << amp.min(30)) as f32,
                                    Damage::of(DamageKind::Magic),
                                );
                                crate::game::paper_entity_events::effects_now(self);'''),
    (GAME + 'potions.rs', '''                                let harm = m.potion_harm(amount(6));
                                m.hurt(harm, Some(at), &mut self.world.random);''', '''                                let harm = m.potion_harm(amount(6));
                                m.damage_held(harm, Damage::player(at));
                                crate::game::paper_entity_events::effects_now(self);'''),
    (GAME + 'potions.rs', '''                    m.hurt(harm, Some([at.x, at.y, at.z]), &mut self.world.random);''', '''                    m.damage_held(harm, Damage::player([at.x, at.y, at.z]));
                    crate::game::paper_entity_events::effects_now(self);'''),
    (GAME + 'splash_water.rs', '''                    let invulnerable = m.invulnerable_flag;
                    m.damage(1.0, source);''', '''                    let invulnerable = m.invulnerable_flag;
                    m.damage_held(1.0, source);'''),
    (GAME + 'wind.rs', '''                    m.damage(1.0, source);
                }
                if p.owner == Some(Who::Player) {''', '''                    m.damage_held(1.0, source);
                }
                crate::game::paper_entity_events::effects_now(self);
                if p.owner == Some(Who::Player) {'''),
    # Players are in `self.player` for the hit, so it's asked first.
    (GAME + 'creatures.rs', '''        if self.creative || self.combat.refuse_player_hit {
            self.combat.forget_noted(false);
            return false;
        }
''', '''        if self.creative || self.combat.refuse_player_hit {
            self.combat.forget_noted(false);
            return false;
        }
        let causing = known.flatten().or(self.combat.projectile_owner).map_or(0, |id| id as i32);
        if !crate::game::paper_entity_events::player_hit_asked(self, damage_type, causing, 0, amount) {
            self.combat.forget_noted(false);
            return false;
        }
'''),
    (GAME + 'creatures.rs', '''    pub(super) fn hurt_player_by(&mut self, damage_type: &str, amount: f32) {
        let mut canceled = false;
''', '''    pub(super) fn hurt_player_by(&mut self, damage_type: &str, amount: f32) {
        if !crate::game::paper_entity_events::player_hit_asked(self, damage_type, 0, 0, amount) {
            self.combat.forget_noted(false);
            return;
        }
        let mut canceled = false;
'''),
    (GAME + 'dragon.rs', '''        let (health, rules) = (&self.motion.health, self.world.health_rules);
        let id = health.damage.id("minecraft:mob_attack");
        let mut canceled = false;
        if let Some(p) = self.player.as_mut() {
            push_seen_with_hurt(''', '''        let hurts = hurts && crate::game::paper_entity_events::player_hit_asked(self, "minecraft:mob_attack", 0, 0, 5.0);
        let (health, rules) = (&self.motion.health, self.world.health_rules);
        let id = health.damage.id("minecraft:mob_attack");
        let mut canceled = false;
        if let Some(p) = self.player.as_mut() {
            push_seen_with_hurt('''),
    (GAME + 'explosion.rs', '''        if let Some((id, name)) = attacker
            .filter(|_| !creative && !spectator && hurtable)
        {
            self.note_named_attacker(damage_type, *id, name.clone());
        }''', '''        let hurtable = hurtable
            && crate::game::paper_entity_events::player_hit_asked(self, damage_type, attacker.map_or(0, |(id, _)| *id as i32), 0, amount);
        if let Some((id, name)) = attacker
            .filter(|_| !creative && !spectator && hurtable)
        {
            self.note_named_attacker(damage_type, *id, name.clone());
        }'''),
    (GAME + 'falling_damage.rs', '''        if let Some(p) = self.player.as_mut()
            && let Some(id) = self.motion.health.damage.id(kind)
        {
            let _ = p.hurt(&self.motion.health, self.world.health_rules, id, amount);
        }''', '''        if !crate::game::paper_entity_events::player_hit_asked(self, kind, 0, 0, amount) {
            return;
        }
        if let Some(p) = self.player.as_mut()
            && let Some(id) = self.motion.health.damage.id(kind)
        {
            let _ = p.hurt(&self.motion.health, self.world.health_rules, id, amount);
        }
        crate::game::paper_entity_events::effects_now(self);'''),
    (GAME + 'potions.rs', '''        let creative = self.creative;
        let rules = self.world.health_rules;
        let Some(p) = self.player.as_mut() else {
            self.combat.forget_noted(false);
            return;''', '''        if !crate::game::paper_entity_events::effect_hit_asked(self, &action) {
            self.combat.forget_noted(false);
            return;
        }
        let creative = self.creative;
        let rules = self.world.health_rules;
        let Some(p) = self.player.as_mut() else {
            self.combat.forget_noted(false);
            return;'''),
    # ---- `/tp` of what isn't a player into another level: it travels as an arrival, with no portal events.
    (GAME + 'commands.rs', '''        rotation: Option<[f32; 2]>,
    ) -> bool {
        let conn = match t {
            Target::Player => self.lan_acting,
            Target::Guest(conn) => Some(conn),
            _ => None,
        };
        let at = self.target_level(t);''', '''        rotation: Option<[f32; 2]>,
    ) -> bool {
        if crate::game::paper_entity_events::entity_across(self, t, level, pos, rotation) {
            return true;
        }
        let conn = match t {
            Target::Player => self.lan_acting,
            Target::Guest(conn) => Some(conn),
            _ => None,
        };
        let at = self.target_level(t);'''),
    (GAME + 'nether_portal.rs', '''        if let Some(i) = self.dropped.iter().position(|e| e.id == id) {
            let e = &self.dropped[i];
            let plan = self.entity_plan(entry, e.position, 0.25, 0.25, false);''', '''        self.send_entity(id, entry);
    }
    /// `Entity.teleport` by a command into `dim`: an arrival that lands exactly where it's told.
    pub(super) fn send_to_level(&mut self, id: u32, dim: Dim, pos: [f64; 3], rotation: Option<[f32; 2]>) {
        let approximate = BlockPos::new(pos[0].floor() as i32, pos[1].floor() as i32, pos[2].floor() as i32);
        let (waiting, from) = (self.portals.arrivals.len(), self.dim);
        let riders = crate::game::paper_entity_events::riders_of(self, id);
        SENDING.set(Some(Plan {
            dim,
            approximate,
            entry_x_axis: true,
            offset: Vec3::ZERO,
            width: 0.0,
            height: 0.0,
            end: false,
            rules: PortalRules { exact: true, ..PortalRules::default() },
        }));
        self.send_entity(id, approximate);
        SENDING.set(None);
        if self.portals.arrivals.len() > waiting && let Some(arrival) = self.portals.arrivals.last_mut() {
            crate::game::paper_entity_events::portal_left(&mut arrival.tag, id, from.id(), &riders);
            crate::game::paper_entity_events::sent_to(&mut arrival.tag, pos, rotation);
        }
    }
    fn send_entity(&mut self, id: u32, entry: BlockPos) {
        if let Some(i) = self.dropped.iter().position(|e| e.id == id) {
            let e = &self.dropped[i];
            let plan = self.entity_plan(entry, e.position, 0.25, 0.25, false);'''),
    (GAME + 'nether_portal.rs', '''        let mut plan = self.entity_plan_own(entry, at, width, height, reset_forward);
''', '''        if let Some(plan) = SENDING.get() {
            return plan;
        }
        let mut plan = self.entity_plan_own(entry, at, width, height, reset_forward);
'''),
    (GAME + 'nether_portal.rs', '''pub(super) struct Plan {
    pub(super) dim: Dim,''', '''pub(super) struct Plan {
    pub(super) dim: Dim,'''),
    (GAME + 'nether_portal.rs', '''/// What a listener of `EntityPortalEvent` set on a traveller: the game's own until one does.
''', '''thread_local! {
    /// The plan of a `/tp` into another level, for what's being sent.
    static SENDING: std::cell::Cell<Option<Plan>> = const { std::cell::Cell::new(None) };
}
/// What a listener of `EntityPortalEvent` set on a traveller: the game's own until one does.
'''),
    (GAME + 'nether_portal.rs', '''            let (at, motion) = super::paper_fam_ev_entity::portal_exit(self, &tag, plan.rules, at, motion);
''', '''            let (at, motion, [yaw, pitch]) = crate::game::paper_entity_events::sent_landing(&mut tag, at, motion, [yaw, pitch]);
            let (at, motion) = super::paper_fam_ev_entity::portal_exit(self, &tag, plan.rules, at, motion);
'''),
    # ---- PlayerBlockBreakEvents.AFTER: removed, before `Block.destroy` and the drops.
    (LIB + 'game.rs', '''        let removed = self
            .with_level(|b, level, ticks| {
                b.player_destroy_block(
                    level,
                    ticks,
                    BlockPos::new(pos[0], pos[1], pos[2]),
                    creative,
                    correct_tool,
                )
            });
        if removed && creative {''', '''        let block_was = crate::game::paper_entity_events::block_was(self, pos);
        let removed = self
            .with_level(|b, level, ticks| {
                b.player_destroy_block(
                    level,
                    ticks,
                    BlockPos::new(pos[0], pos[1], pos[2]),
                    creative,
                    correct_tool,
                )
            });
        if removed {
            crate::game::paper_entity_events::block_broken(self, pos, state, &block_was);
        }
        if removed && creative {'''),
    # ---- ServerBlockEntityEvents: what comes and goes is written down as it does, in order, even within a tick.
    ('horizon-client-render/src/block_entities.rs', '''#[derive(Default)]
pub struct BlockEntities {
    map: HashMap<Pos, BlockEntity>,
''', '''static NOTES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether a server wants to hear what comes and goes.
pub fn hear_changes(listening: bool) {
    NOTES.store(listening, std::sync::atomic::Ordering::Relaxed);
}

#[derive(Default)]
pub struct BlockEntities {
    /// What came (`None`) and went (`Some`, what it was), in order. Only while a server listens.
    notes: Vec<(Pos, Option<BlockEntity>)>,
    map: HashMap<Pos, BlockEntity>,
'''),
    ('horizon-client-render/src/block_entities.rs', '''    pub fn insert(&mut self, pos: Pos, entity: BlockEntity) -> Option<BlockEntity> {
        let old = self.map.insert(pos, entity);
        if old.is_none() {
            self.add(pos);
        }
        old
    }
''', '''    pub fn insert(&mut self, pos: Pos, entity: BlockEntity) -> Option<BlockEntity> {
        // Written back after a change, it's the same block entity.
        let kind = std::mem::discriminant(&entity);
        let old = self.map.insert(pos, entity);
        if old.is_none() {
            self.add(pos);
        }
        if old.as_ref().is_none_or(|old| std::mem::discriminant(old) != kind) {
            self.note(pos, None);
            self.note_gone(pos, old.as_ref());
        }
        old
    }
    fn note(&mut self, pos: Pos, gone: Option<BlockEntity>) {
        if NOTES.load(std::sync::atomic::Ordering::Relaxed) {
            self.notes.push((pos, gone));
        }
    }
    fn note_gone(&mut self, pos: Pos, gone: Option<&BlockEntity>) {
        if let Some(gone) = gone.filter(|_| NOTES.load(std::sync::atomic::Ordering::Relaxed)) {
            self.notes.push((pos, Some(gone.clone())));
        }
    }
    /// What came and went since the last call, in order: `None` came, `Some` went.
    pub fn take_notes(&mut self) -> Vec<(Pos, Option<BlockEntity>)> {
        std::mem::take(&mut self.notes)
    }
'''),
    ('horizon-client-render/src/block_entities.rs', '''        if !self.map.contains_key(&pos) {
            self.add(pos);
        }
        self.map.entry(pos).or_insert_with(make)
''', '''        if !self.map.contains_key(&pos) {
            self.add(pos);
            self.map.insert(pos, make());
            self.note(pos, None);
        }
        self.map.get_mut(&pos).expect("just made")
'''),
    ('horizon-client-render/src/block_entities.rs', '''        self.cancel(pos);
        self.map.remove(pos)
    }
''', '''        self.cancel(pos);
        let gone = self.map.remove(pos);
        self.note_gone(*pos, gone.as_ref());
        gone
    }
'''),
    ('horizon-client-render/src/block_entities.rs', '''        let (live, pending, freed) = (&mut self.live, &mut self.pending, &mut self.freed);
        self.map.retain(|p, e| {
            let kept = keep(p, e);
            if !kept {
                *freed |= live.remove(p).is_some();
                pending.remove(p);
            }
            kept
        });
''', '''        let (live, pending, freed) = (&mut self.live, &mut self.pending, &mut self.freed);
        let notes = (&mut self.notes, NOTES.load(std::sync::atomic::Ordering::Relaxed));
        let (list, heard) = notes;
        self.map.retain(|p, e| {
            let kept = keep(p, e);
            if !kept {
                *freed |= live.remove(p).is_some();
                pending.remove(p);
                if heard {
                    list.push((*p, Some(e.clone())));
                }
            }
            kept
        });
'''),
    ('horizon-client-render/src/block_entities.rs', '''        for (p, e) in entities {
            if self.map.insert(p, e).is_some() {
                self.replaced.insert(p);
            }
        }
''', '''        for (p, e) in entities {
            let old = self.map.insert(p, e);
            self.note(p, None);
            if old.is_some() {
                self.replaced.insert(p);
                self.note_gone(p, old.as_ref());
            }
        }
'''),
    # ---- ALLOW_DAMAGE at the last sites: a wither skull's hit on a mob, a parrot's cookie.
    (GAME + 'wither.rs', '''                let owner_species = owner
                    .and_then(|o| self.mobs.iter().find(|m| m.id == o))
                    .map(|m| m.species());
                let killed = match self.mobs.iter_mut().find(|m| m.id == id) {
                    Some(
                        m,
                    ) if owner_species
                        .is_some_and(|o| {
                            horizon_client::mobs::wither::shielded_from(m.species(), o)
                        }) => false,
                    Some(m) => {
''', '''                let owner_species = owner
                    .and_then(|o| self.mobs.iter().find(|m| m.id == o))
                    .map(|m| m.species());
                let asked = crate::game::paper_entity_events::hit_asked(self, id, amount, source);
                let killed = match self.mobs.iter_mut().find(|m| m.id == id) {
                    Some(
                        m,
                    ) if owner_species
                        .is_some_and(|o| {
                            horizon_client::mobs::wither::shielded_from(m.species(), o)
                        }) => false,
                    Some(_) if !asked => false,
                    Some(m) => {
'''),
    (GAME + 'creatures.rs', '''                let id = m.id;
                let hurt = m
                    .damage(
                        f32::MAX,
                        horizon_client::mobs::Damage::player(player_at.unwrap_or(center)),
                    );
                if hurt {
''', '''                let id = m.id;
                let source = horizon_client::mobs::Damage::player(player_at.unwrap_or(center));
                let hurt = crate::game::paper_entity_events::hit_asked(self, id, f32::MAX, source)
                    && self.mobs[index].damage(f32::MAX, source);
                if hurt {
'''),
    # `execute in` moves the command to another level: `@s` is still whoever runs it, wherever they are.
    (GAME + 'commands.rs', '''        let source_dim = self.dim;
        let dims = if s.world_limited() { vec![source_dim] } else { self.loaded_dims() };
''', '''        if s.kind == 's' && let Some(&t) = source_candidates.first() && !self.level_holds(t) {
            let found = self.in_target_level(t, |game| game.select_in_level(vec![t], s, origin));
            return found.unwrap_or_default().into_iter().map(|(t, _)| t).collect();
        }
        let source_dim = self.dim;
        let dims = if s.world_limited() { vec![source_dim] } else { self.loaded_dims() };
'''),
    # A selector sees every entity of the level, thrown things too.
    (GAME + 'commands.rs', '''    fn level_candidates(&self) -> Vec<Target> {
        use horizon_client::projectiles::ProjectileKind as K;
        let mut candidates = Vec::new();
''', '''    fn level_candidates(&self) -> Vec<Target> {
        let mut candidates = Vec::new();
'''),
    (GAME + 'commands.rs', '''                    .filter(|p| {
                        p.is_arrow()
                            || matches!(
                                p.kind, K::LargeFireball { .. } | K::SmallFireball |
                                K::WindCharge | K::BreezeWindCharge
                            )
                    })
                    .map(|p| Target::Projectile(p.id)),
''', '''                    .filter(|p| p.id != 0)
                    .map(|p| Target::Projectile(p.id)),
'''),
]
