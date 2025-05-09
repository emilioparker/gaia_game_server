
use bson::oid::ObjectId;
use futures_util::StreamExt;
use hyper::{Request, Body, Response, http::Error, body, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{hero::{hero_entity::HeroEntity, hero_presentation::HeroPresentation}, long_term_storage_service::{db_hero::StoredHero, db_player::StoredPlayer, db_world::StoredWorld}, map::tetrahedron_id::TetrahedronId};

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

pub async fn handle_create_player(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> 
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
        let mut response = Response::new(Body::from(String::from("Player already created")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
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

    // let _object_id: Option<ObjectId> = match result.inserted_id {
    //     bson::Bson::ObjectId(id) => Some(id),
    //     _ => None,
    // };

    let new_character = PlayerCreationResponse
    {
        player_token,
    };

    let response = serde_json::to_vec(&new_character).unwrap();
    Ok(Response::new(Body::from(response)))
}

pub async fn handle_player_request(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: PlayerDetailsRequest = serde_json::from_slice(&data).unwrap();
    cli_log::info!("handling request {:?}", data);

    let data_collection: mongodb::Collection<StoredPlayer> = context.db_client.database("game").collection::<StoredPlayer>("players");
    let stored_player: Option<StoredPlayer> = data_collection
    .find_one(
        bson::doc! {
                "player_name": data.player_name,
        },
        None,
    ).await
    .unwrap();

    if stored_player.is_none()
    {
        let mut response = Response::new(Body::from(String::from("Player not found")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
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
    while let Some(result) = characters_cursor.next().await {
        match result 
        {
            Ok(doc) => 
            {
                characters.push(Iteration {
                    playing: true, 
                    world_name: doc.world_name.clone(),
                    hero_id: doc.character_id,
                    hero_name: doc.character_name.to_string(),
                    })
            },
            Err(error_details) => {
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
    let response = Response::new(Body::from(response_json));
    return Ok(response);
}

pub async fn handle_create_hero(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> 
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
        let mut response = Response::new(Body::from(String::from("player doesn't exist")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
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
        buffs : Vec::new()
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
        buffs_summary: [0,0,0,0,0]
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

    let response = serde_json::to_vec(&new_character).unwrap();
    Ok(Response::new(Body::from(response)))
}

pub async fn handle_login_with_hero(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> 
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
        let mut response = Response::new(Body::from(String::from("player token is not valid")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }

    let players = context.working_game_map.character.lock().await;

    if let Some(player) = players.get(&data.hero_id) 
    {
        cli_log::info!("player login {:?} vertex id {}", player, player.vertex_id);

        cli_log::info!("position {:?} {}", player.position, player.health);
        let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
        let session_id = current_time.as_secs();

        let encoded_player_data = player.to_bytes();
        drop(players);

        cli_log::info!("creating session id {} for {}", session_id, data.hero_id);
        let session = &context.working_game_map.logged_in_players[data.hero_id as usize];
        session.store(session_id, std::sync::atomic::Ordering::Relaxed);

        let session_bytes = u64::to_le_bytes(session_id); // 8 bytes
        let mut output = Vec::<u8>::new();
        output.extend_from_slice(&session_bytes);
        output.extend_from_slice(&encoded_player_data);

        let response = Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "application/octet-stream")
            .body(Body::from(output))
            .expect("Failed to create response");
        Ok(response)
    }
    else 
    {
        let response = Response::builder()
            .status(hyper::StatusCode::NOT_FOUND)
            .header("Content-Type", "application/octet-stream")
            .body(Body::from("Error"))
            .expect("Failed to create response");
        Ok(response)
    }
}

pub async fn handle_characters_request(context: AppContext) -> Result<Response<Body>, Error> 
{
    let presentation_cache = context.cached_presentation_data.lock().await;
    let presentation_cache : Vec<u8> = presentation_cache.to_vec();

    let response = Response::builder()
        .status(hyper::StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(presentation_cache))
        .expect("Failed to create response");
    Ok(response)
}

pub async fn exchange_skill_points(context: AppContext, mut req: Request<Body>) -> Result<Response<Body>, Error> 
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
            let mut response = Response::new(Body::from(String::from("missing route")));
            *response.status_mut() = StatusCode::NOT_ACCEPTABLE;
            return Ok(response);
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

        Ok(Response::new(Body::from("Done")))
    }
    else 
    {
        let mut response = Response::new(Body::from(String::from("char not found")));
        *response.status_mut() = StatusCode::NOT_ACCEPTABLE;
        return Ok(response);
    }
}