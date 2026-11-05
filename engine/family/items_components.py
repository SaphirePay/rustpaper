"""Paper's data component classes, by hand: each is the game's own component JSON, and the table says which
field of it each Java method is. The fields are the ones the game's codecs read, checked against their class
files. `sdk_components.py` writes these out as components.rs.

A class is `obj` (the component is a JSON object) or `whole` (it's a list or one value, and `self.0` is all of
it). Values go in as the game's JSON has them: ids as strings, text as components, `head` for a slot.
"""
import re

ROOT = 'io/papermc/paper/datacomponent/item/'
KEYWORDS = {'type', 'match', 'move', 'ref', 'use', 'in', 'loop', 'fn', 'mod', 'box', 'final', 'override', 'abstract', 'where', 'as', 'try', 'dyn', 'do', 'yield', 'macro', 'virtual', 'typeof', 'unsized', 'priv', 'become', 'async', 'await', 'static', 'const', 'enum', 'struct', 'trait', 'impl', 'self', 'super', 'crate', 'extern', 'let', 'mut', 'pub', 'return', 'true', 'false', 'if', 'else', 'for', 'while', 'break', 'continue', 'unsafe'}


def snake(name):
    return re.sub(r'(?<=[a-z0-9])(?=[A-Z])', '_', name).lower()


def rust(name):
    word = snake(name)
    return f'{word}_' if word in KEYWORDS else word


