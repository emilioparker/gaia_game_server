
use bson::oid::ObjectId;
use futures_util::StreamExt;
use hyper::{body, http::Error, Body, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{hero::{hero_card_inventory::CardItem, hero_entity::HeroEntity, hero_inventory::InventoryItem, hero_presentation::HeroPresentation, hero_tower_progress::HeroTowerProgress, hero_weapon_inventory::WeaponItem}, long_term_storage_service::{db_hero::StoredHero, db_player::StoredPlayer, db_world::StoredWorld}, map::tetrahedron_id::TetrahedronId, web_service::create_response_builder};

use super::AppContext;


#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerCreationRequest 
{
    pub player_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerCreationResponse 
{
    pub player_token: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HeroCreationRequest 
{
    pub player_token: String,
    // pub character_name:String,
    pub faction:u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HeroCreationResponse 
{
    pub hero_id:u16,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerDetailsRequest 
{
    pub player_name:String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Iteration 
{
    pub playing:bool,
    pub world_name:String,
    pub hero_id:u16,
    pub hero_name:String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerDetailsResponse 
{
    pub player_token: String,
    pub joined_worlds: Vec<Iteration>,
    pub active_worlds: Vec<String>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JoinWithHeroRequest 
{
    pub player_token: String,
    pub hero_id:u16,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JoinWithHeroResponse 
{
    pub session_id:u64,
    pub hero_id:u16,
    pub faction:u8,
    pub position:String,
    pub vertex_id:i32,
    pub level:u8,
    pub experience:u32,
    pub available_points:u8,
    pub health:u16,
    pub strength:u16,
    pub defense:u16,
    pub intelligence:u16,
    pub mana:u16,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ExchangeSkillPointsRequest 
{
    pub character_id : u16,
    pub strength:u8,
    pub defense:u8,
    pub intelligence:u8,
    pub mana:u8,
}


pub async fn handle_create_player(context: AppContext, mut req: Request<Body>) ->Result<Body, String> 
{
    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: PlayerCreationRequest = serde_json::from_slice(&data).unwrap();
    cli_log::info!("handling request {:?}", data);

    let data_collection: mongodb::Collection<StoredPlayer> = context.db_client.database("game").collection::<StoredPlayer>("players");
    let data_from_db: Option<StoredPlayer> = data_collection
    .find_one(
        bson::doc! 
        {
                "player_name": data.player_name.clone(),
        },
        None,
    ).await
    .unwrap();


    if let Some(_player) = data_from_db 
    {
        return Err("player_already_created".to_owned());
    }

    let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
    let player_token = format!("token_{}", current_time.as_secs());
    cli_log::info!("got a {} as player token",player_token);

    let stored_character = StoredPlayer
    {
        id: None,
        player_name: data.player_name,
        player_token: player_token.clone(),
    };

    let _result = data_collection.insert_one(stored_character, None).await.unwrap();
    let new_character = PlayerCreationResponse
    {
        player_token,
    };

    let response = serde_json::to_vec(&new_character).unwrap();
    Ok(Body::from(response))
}

pub async fn handle_player_request(context: AppContext, mut req: Request<Body>) ->Result<Body, String> 
{
    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: PlayerDetailsRequest = serde_json::from_slice(&data).unwrap();
    cli_log::info!("handling request {:?}", data);

    let data_collection: mongodb::Collection<StoredPlayer> = context.db_client.database("game").collection::<StoredPlayer>("players");
    let stored_player: Option<StoredPlayer> = data_collection
    .find_one(
        bson::doc! 
        {
                "player_name": data.player_name,
        },
        None,
    ).await
    .unwrap();

    if stored_player.is_none()
    {
        return Err("player_not_found".to_owned());
    }

    let player_data = stored_player.unwrap();
    let stored_player_id = player_data.id;

    let data_collection: mongodb::Collection<StoredHero> = context.db_client.database("game").collection::<StoredHero>("characters");
    let mut characters_cursor = data_collection
    .find(
        bson::doc! 
        {
                "player_id": stored_player_id,
        },
        None,
    ).await
    .unwrap();


    let mut characters : Vec<Iteration>= Vec::new();
    while let Some(result) = characters_cursor.next().await 
    {
        match result 
        {
            Ok(doc) => 
            {
                characters.push(Iteration 
                    {
                    playing: true, 
                    world_name: doc.world_name.clone(),
                    hero_id: doc.character_id,
                    hero_name: doc.character_name.to_string(),
                    })
            },
            Err(error_details) => 
            {
                cli_log::info!("error getting characters from db with {:?}", error_details);
            },
        }
    }
    cli_log::info!("----- characters {}", characters.len());

    // now get all the worlds to see available ones

    let worlds_collection: mongodb::Collection<StoredWorld> = context.db_client.database("game").collection::<StoredWorld>("worlds");

    let mut active_worlds = Vec::<String>::new();
    //  I should be requesting all available worlds...
    let data_from_db = worlds_collection
    .find_one(
        bson::doc! 
        {
                "world_name": context.storage_game_map.world_name.to_owned()
        },
        None,
    ).await.unwrap();

    if let Some(w) = data_from_db
    {
        active_worlds.push(w.world_name);
    }

    let response_data = PlayerDetailsResponse 
    {
        
        joined_worlds: characters,
        active_worlds,
        player_token: player_data.player_token,
    };

    let response_json = serde_json::to_vec(&response_data).unwrap();
    Ok(Body::from(response_json))
}

pub async fn handle_create_hero(context: AppContext, mut req: Request<Body>) ->Result<Body, String> 
{
    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: HeroCreationRequest = serde_json::from_slice(&data).unwrap();
    cli_log::info!("handling request {:?}", data);

    let data_collection: mongodb::Collection<StoredPlayer> = context.db_client.database("game").collection::<StoredPlayer>("players");
    let data_from_db: Option<StoredPlayer> = data_collection
    .find_one(
        bson::doc! {
                "player_token": data.player_token,
        },
        None,
    ).await
    .unwrap();


    if data_from_db.is_none()
    {
        return Err("player_not_found".to_owned());
    }

    let stored_player = data_from_db.unwrap();

    let generator = &context.working_game_map.id_generator;
    let new_id = generator.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    cli_log::info!("got a {} as id base ",new_id);


    let initial_position = match data.faction
    {
        1 => "t312222222",
        2 => "m113222222",
        3 => "k121222222",
        _ => "k121222222",
    };

    // public TetrahedronId GetInitialTile()
    // {
    //     if (InitialCharacterState != null)
    //     {
    //         switch (InitialCharacterState.FactionCode)
    //         {
    //             case 1 :
    //                 return TetrahedronId.GetTetrahedronId("t312222222");
    //             case 2 :
    //                 return TetrahedronId.GetTetrahedronId("m113222222");
    //             case 3 :
    //                 return TetrahedronId.GetTetrahedronId("k121222222");
    //         }
    //     }
    //     return TetrahedronId.GetTetrahedronId("t312222222");
    // }


    let stored_character = StoredHero
    {
        id: None,
        player_id: stored_player.id,
        version:1,
        world_id: context.working_game_map.world_id.clone(),
        world_name: context.working_game_map.world_name.clone(),
        character_id: new_id,
        character_name: stored_player.player_name.clone(),
        position:initial_position.to_owned(),
        vertex_id: -1,
        faction: data.faction as u8,
        action: 0,
        flags: 0,
        inventory : Vec::new(),
        card_inventory : Vec::new(),
        weapon_inventory : Vec::new(),
        level: 0,
        experience: 0,
        available_skill_points: 5,
        weapon:0,
        strength_points: 0,
        defense_points: 0,
        intelligence_points: 0,
        mana_points: 0,
        strength: 10,
        defense: 10,
        intelligence: 10,
        mana: 10,
        health: 10,
        buffs : Vec::new(),
        tower_progress: HeroTowerProgress::default().into(),
    };

    let data_collection: mongodb::Collection<StoredHero> = context.db_client.database("game").collection::<StoredHero>("characters");
    let result = data_collection.insert_one(stored_character, None).await.unwrap();

    let object_id: Option<ObjectId> = match result.inserted_id 
    {
        bson::Bson::ObjectId(id) => Some(id),
        _ => None,
    };

    let initial_position_tile_id = TetrahedronId::from_string(initial_position);
    let player_entity = HeroEntity 
    {
        object_id,
        player_id: stored_player.id,
        hero_name : stored_player.player_name.clone(),
        hero_id: new_id,
        version:1,
        faction: data.faction as u8,
        action: 0,
        position: initial_position_tile_id.clone(),
        second_position: initial_position_tile_id.clone(),
        vertex_id: -1,
        path: [0,0,0,0,0,0],
        time:0,
        inventory: Vec::new(), // fill this from storedcharacter
        card_inventory : Vec::new(),
        weapon_inventory : Vec::new(),
        inventory_version : 1,
        flags:0,
        level: 0,
        experience: 0,
        available_skill_points: 5,
        weapon:0,
        strength_points: 0,
        defense_points: 0,
        intelligence_points: 0,
        mana_points: 0,
        base_strength: 10,
        base_defense: 10,
        base_intelligence: 10,
        base_mana: 10,
        health: 10,
        buffs : Vec::new(),
        buffs_summary: [0,0,0,0,0],
        tower_progress: HeroTowerProgress::default(),
    };

    let mut players = context.working_game_map.character.lock().await;
    players.insert(new_id, player_entity.clone());
    drop(players);

    let mut players = context.storage_game_map.character.lock().await;
    players.insert(new_id, player_entity);

    drop(players);

    let name_with_padding = format!("{: <5}", stored_player.player_name);
    let name_data : Vec<u32> = name_with_padding.chars().into_iter().map(|c| c as u32).collect();
    let mut name_array = [0u32; 5];
    name_array.clone_from_slice(&name_data.as_slice()[0..5]);

    let player_presentation = HeroPresentation 
    {
        player_id: new_id,
        character_name: name_array,
    };
    
    cli_log::info!("Adding player data {}", stored_player.player_name);
    let mut presentation_data_cache =  context.cached_presentation_data.lock().await;
    presentation_data_cache.extend(player_presentation.to_bytes());

    let new_character = HeroCreationResponse
    {
        hero_id: new_id,
    };

    context.server_state.total_players.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let data= serde_json::to_vec(&new_character).unwrap();
    Ok(Body::from(data))
}

pub async fn handle_login_with_hero(context: AppContext, mut req: Request<Body>) ->Result<Body, String> 
{
    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: JoinWithHeroRequest = serde_json::from_slice(&data).unwrap();
    cli_log::info!("handling request {:?}", data);

    let data_collection: mongodb::Collection<StoredPlayer> = context.db_client.database("game").collection::<StoredPlayer>("players");
    let data_from_db: Option<StoredPlayer> = data_collection
    .find_one(
        bson::doc! 
        {
                "player_token": data.player_token.clone(),
        },
        None,
    ).await
    .unwrap();

    let is_valid = if let Some(player) = data_from_db 
    {
        player.player_token == data.player_token
    }
    else 
    {
        false 
    };

    if !is_valid 
    {
        return Err("player_token_not_valid".to_owned())
    }

    let players = context.working_game_map.character.lock().await;

    if let Some(player) = players.get(&data.hero_id) 
    {
        cli_log::info!("player login {:?} vertex id {}", player, player.vertex_id);

        cli_log::info!("position {:?} {}", player.position, player.health);
        let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
        let session_id = current_time.as_secs();

        let mut output = Vec::<u8>::with_capacity(100);

        let session_bytes = u64::to_le_bytes(session_id); // 8 bytes
        output.extend_from_slice(&session_bytes);
        let encoded_player_data = player.to_bytes();
        output.extend_from_slice(&encoded_player_data);
        pack_inventory(&mut output, &player.inventory, &player.card_inventory, &player.weapon_inventory, player.inventory_version);

        drop(players);

        cli_log::info!("creating session id {} for {}", session_id, data.hero_id);
        let session = &context.working_game_map.logged_in_players[data.hero_id as usize];
        session.store(session_id, std::sync::atomic::Ordering::Relaxed);

        Ok(Body::from(output))
    }
    else 
    {
        return Err("hero_not_found".to_owned())
    }
}


pub fn pack_inventory(
    output : &mut Vec<u8>,
    inventory: &Vec<InventoryItem>, 
    card_inventory: &Vec<CardItem>,
    weapon_inventory : &Vec<WeaponItem>,
    inventory_version: u8)
{
    // only need when using the inventory request protocol, here we only use it on login.
    // we write the protocol
    // let inventory_request = crate::protocols::Protocol::InventoryRequest as u8;
    // let buffer = [inventory_request;1];

    // output.extend_from_slice(&buffer);
    // we write the amount of items.
    let item_len_bytes = u32::to_le_bytes(inventory.len() as u32);
    output.extend_from_slice(&item_len_bytes);

    // cli_log::info!("--- inventory length {}", inventory.len());

    for item in inventory 
    {
        // cli_log::info!("---- item {:?}", item);
        let buffer = item.to_bytes();
        output.extend_from_slice(&buffer);
    }

    // card inventory
    let card_inventory_len_bytes = u32::to_le_bytes(card_inventory.len() as u32);
    output.extend_from_slice(&card_inventory_len_bytes);

    // cli_log::info!("--- inventory length {}", card_inventory.len());
    for item in card_inventory 
    {
        // cli_log::info!("---- card {:?}", item);
        let buffer = item.to_bytes();
        output.extend_from_slice(&buffer);
    }

    // weapon inventory
    let weapon_inventory_len_bytes = u32::to_le_bytes(weapon_inventory.len() as u32);
    output.extend_from_slice(&weapon_inventory_len_bytes);

    // cli_log::info!("--- weapon inventory length {}", weapon_inventory.len());
    for item in weapon_inventory 
    {
        // cli_log::info!("---- card {:?}", item);
        let buffer = item.to_bytes();
        output.extend_from_slice(&buffer);
    }

    output.extend_from_slice(&[inventory_version]);
}

pub async fn handle_characters_request(context: AppContext) -> Result<Body, String> 
{
    let presentation_cache = context.cached_presentation_data.lock().await;
    let presentation_cache : Vec<u8> = presentation_cache.to_vec();
    Ok(Body::from(presentation_cache))
}

pub async fn exchange_skill_points(context: AppContext, mut req: Request<Body>) -> Result<Body, String> 
{
    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: ExchangeSkillPointsRequest = serde_json::from_slice(&data).unwrap();
    cli_log::info!("handling request {:?}", data);
    
    let mut players = context.working_game_map.character.lock().await;

    if let Some(player) = players.get_mut(&data.character_id) 
    {
        cli_log::info!("player points exchange {:?}", player);
        let total_points = data.strength + data.defense + data.mana + data.intelligence;
        if total_points > player.available_skill_points
        {
            return Err("not_enough_skill_points".to_owned());
        }
        else 
        {
            player.available_skill_points -= total_points as u8;
            player.strength_points += data.strength;
            player.defense_points += data.defense;
            player.intelligence_points += data.intelligence;
            player.mana_points += data.mana;
            player.version += 1;
        }
        drop(players);

        return Ok(Body::from("done".to_owned()));
    }
    else 
    {
        return Err("hero_not_found".to_owned());
    }
}