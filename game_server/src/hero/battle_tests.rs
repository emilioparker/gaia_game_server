use crate::{definitions::{Definition, damage_table, definitions_container::Definitions}, map::tetrahedron_id::TetrahedronId};
use super::hero_entity::HeroEntity;
use crate::mob::mob_entity::MobEntity;

fn make_definitions() -> crate::definitions::definitions_container::Definitions
{
    use crate::definitions::card::Card;
    use crate::definitions::stat_bonuses::StatBonus;
    use crate::definitions::realms::Realm;
    use crate::definitions::skills::Skill;
    use crate::definitions::buffs_data::BuffData;
    use crate::definitions::character_progression::CharacterProgression;
    use crate::definitions::mob_progression::MobProgression;
    use crate::definitions::mobs_data::MobData;
    use crate::definitions::professions::Profession;
    use crate::definitions::items::Item;
    use crate::definitions::equipment::Equipment;
    use crate::definitions::main_paths::MapPath;
    use crate::definitions::tower_difficulty::TowerDifficulty;
    use crate::definitions::stat_gains::StatGain;
    use crate::definitions::definitions_container::Definitions;
    use crate::definitions::Definition;
    use std::collections::HashMap;

    fn load_csv<T: serde::de::DeserializeOwned + Definition>(path: &str) -> Vec<T>
    {
        println!("decoding {path}");
        let mut rdr = csv::Reader::from_path(path).unwrap();
        let mut data = Vec::new();
        for result in rdr.deserialize()
        {
            let mut record: T = result.unwrap();
            record.fill_details();
            data.push(record);
        }
        data
    }

    let cards                = load_csv::<Card>               ("definitions/cards.csv");
    let stat_bonus_table     = load_csv::<StatBonus>           ("definitions/stat_bonuses.csv");
    let realms               = load_csv::<Realm>              ("definitions/realms.csv");
    let character_progression= load_csv::<CharacterProgression>("definitions/character_progression.csv");
    let mob_progression      = load_csv::<MobProgression>     ("definitions/mob_progression.csv");
    let mobs                 = load_csv::<MobData>            ("definitions/mobs.csv");
    let buffs_by_code        = load_csv::<BuffData>           ("definitions/buffs.csv");
    let equipment            = load_csv::<Equipment>          ("definitions/equipment.csv");
    let skills_by_id         = load_csv::<Skill>              ("definitions/skills.csv");
    let professions_vec      = load_csv::<Profession>         ("definitions/professions.csv");
    let items                = load_csv::<Item>               ("definitions/items.csv");
    let stat_gains           = load_csv::<StatGain>           ("definitions/stat_gains.csv");
    let main_paths           = load_csv::<MapPath>            ("definitions/main_paths.csv");
    let towers_difficulty    = load_csv::<TowerDifficulty>    ("definitions/towers_difficulty.csv");
    let armor_types          = load_csv::<crate::definitions::armor_types::ArmorType>("definitions/armor_types.csv");
    let mut damage_table = std::collections::HashMap::new();
    for damage_type in crate::definitions::damage_table::DAMAGE_TYPES
    {
        let entries = load_csv::<crate::definitions::damage_table::DamageTableEntry>(&format!("definitions/damage_table_{damage_type}.csv"));
        damage_table.insert(damage_type.to_string(), entries);
    }

    let mut buffs = HashMap::new();
    for e in &buffs_by_code { buffs.insert(e.id.clone(), e.clone()); }

    let mut skills = HashMap::new();
    for e in &skills_by_id { skills.insert(e.name.clone(), e.clone()); }

    let mut professions = HashMap::new();
    for e in &professions_vec { professions.insert(e.profession.clone(), e.clone()); }

    let mut professions_by_id = vec![Profession::default(); professions_vec.len()];
    for e in professions_vec { professions_by_id[e.id.clone() as usize] = e.clone(); }

    let mut mob_progression_by_mob = vec![Vec::new(); mobs.len()];
    for e in &mob_progression { mob_progression_by_mob[e.mob as usize].push(e.clone()); }

    Definitions {
        regions_by_code: std::array::from_fn(|_| TetrahedronId::default()),
        regions_by_id: HashMap::new(),
        character_progression,
        mob_progression,
        mob_progression_by_mob,
        props: Vec::new(),
        main_paths,
        towers_difficulty,
        items,
        cards,
        mobs,
        buffs,
        buffs_by_code,
        equipment,
        skills,
        skills_by_id,
        professions,
        professions_by_id,
        stat_bonus_table,
        stat_gains,
        realms,
        armor_types,
        damage_table,
    }
}

