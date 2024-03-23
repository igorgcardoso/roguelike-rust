use super::Raws;
use crate::{attribute_bonus, components::*, mana_at_level, npc_hp, random_table::RandomTable};
use regex::Regex;
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};
use std::collections::{HashMap, HashSet};

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
    Equipped { by: Entity },
    Carried { by: Entity },
}

pub struct RawMaster {
    raws: Raws,
    item_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
    loot_index: HashMap<String, usize>,
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws: Raws {
                items: Vec::new(),
                mobs: Vec::new(),
                props: Vec::new(),
                spawn_table: Vec::new(),
                loot_tables: Vec::new(),
            },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
            loot_index: HashMap::new(),
        }
    }

    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        let mut used_names: HashSet<String> = HashSet::new();

        for (i, item) in self.raws.items.iter().enumerate() {
            if used_names.contains(&item.name) {
                rltk::console::log(format!(
                    "WARNING - duplicate item name in raws [{}]",
                    item.name
                ));
            }
            self.item_index.insert(item.name.clone(), i);
            used_names.insert(item.name.clone());
        }
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            if used_names.contains(&mob.name) {
                rltk::console::log(format!(
                    "WARNING - duplicate mob name in raws [{}]",
                    mob.name
                ));
            }
            self.mob_index.insert(mob.name.clone(), i);
            used_names.insert(mob.name.clone());
        }
        for (i, prop) in self.raws.props.iter().enumerate() {
            if used_names.contains(&prop.name) {
                rltk::console::log(format!(
                    "WARNING - duplicate prop name in raws [{}]",
                    prop.name
                ));
            }
            self.prop_index.insert(prop.name.clone(), i);
            used_names.insert(prop.name.clone());
        }

        for spawn in self.raws.spawn_table.iter() {
            if !used_names.contains(&spawn.name) {
                rltk::console::log(format!(
                    "WARNING - spawn table entry [{}] does not have a corresponding item, mob, or prop",
                    spawn.name
                ));
            }
        }

        for (idx, loot) in self.raws.loot_tables.iter().enumerate() {
            self.loot_index.insert(loot.name.clone(), idx);
        }
    }
}

pub fn string_to_slot(slot: &str) -> EquipmentSlot {
    match slot {
        "Shield" => EquipmentSlot::Shield,
        "Head" => EquipmentSlot::Head,
        "Torso" => EquipmentSlot::Torso,
        "Legs" => EquipmentSlot::Legs,
        "Feet" => EquipmentSlot::Feet,
        "Hands" => EquipmentSlot::Hands,
        "Melee" => EquipmentSlot::Melee,
        _ => {
            rltk::console::log(format!("Warning: unknown equipment slot type [{}]", slot));
            EquipmentSlot::Melee
        }
    }
}

fn find_slot_for_equippable_item(tag: &str, raws: &RawMaster) -> EquipmentSlot {
    if !raws.item_index.contains_key(tag) {
        panic!("Trying to equip an unknown item: {}", tag);
    }
    let item_index = raws.item_index[tag];
    let item = &raws.raws.items[item_index];
    if let Some(_weapon) = &item.weapon {
        return EquipmentSlot::Melee;
    }
    if let Some(wearable) = &item.wearable {
        return string_to_slot(&wearable.slot);
    }
    panic!("Trying to equip {}, but it has no slot tag.", tag);
}

fn spawn_position<'a>(
    pos: SpawnType,
    new_entity: EntityBuilder<'a>,
    tag: &str,
    raws: &RawMaster,
) -> EntityBuilder<'a> {
    let mut entity_builder = new_entity;
    match pos {
        SpawnType::AtPosition { x, y } => entity_builder = entity_builder.with(Position { x, y }),
        SpawnType::Carried { by } => entity_builder = entity_builder.with(InBackpack { owner: by }),
        SpawnType::Equipped { by } => {
            let slot = find_slot_for_equippable_item(tag, raws);
            entity_builder = entity_builder.with(Equipped { owner: by, slot });
        }
    }

    entity_builder
}