class Class:
    def __init__(self, java, kind='obj', default='[]'):
        self.java, self.kind, self.default = java, kind, default
        self.rust = java.rsplit('/', 1)[-1].replace('$', '')
        self.body, self.builder, self.extra = [], [], []
        self.has_builder = False

    def tag(self, method, owner=None):
        return f'{ROOT}{owner or self.java}.{method}'

    def get(self, *names, **renamed):
        """Getters: the field of the same name in snake_case, or as given."""
        for name in names:
            renamed[name] = snake(name)
        for name, field in renamed.items():
            self.body += [f'    /// bukkit: {self.tag(name)}', f'    pub fn {rust(name)}(&self) -> &Value {{', f'        &self.0["{field}"]', '    }']
        return self

    def all(self, *names):
        """Getters of the whole component."""
        for name in names:
            self.body += [f'    /// bukkit: {self.tag(name)}', f'    pub fn {rust(name)}(&self) -> &Value {{', '        &self.0', '    }']
        return self

    def make(self, name, *fields, constants=''):
        """A factory that gives the class, its arguments being these fields."""
        args = ', '.join(f'{snake(field)}: impl Into<Value>' for field in fields)
        rows = ''.join(f'"{field}": {snake(field)}.into(), ' for field in fields)
        self.body += [f'    /// bukkit: {self.tag(name)}', f'    pub fn {rust(name)}({args}) -> Self {{', f'        Self(json!({{ {constants}{rows}}}))', '    }']
        return self

    def make_whole(self, name):
        """A factory of the whole component."""
        self.body += [f'    /// bukkit: {self.tag(name)}', f'    pub fn {rust(name)}(value: impl Into<Value>) -> Self {{', '        Self(value.into())', '    }']
        return self

    def start(self, name, *fields):
        """A factory that gives the builder, with these fields set."""
        self.has_builder = True
        args = ', '.join(f'{snake(field)}: impl Into<Value>' for field in fields)
        lines = [f'    /// bukkit: {self.tag(name)}', f'    pub fn {rust(name)}({args}) -> {self.rust}Builder {{']
        if not fields:
            lines.append(f'        {self.rust}Builder::default()')
        else:
            lines.append('        let mut built = Map::new();')
            for field in fields:
                key = 'value' if self.kind == 'whole' else field
                lines.append(f'        built.insert("{key}".to_owned(), {snake(field)}.into());')
            lines.append(f'        {self.rust}Builder(built)')
        self.body += lines + ['    }']
        return self

    def set(self, *names, **renamed):
        """Builder calls that set a field."""
        self.has_builder = True
        for name in names:
            renamed[name] = snake(name)
        for name, field in renamed.items():
            self.builder += [f'    /// bukkit: {self.tag(name, self.java + "$Builder")}', f'    pub fn {rust(name)}(mut self, value: impl Into<Value>) -> Self {{', f'        self.0.insert("{field}".to_owned(), value.into());', '        self', '    }']
        return self

    def add(self, name, field, row='value.into()'):
        """A builder call that adds to a list."""
        self.has_builder = True
        self.builder += [f'    /// bukkit: {self.tag(name, self.java + "$Builder")}', f'    pub fn {rust(name)}(mut self, value: impl Into<Value>) -> Self {{', f'        push(&mut self.0, "{field}", {row});', '        self', '    }']
        return self

    def add_all(self, name, field, row='value'):
        """A builder call that adds a list to a list."""
        self.has_builder = True
        self.builder += [f'    /// bukkit: {self.tag(name, self.java + "$Builder")}', f'    pub fn {rust(name)}(mut self, value: impl Into<Value>) -> Self {{', '        if let Value::Array(rows) = value.into() {', '            for value in rows {', f'                push(&mut self.0, "{field}", {row});', '            }', '        }', '        self', '    }']
        return self

    def code(self, text):
        self.body += text.strip('\n').split('\n')
        return self

    def build_code(self, text):
        self.builder += text.strip('\n').split('\n')
        return self

    def render(self):
        out = [f'/// `{(ROOT + self.java).replace("/", ".")}`.', '#[derive(Clone, Debug, PartialEq)]', f'pub struct {self.rust}(pub Value);', '']
        if self.has_builder:
            out += [f'impl {self.rust} {{', f'    /// A builder of this component, as Java\'s no-argument factory gives.', f'    pub fn builder() -> {self.rust}Builder {{', f'        {self.rust}Builder::default()', '    }', '}', '']
        if self.body:
            out += [f'impl {self.rust} {{'] + self.body + ['}', '']
        out += self.extra
        if self.has_builder:
            build = f'        {self.rust}(self.0.remove("value").unwrap_or_else(|| json!({self.default})))' if self.kind == 'whole' else f'        {self.rust}(Value::Object(self.0))'
            take = 'built.as_object().cloned().unwrap_or_default()' if self.kind == 'obj' else 'Map::from_iter([("value".to_owned(), built.clone())])'
            mutable = 'mut self' if self.kind == 'whole' else 'self'
            out += [f'/// `{(ROOT + self.java + "$Builder").replace("/", ".")}`.', '#[derive(Clone, Debug, Default, PartialEq)]', f'pub struct {self.rust}Builder(pub Map<String, Value>);', '', f'impl {self.rust}Builder {{',
                    '    /// `BuildableDataComponent.toBuilder`: from the built value.', f'    pub fn from_built(built: &Value) -> Self {{', f'        Self({take})', '    }'] + self.builder + [
                    f'    /// bukkit: {self.tag("build", self.java + "$Builder")}', f'    pub fn build({mutable}) -> {self.rust} {{', build, '    }', '}', '']
        return out


