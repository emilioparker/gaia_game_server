use crate::{definitions::damage_table, map::tetrahedron_id::TetrahedronId};
use super::hero_entity::HeroEntity;

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
    let damage_table         = load_csv::<crate::definitions::damage_table::DamageTableEntry>("definitions/damage_table.csv");

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

fn make_hero(strength: u16, vitality: u16, agility: u16, will: u16, health: u16) -> HeroEntity
{
    HeroEntity
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
        health,
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
        buffs: Vec::new(),
        buffs_summary: [0; 5],
        card_id_generator: 0,
        equipment_id_generator: 0,
    }
}

#[test]
fn test_simple_battle()
{
    use crate::ability_user::AbilityUser;

    let definitions = make_definitions();

    let warrior = definitions.professions.get("Warrior").unwrap();
    let (ws, wv, wa, ww) = (warrior.strength.init, warrior.vitality.init, warrior.agility.init, warrior.will.init);

    let tank = definitions.professions.get("Tank").unwrap();
    let (ts, tv, ta, tw) = (tank.strength.init, tank.vitality.init, tank.agility.init, tank.will.init);

    // health = vitality at realm 0 (multiplier 1)
    let attacker = make_hero(ws, wv, wa, ww, wv);
    let mut defender = make_hero(ts, tv, ta, tw, tv);

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

    let damage = damage_table::get_damage(&definitions.damage_table, result, &card_data.damage_type, armor);
    println!("---- result {result} => {damage:?}");
    // defender.health = defender.health.saturating_sub(damage);

    println!("warrior(str:{ws} agi:{wa}) vs tank(str:{ts} agi:{ta}) — attack:{attack} defense:{defense} roll:{roll} defender_hp:{}", defender.health);
}