fn get_renderable_component(
    renderable: &super::item_structs::Renderable,
) -> crate::components::Renderable {
    crate::components::Renderable {
        glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
        fg: rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg: rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order: renderable.order,
    }
}

pub fn get_item_drop(
    raws: &RawMaster,
    rng: &mut rltk::RandomNumberGenerator,
    table: &str,
) -> Option<String> {
    if raws.loot_index.contains_key(table) {
        let mut random_table = RandomTable::new();
        let available_options = &raws.raws.loot_tables[raws.loot_index[table]];
        for item in available_options.drops.iter() {
            random_table = random_table.add(item.name.clone(), item.weight);
        }
        return Some(random_table.roll(rng));
    }
    None
}
pub fn parse_dice_string(dice: &str) -> (i32, i32, i32) {
    lazy_static! {
        static ref DICE_RE: Regex = Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").unwrap();
    }

    let mut n_dice = 1;
    let mut die_type = 4;
    let mut die_bonus = 6;
    for cap in DICE_RE.captures_iter(dice) {
        if let Some(group) = cap.get(1) {
            n_dice = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(2) {
            die_type = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(3) {
            die_bonus = group.as_str().parse::<i32>().expect("Not a digit");
        }
    }
    (n_dice, die_type, die_bonus)
}

pub fn spawn_named_item(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];

        let mut entity_builder = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        // Spawn in the specified location
        entity_builder = spawn_position(pos, entity_builder, key, raws);

        // Renderable
        if let Some(renderable) = &item_template.renderable {
            entity_builder = entity_builder.with(get_renderable_component(renderable));
        }

        entity_builder = entity_builder.with(Name {
            name: item_template.name.clone(),
        });
        entity_builder = entity_builder.with(crate::components::Item {});

        if let Some(consumable) = &item_template.consumable {
            entity_builder = entity_builder.with(Consumable {});
            for effect in consumable.effects.iter() {
                let effect_name = effect.0.as_str();
                match effect_name {
                    "provides_healing" => {
                        entity_builder = entity_builder.with(ProvidesHealing {
                            heal_amount: effect.1.parse::<i32>().unwrap(),
                        });
                    }
                    "ranged" => {
                        entity_builder = entity_builder.with(Ranged {
                            range: effect.1.parse::<i32>().unwrap(),
                        });
                    }
                    "damage" => {
                        entity_builder = entity_builder.with(InflictsDamage {
                            damage: effect.1.parse::<i32>().unwrap(),
                        });
                    }
                    "area_of_effect" => {
                        entity_builder = entity_builder.with(AreaOfEffect {
                            radius: effect.1.parse::<i32>().unwrap(),
                        });
                    }
                    "confusion" => {
                        entity_builder = entity_builder.with(Confusion {
                            turns: effect.1.parse::<i32>().unwrap(),
                        });
                    }
                    "magic_mapping" => {
                        entity_builder = entity_builder.with(MagicMapper {});
                    }
                    "food" => {
                        entity_builder = entity_builder.with(ProvidesFood {});
                    }
                    _ => {
                        rltk::console::log(format!(
                            "Warning: consumable effect {} not implemented",
                            effect_name
                        ));
                    }
                }
            }
        }

        if let Some(weapon) = &item_template.weapon {
            entity_builder = entity_builder.with(Equippable {
                slot: EquipmentSlot::Melee,
            });
            let (n_dice, die_type, bonus) = parse_dice_string(&weapon.base_damage);
            let mut wpn = MeleeWeapon {
                attribute: WeaponAttribute::Might,
                damage_n_dice: n_dice,
                damage_die_type: die_type,
                damage_bonus: bonus,
                hit_bonus: weapon.hit_bonus,
            };
            match weapon.attribute.as_str() {
                "Quickness" => wpn.attribute = WeaponAttribute::Quickness,
                _ => wpn.attribute = WeaponAttribute::Might,
            }
            entity_builder = entity_builder.with(wpn);
        }

        if let Some(wearable) = &item_template.wearable {
            let slot = string_to_slot(&wearable.slot);
            entity_builder = entity_builder.with(Equippable { slot });
            entity_builder = entity_builder.with(Wearable {
                armor_class: wearable.armor_class,
                slot,
            });
        }

        return Some(entity_builder.build());
    }
    None
}

