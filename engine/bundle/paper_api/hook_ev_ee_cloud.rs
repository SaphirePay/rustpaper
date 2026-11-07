// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_cloud(game: &mut crate::game::Game<'_>, c: &mut Breath, players: &[FightPlayer]) -> ::core::ops::ControlFlow<bool> {
    // Every event on this hook answers from `ee_refused` alone (`verdict` first: tools' echoes check), so its listeners never ran here; the round trip to each plugin is gone.
    if crate::game::paper_bridge::hears("ee_cloud") && ({ let able: Vec<u32> = c.potion.as_ref().map_or_else(Vec::new, |potion| game.mobs.iter().filter(|m| potion.effects.iter().any(|(id, ..)| mob_can_be_affected(&game.stations().enchantments.entity_tags, &m.kind.id, id))).map(|m| m.id).collect()); crate::game::paper_fam_ev_entity::cloud(game, c, players, &able) }) {
        return ::core::ops::ControlFlow::Break({ if let Some((mobs, guests)) = crate::game::paper_fam_ev_entity::spared(game) { if let Some(potion) = c.potion.as_mut() { let until = c.age + 1 + REAPPLICATION_DELAY; potion.victims.extend(mobs.into_iter().map(|id| (Victim::Mob(id), until))); potion.victims.extend(guests.into_iter().map(|conn| (Victim::Player(FightWho::Guest(conn)), until))); } game.tick_potion_cloud(c, players) } else { c.age += 1; true } });
    }
    ::core::ops::ControlFlow::Continue(())
}
