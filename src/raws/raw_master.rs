use super::Raws;
use crate::components::*;
use crate::random_table::RandomTable;
use specs::prelude::*;
use std::collections::{HashMap, HashSet};

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
}

pub struct RawMaster {
    raws: Raws,
    item_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws: Raws {
                items: Vec::new(),
                mobs: Vec::new(),
                props: Vec::new(),
                spawn_table: Vec::new(),
            },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
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
    }
}

fn spawn_position(pos: SpawnType, new_entity: EntityBuilder) -> EntityBuilder {
    let mut entity_builder = new_entity;
    match pos {
        SpawnType::AtPosition { x, y } => entity_builder = entity_builder.with(Position { x, y }),
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

pub fn spawn_named_item(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];

        let mut entity_builder = new_entity;

        // Spawn in the specified location
        entity_builder = spawn_position(pos, entity_builder);

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
            entity_builder = entity_builder.with(MeleePowerBonus {
                power: weapon.power_bonus,
            });
        }

        if let Some(shield) = &item_template.shield {
            entity_builder = entity_builder.with(Equippable {
                slot: EquipmentSlot::Shield,
            });
            entity_builder = entity_builder.with(DefenseBonus {
                defense: shield.defense_bonus,
            });
        }

        return Some(entity_builder.build());
    }
    None
}

pub fn spawn_named_mob(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];

        let mut entity_builder = new_entity;

        // Spawn in the specified location
        entity_builder = spawn_position(pos, entity_builder);

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
        entity_builder = entity_builder.with(CombatStats {
            max_hp: mob_template.stats.max_hp,
            hp: mob_template.stats.hp,
            defense: mob_template.stats.defense,
            power: mob_template.stats.power,
        });
        entity_builder = entity_builder.with(Viewshed {
            visible_tiles: Vec::new(),
            range: mob_template.vision_range,
            dirty: true,
        });

        return Some(entity_builder.build());
    }
    None
}

pub fn spawn_named_prop(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.prop_index.contains_key(key) {
        let prop_template = &raws.raws.props[raws.prop_index[key]];

        let mut entity_builder = new_entity;

        // Spawn in the specified location
        entity_builder = spawn_position(pos, entity_builder);

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
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, new_entity, key, pos);
    }
    if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, new_entity, key, pos);
    }
    if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, new_entity, key, pos);
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