pub fn spawn_named_mob(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];

        let mut entity_builder = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        // Spawn in the specified location
        entity_builder = spawn_position(pos, entity_builder, key, raws);

        // Renderable
        if let Some(renderable) = &mob_template.renderable {
            entity_builder = entity_builder.with(get_renderable_component(renderable));
        }

        entity_builder = entity_builder.with(Name {
            name: mob_template.name.clone(),
        });

        match mob_template.ai.as_ref() {
            "melee" => entity_builder = entity_builder.with(Monster {}),
            "bystander" => entity_builder = entity_builder.with(Bystander {}),
            "vendor" => entity_builder = entity_builder.with(Vendor {}),
            "carnivore" => entity_builder = entity_builder.with(Carnivore {}),
            "herbivore" => entity_builder = entity_builder.with(Herbivore {}),
            _ => {}
        }

        if let Some(quips) = &mob_template.quips {
            entity_builder = entity_builder.with(Quips {
                available: quips.clone(),
            });
        }

        if mob_template.blocks_tile {
            entity_builder = entity_builder.with(BlocksTile {});
        }

        let mut mob_fitness = 11;
        let mut mob_intelligence = 11;

        let mut attributes = Attributes {
            might: Attribute {
                base: 11,
                modifiers: 0,
                bonus: attribute_bonus(11),
            },
            fitness: Attribute {
                base: mob_fitness,
                modifiers: 0,
                bonus: attribute_bonus(mob_fitness),
            },
            quickness: Attribute {
                base: 11,
                modifiers: 0,
                bonus: attribute_bonus(11),
            },
            intelligence: Attribute {
                base: mob_intelligence,
                modifiers: 0,
                bonus: attribute_bonus(mob_intelligence),
            },
        };

        if let Some(might) = mob_template.attributes.might {
            attributes.might = Attribute {
                base: might,
                modifiers: 0,
                bonus: attribute_bonus(might),
            };
        }
        if let Some(fitness) = mob_template.attributes.fitness {
            attributes.fitness = Attribute {
                base: fitness,
                modifiers: 0,
                bonus: attribute_bonus(fitness),
            };
            mob_fitness = fitness;
        }
        if let Some(quickness) = mob_template.attributes.quickness {
            attributes.quickness = Attribute {
                base: quickness,
                modifiers: 0,
                bonus: attribute_bonus(quickness),
            };
        }
        if let Some(intelligence) = mob_template.attributes.intelligence {
            attributes.intelligence = Attribute {
                base: intelligence,
                modifiers: 0,
                bonus: attribute_bonus(intelligence),
            };
            mob_intelligence = intelligence;
        }
        entity_builder = entity_builder.with(attributes);

        let mut skills = Skills {
            skills: HashMap::new(),
        };
        skills.skills.insert(Skill::Melee, 1);
        skills.skills.insert(Skill::Defense, 1);
        skills.skills.insert(Skill::Magic, 1);

        if let Some(mob_skill) = &mob_template.skills {
            for skill in mob_skill.iter() {
                match skill.0.as_str() {
                    "Melee" => {
                        skills.skills.insert(Skill::Melee, *skill.1);
                    }
                    "Defense" => {
                        skills.skills.insert(Skill::Defense, *skill.1);
                    }
                    "Magic" => {
                        skills.skills.insert(Skill::Magic, *skill.1);
                    }
                    _ => {
                        rltk::console::log(format!("Unknown skill referenced: [{}]", skill.0));
                    }
                }
            }
        }
        entity_builder = entity_builder.with(skills);

        let mob_level = if mob_template.level.is_some() {
            mob_template.level.unwrap()
        } else {
            1
        };
        let mob_hp = npc_hp(mob_fitness, mob_level);
        let mob_mana = mana_at_level(mob_intelligence, mob_level);

        let pools = Pools {
            level: mob_level,
            xp: 0,
            hit_points: Pool {
                max: mob_hp,
                current: mob_hp,
            },
            mana: Pool {
                max: mob_mana,
                current: mob_mana,
            },
        };
        entity_builder = entity_builder.with(pools);

        entity_builder = entity_builder.with(Viewshed {
            visible_tiles: Vec::new(),
            range: mob_template.vision_range,
            dirty: true,
        });

        if let Some(natural_attack) = &mob_template.natural {
            let mut nature = NaturalAttackDefense {
                armor_class: natural_attack.armor_class,
                attacks: Vec::new(),
            };
            if let Some(attacks) = &natural_attack.attacks {
                for nature_attack in attacks.iter() {
                    let (n_dice, dice_type, bonus) = parse_dice_string(&nature_attack.damage);
                    let attack = NaturalAttack {
                        name: nature_attack.name.clone(),
                        hit_bonus: nature_attack.hit_bonus,
                        damage_n_dice: n_dice,
                        damage_die_type: dice_type,
                        damage_bonus: bonus,
                    };
                    nature.attacks.push(attack);
                }
            }
            entity_builder = entity_builder.with(nature);
        }

        if let Some(loot) = &mob_template.loot_table {
            entity_builder = entity_builder.with(LootTable {
                table: loot.clone(),
            });
        }

        if let Some(light) = &mob_template.light {
            entity_builder = entity_builder.with(LightSource {
                range: light.range,
                color: rltk::RGB::from_hex(&light.color).expect("Bad color"),
            });
        }

        let new_mob = entity_builder.build();

        // Are they wielding anything
        if let Some(wielding) = &mob_template.equipped {
            for tag in wielding.iter() {
                spawn_named_entity(raws, ecs, tag, SpawnType::Equipped { by: new_mob });
            }
        }

        return Some(new_mob);
    }
    None
}

