// Written by tools/gen_bundle.py.
fn paper_hook_paper_combat(game: &mut crate::game::Game<'_>) {
    use crate::game::paper_fam_paper as paper;
    let Some(change) = paper::combat_wanted() else {
        paper::combat_tick(game.lan_acting);
        return;
    };
    let entry = |row: paper::CombatRow| Entry { damage_type: row.damage_type, damage: row.damage, fall_location: row.fall_location, fall_distance: row.fall_distance, attacker: row.attacker.map(|(id, name)| (id, Text::literal(name))), item: None, direct: None };
    match change {
        paper::CombatChange::Read => {}
        paper::CombatChange::Reset => {
            game.combat.entries.clear();
            game.combat.in_combat = false;
            game.combat.ended = game.world.game_time;
        }
        paper::CombatChange::Set(rows) => game.combat.entries = rows.into_iter().map(entry).collect(),
        paper::CombatChange::Add(row) => {
            if row.attacker.is_some() && !game.combat.in_combat {
                game.combat.in_combat = true;
                game.combat.started = game.world.game_time;
                game.combat.ended = game.combat.started;
            }
            game.combat.last_damage = game.world.game_time;
            game.combat.entries.push(entry(row));
        }
    }
    let rows = game.combat.entries.iter().map(|e| serde_json::json!({ "damage_type": e.damage_type, "damage": e.damage, "fall_location": e.fall_location, "fall_distance": e.fall_distance, "attacker": e.attacker.as_ref().map(|(id, _)| *id) })).collect();
    let duration = if game.combat.in_combat { game.world.game_time - game.combat.started } else { game.combat.ended - game.combat.started };
    paper::combat_read(rows, game.combat.in_combat, game.combat.last_damage, duration);
}