fn make_hero(strength: u16, vitality: u16, agility: u16, will: u16, definitions: &Definitions) -> HeroEntity
{
    let mut hero = HeroEntity
    {
        object_id: None,
        player_id: None,
        version: 1,
        hero_name: "test".to_owned(),
        hero_id: 0,
        faction: 0,
        profession: 0,
        realm: 0,
        action: 0,
        flags: 0,
        position: TetrahedronId::default(),
        second_position: TetrahedronId::default(),
        vertex_id: -1,
        path: [0; 6],
        time: 0,
        inventory: Vec::new(),
        card_inventory: Vec::new(),
        equipment_inventory: Vec::new(),
        skills: Vec::new(),
        inventory_version: 0,
        health: 0,
        mana: 0,
        stamina: 0,
        intelligence: 0,
        level: 1,
        experience: 0,
        available_skill_points: 0,
        strength_stat: strength,
        vitality_stat: vitality,
        agility_stat: agility,
        will_stat: will,
        regeneration_time: 0,
        buffs: Vec::new(),
        buffs_summary: [0; 5],
        card_id_generator: 0,
        equipment_id_generator: 0,
    };

    hero.max_stats(definitions);

    hero
}

fn make_mob(mob_definition_id: u16, level: u8, definitions: &Definitions) -> MobEntity
{
    use crate::ability_user::AbilityUser;

    let mut mob = MobEntity
    {
        mob_id: 1,
        mob_definition_id,
        level,
        version: 1,
        owner_id: 0,
        ownership_time: 0,
        start_position_id: TetrahedronId::default(),
        end_position_id: TetrahedronId::default(),
        path: [0; 6],
        time: 0,
        strength_stat: 0,
        vitality_stat: 0,
        agility_stat: 0,
        will_stat: 0,
        health: 0,
        buffs: Vec::new(),
        buffs_summary: [0; 5],
    };

    mob.init_stats(definitions);
    mob
}

#[test]
fn test_warrior_vs_mob()
{
    use crate::ability_user::AbilityUser;

    let definitions = make_definitions();

    let warrior = definitions.professions.get("Warrior").unwrap();
    let (ws, wv, wa, ww) = (warrior.strength.init, warrior.vitality.init, warrior.agility.init, warrior.will.init);

    // mob 1 = enemy_1 (goblin), level 0
    let mut attacker = make_hero(ws, wv, wa, ww, &definitions);
    let mut mob = make_mob(1, 0, &definitions);

    println!("before: warrior(str:{ws} agi:{wa}) vs mob(hp:{})", mob.health);

    let attack = attacker.get_total_attack(1, &definitions);
    let defense = mob.get_total_defense(&definitions);

    let roll = rand::Rng::gen_range(&mut rand::thread_rng(), 1i16..=100);
    let result = (roll + attack - defense).max(0);

    let card_data = definitions.cards.get(1).unwrap();
    let damage = damage_table::get_damage(&definitions.damage_table, result, &card_data.damage_type, "at1").unwrap_or((0, 'A'));
    println!("---- result {result} => {damage:?}");
    mob.health = mob.health.saturating_sub(damage.0);
    attacker.mana = attacker.mana.saturating_sub(card_data.mana_cost);
    attacker.stamina = attacker.stamina.saturating_sub(card_data.stamina_cost);

    println!("after: warrior(str:{ws} agi:{wa}) vs mob — attack:{attack} defense:{defense} roll:{roll} mob_hp:{}", mob.health);
}

#[test]
fn test_warrior_vs_tank()
{
    use crate::ability_user::AbilityUser;

    let definitions = make_definitions();

    let warrior = definitions.professions.get("Warrior").unwrap();
    let (ws, wv, wa, ww) = (warrior.strength.init, warrior.vitality.init, warrior.agility.init, warrior.will.init);

    let tank = definitions.professions.get("Tank").unwrap();
    let (ts, tv, ta, tw) = (tank.strength.init, tank.vitality.init, tank.agility.init, tank.will.init);

    // health = vitality at realm 0 (multiplier 1)
    let mut attacker = make_hero(ws, wv, wa, ww, &definitions);
    let mut defender = make_hero(ts, tv, ta, tw, &definitions);

    println!("before: warrior(str:{ws} agi:{wa}) vs tank(str:{ts} agi:{ta} defender_hp:{}", defender.health);

    // card 1 is punch_1 (strength-based attack)
    let attack = attacker.get_total_attack(1, &definitions);
    println!("oB {attack}");
    let defense = defender.get_total_defense(&definitions);
    println!("dB {defense}");

    let roll = rand::Rng::gen_range(&mut rand::thread_rng(), 1i16..=100);
    let result = (roll + attack - defense).max(0);

    let card_data = definitions.cards.get(1).unwrap();
    let armor = defender.get_armor()
        .map_or("at1", |i| definitions.equipment[i.equipment_definition_id as usize].armor_type.as_str());

    let damage = damage_table::get_damage(&definitions.damage_table, result, &card_data.damage_type, armor).unwrap_or((0, 'A'));
    println!("---- result {result} => {damage:?}");
    defender.health = defender.health.saturating_sub(damage.0);
    attacker.mana = attacker.mana.saturating_sub(card_data.mana_cost);
    attacker.stamina = attacker.mana.saturating_sub(card_data.stamina_cost);

    println!("after: warrior(str:{ws} agi:{wa}) vs tank(str:{ts} agi:{ta}) — attack:{attack} defense:{defense} roll:{roll} defender_hp:{}", defender.health);
}