pub fn spawn_named_prop(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.prop_index.contains_key(key) {
        let prop_template = &raws.raws.props[raws.prop_index[key]];

        let mut entity_builder = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        // Spawn in the specified location
        entity_builder = spawn_position(pos, entity_builder, key, raws);

        // Renderable
        if let Some(renderable) = &prop_template.renderable {
            entity_builder = entity_builder.with(get_renderable_component(renderable));
        }

        entity_builder = entity_builder.with(Name {
            name: prop_template.name.clone(),
        });

        if let Some(hidden) = prop_template.hidden {
            if hidden {
                entity_builder = entity_builder.with(Hidden {});
            }
        }

        if let Some(blocks_tile) = prop_template.blocks_tile {
            if blocks_tile {
                entity_builder = entity_builder.with(BlocksTile {});
            }
        }

        if let Some(blocks_visibility) = prop_template.blocks_visibility {
            if blocks_visibility {
                entity_builder = entity_builder.with(BlocksVisibility {});
            }
        }

        if let Some(door_open) = prop_template.door_open {
            entity_builder = entity_builder.with(Door { open: door_open });
        }

        if let Some(entry_trigger) = &prop_template.entry_trigger {
            entity_builder = entity_builder.with(EntryTrigger {});
            for effect in entry_trigger.effects.iter() {
                match effect.0.as_str() {
                    "damage" => {
                        entity_builder = entity_builder.with(InflictsDamage {
                            damage: effect.1.parse::<i32>().unwrap(),
                        });
                    }
                    "single_activation" => {
                        entity_builder = entity_builder.with(SingleActivation {});
                    }
                    _ => {}
                }
            }
        }

        return Some(entity_builder.build());
    }
    None
}

pub fn spawn_named_entity(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, ecs, key, pos);
    }
    if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, ecs, key, pos);
    }
    if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, ecs, key, pos);
    }

    None
}

pub fn get_spawn_table_for_depth(raws: &RawMaster, depth: i32) -> RandomTable {
    use super::SpawnTableEntry;

    let available_options: Vec<&SpawnTableEntry> = raws
        .raws
        .spawn_table
        .iter()
        .filter(|entry| entry.min_depth <= depth && entry.max_depth >= depth)
        .collect();

    let mut random_table = RandomTable::new();
    for entry in available_options.iter() {
        let mut weight = entry.weight;
        if entry.add_map_depth_to_weight.is_some() {
            weight += depth;
        }
        random_table = random_table.add(entry.name.clone(), weight);
    }

    random_table
}
