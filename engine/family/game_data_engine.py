# Data packs on the Paper engine: what plugins and commands read of what the world's packs brought.
GAME = 'horizon-client/src/game/'

EDITS = [
    # ---- Data packs: every synchronized registry and its tags, the enchantment table; a gated rule's node.
    ('horizon-client-render/src/datapack.rs', '''            for f in pack.list_kinds(&[
                "advancement",
                "dialog",
                "function",
                "item_modifier",
                "loot_table",
                "predicate",
                "recipe",
                "tags",
                "trade_set",
                "villager_trade",
            ]) {
''', '''            for f in pack.list_kinds(&[
                "advancement",
                "banner_pattern",
                "cat_sound_variant",
                "cat_variant",
                "chat_type",
                "chicken_sound_variant",
                "chicken_variant",
                "cow_sound_variant",
                "cow_variant",
                "damage_type",
                "decorated_pot_pattern",
                "dialog",
                "enchantment",
                "frog_variant",
                "function",
                "instrument",
                "item_modifier",
                "jukebox_song",
                "loot_table",
                "painting_variant",
                "pig_sound_variant",
                "pig_variant",
                "predicate",
                "recipe",
                "sulfur_cube_archetype",
                "tags",
                "test_environment",
                "test_instance",
                "timeline",
                "trade_set",
                "trim_material",
                "trim_pattern",
                "villager_trade",
                "wolf_sound_variant",
                "wolf_variant",
                "world_clock",
                "worldgen/biome",
                "zombie_nautilus_variant",
            ]) {
'''),
    ('horizon-client-render/src/datapack.rs', '''        "recipe" | "loot_table" | "advancement" | "trade_set" | "villager_trade" | "dialog" => kind,
''', '''        "recipe" | "loot_table" | "advancement" | "trade_set" | "villager_trade" | "dialog"
        | "banner_pattern" | "cat_sound_variant" | "cat_variant" | "chat_type"
        | "chicken_sound_variant" | "chicken_variant" | "cow_sound_variant" | "cow_variant"
        | "damage_type" | "decorated_pot_pattern" | "enchantment" | "frog_variant" | "instrument"
        | "jukebox_song" | "painting_variant" | "pig_sound_variant" | "pig_variant"
        | "sulfur_cube_archetype" | "test_environment" | "test_instance" | "timeline"
        | "trim_material" | "trim_pattern" | "wolf_sound_variant" | "wolf_variant" | "world_clock"
        | "zombie_nautilus_variant" => kind,
        "worldgen" => {
            let path = rest.strip_prefix("biome/")?.strip_suffix(".json")?;
            return Some(format!("worldgen/biome/{ns}:{path}"));
        }
'''),
    ('horizon-client/src/lan/data.rs', '''const PACK_REGISTRIES: &[&str] = &["minecraft:dialog"];

''', ''''''),
    ('horizon-client/src/lan/data.rs', '''            if PACK_REGISTRIES.contains(key) {
                pack_entries(path, overlay, &mut entries);
            }
''', '''            pack_entries(path, overlay, &mut entries);
'''),
    ('horizon-client/src/lan/data.rs', '''        Self::merge_dialog_tags(&mut tags, &root, overlay, &builtin, &dynamic);
''', '''        Self::merge_pack_tags(&mut tags, &root, overlay, &builtin, &dynamic);
'''),
    ('horizon-client/src/lan/data.rs', '''    fn merge_dialog_tags(
        tags: &mut UpdateTags,
        root: &Path,
        overlay: &horizon_world::loot::PackOverlay,
        builtin: &Builtin,
        dynamic: &Dynamic,
    ) {
        let Some(names) = Self::registry_names(builtin, dynamic, "minecraft:dialog") else {
            return;
        };
        let mut files: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        let mut own = BTreeMap::new();
        walk(&root.join("tags/dialog"), "", &mut own);
        for (name, value) in own {
            Self::add_tag_values(files.entry(qualified(&name)).or_default(), &value);
        }
        for (id, packs) in overlay.tag_kind("dialog") {
            for bytes in packs {
                if let Ok(value) = serde_json::from_slice::<Value>(bytes) {
                    Self::add_tag_values(files.entry(id.clone()).or_default(), &value);
                }
            }
        }
        let mut entries = Vec::new();
        for name in files.keys() {
            let mut ids = Vec::new();
            let mut seen = HashSet::new();
            Self::expand_dialog_tag(&files, name, &names, &mut ids, &mut seen);
            entries.push(TagEntry {
                name: name.clone(),
                ids,
            });
        }
        tags.registries.retain(|r| r.registry != "minecraft:dialog");
        if !entries.is_empty() {
            tags.registries.push(RegistryTags {
                registry: "minecraft:dialog".into(),
                tags: entries,
            });
        }
    }

''', '''    fn merge_pack_tags(
        tags: &mut UpdateTags,
        root: &Path,
        overlay: &horizon_world::loot::PackOverlay,
        builtin: &Builtin,
        dynamic: &Dynamic,
    ) {
        let keys = builtin.registries.keys().chain(dynamic.registries.keys());
        for key in keys {
            let path = key.trim_start_matches("minecraft:");
            if overlay.tag_kind(path).next().is_none() {
                continue;
            }
            let Some(names) = Self::registry_names(builtin, dynamic, key) else {
                continue;
            };
            let mut files: BTreeMap<String, Vec<Value>> = BTreeMap::new();
            let mut own = BTreeMap::new();
            walk(&root.join("tags").join(path), "", &mut own);
            for (name, value) in own {
                Self::add_tag_values(files.entry(qualified(&name)).or_default(), &value);
            }
            for (id, packs) in overlay.tag_kind(path) {
                for bytes in packs {
                    if let Ok(value) = serde_json::from_slice::<Value>(bytes) {
                        Self::add_tag_values(files.entry(id.clone()).or_default(), &value);
                    }
                }
            }
            let mut entries = Vec::new();
            for name in files.keys() {
                let mut ids = Vec::new();
                let mut seen = HashSet::new();
                Self::expand_dialog_tag(&files, name, &names, &mut ids, &mut seen);
                entries.push(TagEntry {
                    name: name.clone(),
                    ids,
                });
            }
            match tags.registries.iter_mut().find(|r| r.registry == *key) {
                Some(row) => {
                    row.tags.retain(|tag| !files.contains_key(&tag.name));
                    row.tags.extend(entries);
                }
                None => tags.registries.push(RegistryTags {
                    registry: key.clone(),
                    tags: entries,
                }),
            }
        }
    }

'''),
    ('horizon-client/src/lan/data.rs', '''    pub fn keep_registries(&mut self, resources: &Path, kept: &SyncData) {
        self.dynamic = kept.dynamic.clone();
        self.registries = kept.registries.clone();
        self.tags = Self::load_tags(
            &resources.join("data/minecraft"),
            &self.builtin,
            &self.dynamic,
        );
    }
''', '''    pub fn keep_registries(
        &mut self,
        resources: &Path,
        kept: &SyncData,
        overlay: &horizon_world::loot::PackOverlay,
    ) {
        self.dynamic = kept.dynamic.clone();
        self.registries = kept.registries.clone();
        let root = resources.join("data/minecraft");
        self.tags = Self::load_tags(&root, &self.builtin, &self.dynamic);
        Self::merge_pack_tags(&mut self.tags, &root, overlay, &self.builtin, &self.dynamic);
    }
'''),
    ('horizon-client/src/game/lan.rs', '''            sync.keep_registries(root, kept);
''', '''            sync.keep_registries(root, kept, overlay);
'''),
    ('horizon-items/src/enchant.rs', '''        let lang: HashMap<String, String> = fs::read(
                root.join("assets/minecraft/lang/en_us.json"),
            )
''', '''        for (id, packs) in overlay.kind("enchantment") {
            if let Ok(v) = serde_json::from_slice(horizon_world::loot::PackOverlay::top(packs)) {
                files.insert(id.to_owned(), v);
            }
        }
        let lang: HashMap<String, String> = fs::read(
                root.join("assets/minecraft/lang/en_us.json"),
            )
'''),
    # ---- `END_DATA_PACK_RELOAD`: at the tail of the reload, with whether it worked.
    ('horizon-client/src/game.rs', '''                    Err(e) => {
                        log::warn!(
                            "Horizon: the players are not sent the reloaded data: {e}"
                        )
                    }
''', '''                    Err(e) => {
                        reloaded = false;
                        log::warn!(
                            "Horizon: the players are not sent the reloaded data: {e}"
                        )
                    }
'''),
    ('horizon-client/src/game.rs', '''        self.world_tables = self.build_world_tables();
        horizon_client::structure_blocks::forget_loot_data();
        let kept = self.lan.as_ref().map(|host| host.hub.lan.shared.sync());
''', '''        self.world_tables = self.build_world_tables();
        horizon_client::structure_blocks::forget_loot_data();
        let mut reloaded = true;
        let kept = self.lan.as_ref().map(|host| host.hub.lan.shared.sync());
'''),
    ('horizon-client/src/game.rs', '''        self.adv.reset_tree();
    }
    fn join_world_rest(&mut self) -> Result<()> {
''', '''        self.adv.reset_tree();
        crate::game::paper_events::on_reload_done(self, reloaded);
    }
    fn join_world_rest(&mut self) -> Result<()> {
'''),


    # A command's arguments know what the world's packs brought: tags, and enchantments.
    ('horizon-client/src/game/chat.rs', '''    /// Registry contents for argument parsing (built when a world opens).
    pub(super) fn build_registries(&mut self) {
''', '''    /// What the world's packs brought, over the game's own lists.
    pub(super) fn pack_registries(&mut self) {
        let items = match &self.world_tables {
            Some(tables) => &tables.items,
            None => self.res.items(),
        };
        let sorted = |mut list: Vec<String>| {
            list.sort();
            list.dedup();
            list
        };
        let item_tags = sorted(items.item_tags.keys().map(|k| horizon_client::command::args::normalize(k)).collect());
        let block_tags = sorted(items.block_tags.keys().map(|k| horizon_client::command::args::normalize(k)).collect());
        let enchantments = sorted(items.enchantments.list.iter().map(|e| e.id.to_string()).collect());
        let reg = &mut self.ui.registries;
        if let Some(entry) = reg.by_registry.get_mut("minecraft:item") {
            entry.tags = item_tags.clone();
        }
        if let Some(entry) = reg.by_registry.get_mut("minecraft:block") {
            entry.tags = block_tags.clone();
        }
        if let Some(entry) = reg.by_registry.get_mut("minecraft:enchantment") {
            entry.ids = enchantments.clone();
        }
        reg.item_tags = item_tags;
        reg.block_tags = block_tags;
        reg.enchantments = enchantments;
    }
    /// Registry contents for argument parsing (built when a world opens).
    pub(super) fn build_registries(&mut self) {
'''),
    ('horizon-client/src/game/chat.rs', '''        self.ui.chat.clear();
        self.build_registries();
        self.sync_rules();
''', '''        self.ui.chat.clear();
        self.build_registries();
        self.pack_registries();
        self.sync_rules();
'''),
    ('horizon-client/src/game.rs', '''        self.adv.reset_tree();
        crate::game::paper_events::on_reload_done(self, reloaded);
''', '''        self.adv.reset_tree();
        self.pack_registries();
        crate::game::paper_events::on_reload_done(self, reloaded);
'''),
]