def table():
    rows = []
    def new(java, kind='obj', default='[]'):
        row = Class(java, kind, default)
        rows.append(row)
        return row

    new('AttackRange').start('attackRange').get('minReach', 'maxReach', 'minCreativeReach', 'maxCreativeReach', 'hitboxMargin', 'mobFactor').set('minReach', 'maxReach', 'minCreativeReach', 'maxCreativeReach', 'hitboxMargin', 'mobFactor')

    for java, make, getter, one in (('BannerPatternLayers', 'bannerPatternLayers', 'patterns', None), ('BundleContents', 'bundleContents', 'contents', None), ('ChargedProjectiles', 'chargedProjectiles', 'projectiles', None), ('SuspiciousStewEffects', 'suspiciousStewEffects', 'effects', None)):
        new(java, 'whole').make_whole(make).all(getter).add('add', 'value').add_all('addAll', 'value')

    new('BlockItemDataProperties').start('blockItemStateProperties').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/BlockItemDataProperties.applyTo
    ///
    /// The properties this block has of the component's.
    pub fn apply_to(&self, data: &mut crate::block_data::BlockData) {
        for (key, value) in self.0.as_object().into_iter().flatten() {
            if data.property(key).is_some() {
                data.set_property(key, value.as_str().unwrap_or_default());
            }
        }
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlockItemDataProperties.createBlockData
    pub fn create_block_data(&self, block: &str) -> crate::block_data::BlockData {
        let mut data = crate::block_data::BlockData::parse(block);
        self.apply_to(&mut data);
        data
    }
''')

    new('BlocksAttacks').start('blocksAttacks').get('blockDelaySeconds', 'disableCooldownScale', 'damageReductions', 'itemDamage', 'bypassedBy', 'blockSound', disableSound='disabled_sound').set('blockDelaySeconds', 'disableCooldownScale', 'damageReductions', 'itemDamage', 'bypassedBy', 'blockSound', disableSound='disabled_sound').add('addDamageReduction', 'damage_reductions')

    new('Consumable').start('consumable').get('consumeSeconds', 'animation', 'sound', 'hasConsumeParticles', consumeEffects='on_consume_effects').set('consumeSeconds', 'animation', 'sound', 'hasConsumeParticles', effects='on_consume_effects').add('addEffect', 'on_consume_effects').add_all('addEffects', 'on_consume_effects')

    c = new('CustomModelData').start('customModelData').get('floats', 'flags', 'strings', 'colors')
    for one, many, field in (('addFloat', 'addFloats', 'floats'), ('addFlag', 'addFlags', 'flags'), ('addString', 'addStrings', 'strings'), ('addColor', 'addColors', 'colors')):
        c.add(one, field).add_all(many, field)

    new('DamageResistant').make('damageResistant', 'types').get('types')
    new('DeathProtection').make('deathProtection', 'death_effects').get(deathEffects='death_effects').add('addEffect', 'death_effects').add_all('addEffects', 'death_effects')
    new('DyedItemColor', 'whole', 'null').make_whole('dyedItemColor').all('color').set(color='value')
    new('Enchantable').make('enchantable', 'value').get('value')

    fields = ('assetId', 'cameraOverlay', 'allowedEntities', 'dispensable', 'swappable', 'damageOnHurt', 'equipOnInteract', 'canBeSheared', 'equipSound')
    new('Equippable').start('equippable', 'slot').get('slot', *fields, shearSound='shearing_sound').set(*fields, shearSound='shearing_sound')

    new('Fireworks').make('fireworks', 'explosions', 'flight_duration').get(effects='explosions', flightDuration='flight_duration').set('flightDuration').add('addEffect', 'explosions').add_all('addEffects', 'explosions')
    new('FoodProperties').start('food').get('nutrition', 'saturation', 'canAlwaysEat').set('nutrition', 'saturation', 'canAlwaysEat')
    new('ItemAdventurePredicate', 'whole').make_whole('itemAdventurePredicate').all('predicates').add('addPredicate', 'value').add_all('addPredicates', 'value')
    new('ItemArmorTrim', 'whole', 'null').start('itemArmorTrim', 'armor_trim').all('armorTrim').set(armorTrim='value')

    new('ItemAttributeModifiers', 'whole').start('itemAttributes').all('modifiers').add('addModifier', 'value')
    new('ItemAttributeModifiers$Entry').get('display', attribute='type', getGroup='slot').all('modifier')

    new('ItemContainerContents', 'whole').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/ItemContainerContents.containerContents
    ///
    /// The stacks in slot order, from slot 0.
    pub fn container_contents(stacks: impl Into<Value>) -> Self {
        let rows: Vec<Value> = match stacks.into() {
            Value::Array(stacks) => stacks.into_iter().enumerate().filter(|(_, stack)| !stack.is_null()).map(|(slot, item)| json!({ "slot": slot, "item": item })).collect(),
            _ => Vec::new(),
        };
        Self(Value::Array(rows))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemContainerContents.contents
    ///
    /// The stacks by slot, up to the last that has one; `Null` for an empty slot.
    pub fn contents(&self) -> Vec<Value> {
        let rows = self.0.as_array().cloned().unwrap_or_default();
        let size = rows.iter().filter_map(|row| row["slot"].as_u64()).max().map_or(0, |last| last as usize + 1);
        let mut stacks = vec![Value::Null; size];
        for row in rows {
            if let Some(place) = row["slot"].as_u64().and_then(|slot| stacks.get_mut(slot as usize)) {
                *place = row["item"].clone();
            }
        }
        stacks
    }
''').build_code('''
    fn put(&mut self, item: Value) {
        let slot = self.0.get("value").and_then(Value::as_array).map_or(0, Vec::len);
        push(&mut self.0, "value", json!({ "slot": slot, "item": item }));
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemContainerContents$Builder.add
    pub fn add(mut self, value: impl Into<Value>) -> Self {
        self.put(value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemContainerContents$Builder.addAll
    pub fn add_all(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(stacks) = value.into() {
            for stack in stacks {
                self.put(stack);
            }
        }
        self
    }
''').has_builder = True

    new('ItemEnchantments').make_whole('itemEnchantments').all('enchantments').build_code('''
    /// bukkit: io/papermc/paper/datacomponent/item/ItemEnchantments$Builder.add
    pub fn add(mut self, enchantment: &str, level: i32) -> Self {
        self.0.insert(enchantment.to_owned(), json!(level));
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemEnchantments$Builder.addAll
    pub fn add_all(mut self, enchantments: impl Into<Value>) -> Self {
        if let Value::Object(rows) = enchantments.into() {
            self.0.extend(rows);
        }
        self
    }
''').has_builder = True

    lore = new('ItemLore', 'whole').make_whole('lore').all('lines').add('addLine', 'value').add_all('addLines', 'value').set(lines='value')
    lore.code('''
    /// bukkit: io/papermc/paper/datacomponent/item/ItemLore.styledLines
    ///
    /// The lines under lore's own style, dark purple italic, which a line's own styles beat.
    pub fn styled_lines(&self) -> Vec<Value> {
        self.0.as_array().into_iter().flatten().map(|line| {
            let mut styled = if line.is_object() { line.clone() } else { json!({ "text": line }) };
            for (key, value) in [("color", json!("dark_purple")), ("italic", json!(true))] {
                if styled.get(key).is_none() {
                    styled[key] = value;
                }
            }
            styled
        }).collect()
    }
''')

    new('JukeboxPlayable', 'whole', 'null').start('jukeboxPlayable', 'jukebox_song').all('jukeboxSong').set(jukeboxSong='value')

    fields = ('contactCooldownTicks', 'delayTicks', 'dismountConditions', 'knockbackConditions', 'damageConditions', 'forwardMovement', 'damageMultiplier', 'sound', 'hitSound')
    new('KineticWeapon').start('kineticWeapon').get(*fields).set(*fields).code('''
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon.condition
    pub fn condition(max_duration_ticks: i32, min_speed: f32, min_relative_speed: f32) -> KineticWeaponCondition {
        KineticWeaponCondition(json!({ "max_duration_ticks": max_duration_ticks, "min_speed": min_speed, "min_relative_speed": min_relative_speed }))
    }
''')
    new('KineticWeapon$Condition').get('maxDurationTicks', 'minSpeed', 'minRelativeSpeed')

    new('LodestoneTracker').make('lodestoneTracker', 'target', 'tracked').get('tracked', location='target').set('tracked', location='target')

    new('MapDecorations').make_whole('mapDecorations').all('decorations').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations.decoration
    pub fn decoration(&self, id: &str) -> Option<&Value> {
        self.0.get(id)
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations.decorationEntry
    pub fn decoration_entry(kind: impl Into<Value>, x: f64, z: f64, rotation: f32) -> MapDecorationsDecorationEntry {
        MapDecorationsDecorationEntry(json!({ "type": kind.into(), "x": x, "z": z, "rotation": rotation }))
    }
''').build_code('''
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations$Builder.put
    pub fn put(mut self, id: &str, entry: impl Into<Value>) -> Self {
        self.0.insert(id.to_owned(), entry.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations$Builder.putAll
    pub fn put_all(mut self, entries: impl Into<Value>) -> Self {
        if let Value::Object(rows) = entries.into() {
            self.0.extend(rows);
        }
        self
    }
''').has_builder = True
    new('MapDecorations$DecorationEntry').get('rotation', 'x', 'z', **{'type': 'type'})

    new('MapId', 'whole', 'null').make_whole('mapId').all('id')
    new('MobVisibility').make('mobVisibility', 'targeting_entity_types', 'visibility').get('targetingEntityTypes', 'visibility')
    new('OminousBottleAmplifier', 'whole', 'null').make_whole('amplifier').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/OminousBottleAmplifier.amplifier
    pub fn value(&self) -> &Value {
        &self.0
    }
''')
    new('PiercingWeapon').start('piercingWeapon').get('dealsKnockback', 'dismounts', 'sound', 'hitSound').set('dealsKnockback', 'dismounts', 'sound', 'hitSound')

    pot = new('PotDecorations', 'whole')
    pot.code('''
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations.potDecorations
    ///
    /// The game keeps the four as a list; `Null` is the plain side, a brick's.
    pub fn pot_decorations(back: Value, left: Value, right: Value, front: Value) -> Self {
        let side = |value: Value| if value.is_null() { json!("minecraft:brick") } else { value };
        Self(json!([side(back), side(left), side(right), side(front)]))
    }
    fn side(&self, index: usize) -> Option<&Value> {
        self.0.get(index).filter(|side| side.as_str() != Some("minecraft:brick"))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations.back
    pub fn back(&self) -> Option<&Value> {
        self.side(0)
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations.left
    pub fn left(&self) -> Option<&Value> {
        self.side(1)
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations.right
    pub fn right(&self) -> Option<&Value> {
        self.side(2)
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations.front
    pub fn front(&self) -> Option<&Value> {
        self.side(3)
    }
''')
    pot.has_builder = True
    for index, name in enumerate(('back', 'left', 'right', 'front')):
        pot.build_code(f'''
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations$Builder.{name}
    pub fn {name}(mut self, value: impl Into<Value>) -> Self {{
        let sides = self.0.entry("value".to_owned()).or_insert_with(|| json!(["minecraft:brick", "minecraft:brick", "minecraft:brick", "minecraft:brick"]));
        let value = value.into();
        sides[{index}] = if value.is_null() {{ json!("minecraft:brick") }} else {{ value }};
        self
    }}
''')

    new('PotionContents').start('potionContents').get('potion', 'customColor', 'customEffects', 'customName').set('potion', 'customColor', 'customName').add('addCustomEffect', 'custom_effects').add_all('addCustomEffects', 'custom_effects').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents.allEffects
    ///
    /// The potion's own effects, then the custom ones.
    pub fn all_effects(&self) -> Vec<Value> {
        let own = self.0["potion"].as_str().map_or(Value::Null, |potion| horizon_mod::query(json!({ "op": "items_potion", "id": potion })));
        own["effects"].as_array().into_iter().chain(self.0["custom_effects"].as_array()).flatten().cloned().collect()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents.computeEffectiveColor
    ///
    /// `PotionContents.getColor`: the custom colour, or the visible effects' colours by level.
    pub fn compute_effective_color(&self) -> crate::Color {
        if let Some(color) = self.0["custom_color"].as_i64() {
            return crate::Color::from_rgb(((color >> 16) & 255) as u8, ((color >> 8) & 255) as u8, (color & 255) as u8);
        }
        let (mut sum, mut total) = ([0_i64; 3], 0_i64);
        for effect in self.all_effects() {
            if effect["show_particles"] == false || effect["visible"] == false {
                continue;
            }
            let Some(id) = effect["id"].as_str() else {
                continue;
            };
            let color = crate::fam_items::PotionEffectType::of(id).get_color();
            let level = effect["amplifier"].as_i64().unwrap_or(0) + 1;
            sum[0] += level * i64::from(color.get_red());
            sum[1] += level * i64::from(color.get_green());
            sum[2] += level * i64::from(color.get_blue());
            total += level;
        }
        if total == 0 {
            // `BASE_POTION_COLOR`.
            return crate::Color::from_rgb(0x38, 0x5D, 0xC6);
        }
        crate::Color::from_rgb((sum[0] / total) as u8, (sum[1] / total) as u8, (sum[2] / total) as u8)
    }
''')

    new('Repairable').make('repairable', 'items').get(types='items')

    patch = ('cape', 'elytra', 'model')
    new('ResolvableProfile').make_whole('resolvableProfile').get('name', 'properties', uuid='id').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile.dynamic
    ///
    /// A name or an id alone, no properties: the game looks it up each time it's shown.
    pub fn dynamic(&self) -> bool {
        let empty = self.0["properties"].as_array().is_none_or(Vec::is_empty) && self.0["properties"].as_object().is_none_or(Map::is_empty);
        empty && self.0["name"].is_string() != !self.0["id"].is_null()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile.resolve
    ///
    /// Java's future is `callback`, on the server thread. A dynamic profile is looked up, as `PlayerProfile.update`
    /// does; a static one comes as it is, with an empty name or the offline id of its name in place of what it lacks.
    pub fn resolve(&self, callback: impl FnOnce(crate::player::PlayerProfile) + Send + 'static) {
        let id = crate::data::uuid(&self.0["id"]).unwrap_or_default();
        let name = self.0["name"].as_str().unwrap_or_default().to_owned();
        let rows = self.0["properties"].as_array().cloned().unwrap_or_default();
        let properties = rows.iter().map(|row| (row["name"].as_str().unwrap_or_default().to_owned(), row["value"].as_str().unwrap_or_default().to_owned(), row["signature"].as_str().map(str::to_owned))).collect();
        let mut profile = crate::player::PlayerProfile { name, id, properties };
        if self.dynamic() {
            profile.update(callback);
            return;
        }
        if profile.id.is_empty() {
            profile.id = horizon_mod::query(json!({ "op": "paper_offline_uuid", "name": profile.name })).as_str().unwrap_or_default().to_owned();
        }
        callback(profile);
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile.skinPatch
    pub fn skin_patch(&self) -> ResolvableProfileSkinPatch {
        let mut patch = Map::new();
        for key in ["texture", "cape", "elytra", "model"] {
            if let Some(value) = self.0.get(key) {
                patch.insert(key.to_owned(), value.clone());
            }
        }
        ResolvableProfileSkinPatch(Value::Object(patch))
    }
''').set('name', uuid='id').add('addProperty', 'properties').add_all('addProperties', 'properties').build_code('''
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$Builder.skinPatch
    pub fn skin_patch(mut self, patch: &ResolvableProfileSkinPatch) -> Self {
        for (key, value) in patch.0.as_object().into_iter().flatten() {
            self.0.insert(key.clone(), value.clone());
        }
        self
    }
''')
    new('ResolvableProfile$SkinPatch').get(*patch, body='texture').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatch.empty
    pub fn empty() -> Self {
        Self(json!({}))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatch.skinPatch
    pub fn skin_patch() -> ResolvableProfileSkinPatchBuilder {
        ResolvableProfileSkinPatchBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatch.isEmpty
    pub fn is_empty(&self) -> bool {
        self.0.as_object().is_none_or(Map::is_empty)
    }
''').extra += '''/// `io.papermc.paper.datacomponent.item.ResolvableProfile$SkinPatchBuilder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvableProfileSkinPatchBuilder(pub Map<String, Value>);

impl ResolvableProfileSkinPatchBuilder {
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatchBuilder.body
    pub fn body(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("texture".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatchBuilder.cape
    pub fn cape(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("cape".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatchBuilder.elytra
    pub fn elytra(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("elytra".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatchBuilder.model
    pub fn model(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("model".to_owned(), value.into());
        self
    }
    /// The patch built.
    pub fn build(self) -> ResolvableProfileSkinPatch {
        ResolvableProfileSkinPatch(Value::Object(self.0))
    }
}
'''.split('\n')

    new('SeededContainerLoot').make('seededContainerLoot', 'loot_table', 'seed').get('lootTable', 'seed').set('lootTable', 'seed')

    new('SignText').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/SignText.signText
    pub fn sign_text(lines: impl Into<Value>) -> Self {
        Self(json!({ "messages": lines.into() }))
    }
''').get('color', 'hasGlowingText', lines='messages').set('color', 'hasGlowingText', lines='messages').build_code('''
    /// bukkit: io/papermc/paper/datacomponent/item/SignText$Builder.line
    pub fn line(mut self, index: usize, line: impl Into<Value>) -> Self {
        let lines = self.0.entry("messages".to_owned()).or_insert_with(|| json!(["", "", "", ""]));
        lines[index] = line.into();
        self
    }
''')

    new('SulfurCubeContent', 'whole', 'null').make_whole('sulfurCubeContent').all('absorbedItem')
    new('SwingAnimation').start('swingAnimation').get('duration', **{'type': 'type'}).set('duration', **{'type': 'type'})

    new('Tool').start('tool').get('rules', 'defaultMiningSpeed', 'damagePerBlock', 'canDestroyBlocksInCreative').set('defaultMiningSpeed', 'damagePerBlock', 'canDestroyBlocksInCreative').add('addRule', 'rules').add_all('addRules', 'rules').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/Tool.rule
    pub fn rule(blocks: impl Into<Value>, speed: impl Into<Value>, correct_for_drops: impl Into<Value>) -> ToolRule {
        ToolRule(json!({ "blocks": blocks.into(), "speed": speed.into(), "correct_for_drops": correct_for_drops.into() }))
    }
''')
    new('Tool$Rule').get('blocks', 'speed', 'correctForDrops')

    new('TooltipDisplay').start('tooltipDisplay').get('hideTooltip', 'hiddenComponents').set('hideTooltip', 'hiddenComponents').add_all('addHiddenComponents', 'hidden_components')
    new('UseCooldown').start('useCooldown', 'seconds').get('seconds', 'cooldownGroup').set('cooldownGroup')
    new('UseEffects').start('useEffects').get('canSprint', 'interactVibrations', 'speedMultiplier').set('canSprint', 'interactVibrations', 'speedMultiplier')
    new('UseRemainder', 'whole', 'null').make_whole('useRemainder').all('transformInto')
    new('VillagerFood').make('villagerFood', 'nutrition').get('nutrition')
    new('Weapon').start('weapon').get('itemDamagePerAttack', 'disableBlockingForSeconds').set('itemDamagePerAttack', 'disableBlockingForSeconds')

    new('WritableBookContent').start('writeableBookContent').get('pages').add('addPage', 'pages', 'json!({ "raw": value.into() })').add_all('addPages', 'pages', 'json!({ "raw": value })').add('addFilteredPage', 'pages').add_all('addFilteredPages', 'pages')

    new('WrittenBookContent').start('writtenBookContent').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent.title
    pub fn title(&self) -> &Value {
        &self.0["title"]
    }
''').get('author', 'generation', 'pages', 'resolved').set('author', 'generation', 'resolved', filteredTitle='title').add('addPage', 'pages', 'json!({ "raw": value.into() })').add_all('addPages', 'pages', 'json!({ "raw": value })').add('addFilteredPage', 'pages').add_all('addFilteredPages', 'pages').build_code('''
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent$Builder.title
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.insert("title".to_owned(), json!({ "raw": title.into() }));
        self
    }
''')

    new('attribute/AttributeModifierDisplay').make('reset', constants='"type": "default", ').make('hidden', constants='"type": "hidden", ').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/attribute/AttributeModifierDisplay.override
    pub fn override_(text: impl Into<Value>) -> AttributeModifierDisplayOverrideText {
        AttributeModifierDisplayOverrideText(json!({ "type": "override", "value": text.into() }))
    }
''')
    new('attribute/AttributeModifierDisplay$OverrideText').get(text='value')

    new('blocksattacks/DamageReduction').start('damageReduction').get('base', 'factor', 'horizontalBlockingAngle', **{'type': 'type'}).set('base', 'factor', 'horizontalBlockingAngle', **{'type': 'type'})
    new('blocksattacks/ItemDamageFunction').start('itemDamageFunction').get('threshold', 'base', 'factor').set('threshold', 'base', 'factor').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/ItemDamageFunction.damageToApply
    pub fn damage_to_apply(&self, blocked: f32) -> i32 {
        let number = |key: &str| self.0[key].as_f64().unwrap_or(0.0) as f32;
        if blocked < number("threshold") { 0 } else { (number("base") + number("factor") * blocked).floor() as i32 }
    }
''')

    new('consumable/ConsumeEffect').code('''
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect.applyStatusEffects
    pub fn apply_status_effects(effects: impl Into<Value>, probability: f32) -> ConsumeEffectApplyStatusEffects {
        ConsumeEffectApplyStatusEffects(json!({ "type": "minecraft:apply_effects", "effects": effects.into(), "probability": probability }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect.removeEffects
    pub fn remove_effects(effects: impl Into<Value>) -> ConsumeEffectRemoveStatusEffects {
        ConsumeEffectRemoveStatusEffects(json!({ "type": "minecraft:remove_effects", "effects": effects.into() }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect.clearAllStatusEffects
    pub fn clear_all_status_effects() -> Self {
        Self(json!({ "type": "minecraft:clear_all_effects" }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect.teleportRandomlyEffect
    pub fn teleport_randomly_effect(diameter: f32) -> ConsumeEffectTeleportRandomly {
        ConsumeEffectTeleportRandomly(json!({ "type": "minecraft:teleport_randomly", "diameter": diameter }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect.playSoundConsumeEffect
    pub fn play_sound_consume_effect(sound: impl Into<Value>) -> ConsumeEffectPlaySound {
        ConsumeEffectPlaySound(json!({ "type": "minecraft:play_sound", "sound": sound.into() }))
    }
''')
    new('consumable/ConsumeEffect$ApplyStatusEffects').get('effects', 'probability')
    new('consumable/ConsumeEffect$PlaySound').get('sound')
    new('consumable/ConsumeEffect$RemoveStatusEffects').get(removeEffects='effects')
    new('consumable/ConsumeEffect$TeleportRandomly').get('diameter', 'directionalParticles')
    return rows


def render():
    out = ['//! Written by plugin-tools/tools/sdk_components.py from family/items_components.py: do not edit.', '//!', '//! Each class is the game\'s component JSON. Values go in as the game has them: ids as strings, text as', '//! components, `head` for a slot. `stack.set_component(id, value.0)` puts one on an item.', '#![allow(missing_docs, clippy::wrong_self_convention)]', 'use horizon_mod::Value;', 'use serde_json::{Map, json};', '']
    for row in table():
        out += row.render()
    out += ['fn push(fields: &mut Map<String, Value>, key: &str, value: Value) {', '    let list = fields.entry(key.to_owned()).or_insert_with(|| json!([]));', '    if let Some(list) = list.as_array_mut() {', '        list.push(value);', '    }', '}', '']
    return '\n'.join(out)
