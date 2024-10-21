
use bson::oid::ObjectId;
use futures_util::StreamExt;
use hyper::{Request, Body, Response, http::Error, body, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{character::{character_entity::CharacterEntity, character_presentation::CharacterPresentation}, long_term_storage_service::{db_character::StoredCharacter, db_player::StoredPlayer, db_world::StoredWorld}, map::tetrahedron_id::TetrahedronId};

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
pub struct CharacterCreationRequest 
{
    pub player_token: String,
    pub character_name:String,
    pub faction:u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CharacterCreationResponse 
{
    pub character_id:u16,
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
    pub character_id:u16,
    pub character_name:String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerDetailsResponse 
{
    pub player_token: String,
    pub joined_worlds: Vec<Iteration>,
    pub active_worlds: Vec<String>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JoinWithCharacterRequest 
{
    pub player_token: String,
    pub character_id:u16,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JoinWithCharacterResponse 
{
    pub session_id:u64,
    pub character_id:u16,
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
    println!("handling request {:?}", data);

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
    println!("got a {} as player token",player_token);

    let stored_character = StoredPlayer{
        id: None,
        player_name: data.player_name,
        player_token: player_token.clone(),
    };

    let _result = data_collection.insert_one(stored_character, None).await.unwrap();

    // let _object_id: Option<ObjectId> = match result.inserted_id {
    //     bson::Bson::ObjectId(id) => Some(id),
    //     _ => None,
    // };

    let new_character = PlayerCreationResponse{
        player_token,
    };

    let response = serde_json::to_vec(&new_character).unwrap();
    Ok(Response::new(Body::from(response)))
}

pub async fn handle_player_request(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: PlayerDetailsRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);

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

    let data_collection: mongodb::Collection<StoredCharacter> = context.db_client.database("game").collection::<StoredCharacter>("characters");
    let mut characters_cursor = data_collection
    .find(
        bson::doc! {
                "player_id": stored_player_id,
        },
        None,
    ).await
    .unwrap();


    let mut characters : Vec<Iteration>= Vec::new();
    while let Some(result) = characters_cursor.next().await {
        match result 
        {
            Ok(doc) => {
                characters.push(Iteration {
                    playing: true, 
                    world_name: doc.world_name.clone(),
                    character_id: doc.character_id,
                    character_name: doc.character_name.to_string(),
                    })
            },
            Err(error_details) => {
                println!("error getting characters from db with {:?}", error_details);
            },
        }
    }
    println!("----- characters {}", characters.len());

    // now get all the worlds to see available ones

    let worlds_collection: mongodb::Collection<StoredWorld> = context.db_client.database("game").collection::<StoredWorld>("worlds");

    let mut active_worlds = Vec::<String>::new();
    //  I should be requesting all available worlds...
    let data_from_db = worlds_collection
    .find_one(
        bson::doc! {
                "world_name": context.storage_game_map.world_name.to_owned()
        },
        None,
    ).await.unwrap();

    if let Some(w) = data_from_db
    {
        active_worlds.push(w.world_name);
    }

    let response_data = PlayerDetailsResponse {
        
        joined_worlds: characters,
        active_worlds,
        player_token: player_data.player_token,
    };

    let response_json = serde_json::to_vec(&response_data).unwrap();
    let response = Response::new(Body::from(response_json));
    return Ok(response);
}

pub async fn handle_create_character(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: CharacterCreationRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);

    let data_collection: mongodb::Collection<StoredPlayer> = context.db_client.database("game").collection::<StoredPlayer>("players");
    let data_from_db: Option<StoredPlayer> = data_collection
    .find_one(
        bson::doc! {
                "player_token": data.player_token,
        },
        None,
    ).await
    .unwrap();

    let player_id = data_from_db.map(|p| p.id).flatten();

    if player_id.is_none()
    {
        let mut response = Response::new(Body::from(String::from("player doesn't exist")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }

    let generator = &context.working_game_map.id_generator;
    let new_id = generator.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    println!("got a {} as id base ",new_id);


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


    let stored_character = StoredCharacter
    {
        id: None,
        player_id,
        version:1,
        world_id: context.working_game_map.world_id.clone(),
        world_name: context.working_game_map.world_name.clone(),
        character_id: new_id,
        character_name: data.character_name.clone(),
        position:initial_position.to_owned(),
        vertex_id: -1,
        faction: data.faction as u8,
        action: 0,
        flags: 0,
        inventory : Vec::new(),
        card_inventory : Vec::new(),
        level: 0,
        experience: 0,
        available_skill_points: 5,
        strength_points: 0,
        defense_points: 0,
        intelligence_points: 0,
        mana_points: 0,
        strength: 5,
        defense: 5,
        intelligence: 5,
        mana: 5,
        health: 1,
        buffs : Vec::new()
    };

    let data_collection: mongodb::Collection<StoredCharacter> = context.db_client.database("game").collection::<StoredCharacter>("characters");
    let result = data_collection.insert_one(stored_character, None).await.unwrap();

    let object_id: Option<ObjectId> = match result.inserted_id 
    {
        bson::Bson::ObjectId(id) => Some(id),
        _ => None,
    };

    let initial_position_tile_id = TetrahedronId::from_string(initial_position);
    let player_entity = CharacterEntity 
    {
        object_id,
        player_id,
        character_name : data.character_name.clone(),
        character_id: new_id,
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
        inventory_version : 1,
        flags:0,
        level: 0,
        experience: 0,
        available_skill_points: 5,
        strength_points: 0,
        defense_points: 0,
        intelligence_points: 0,
        mana_points: 0,
        base_strength: 5,
        base_defense: 5,
        base_intelligence: 5,
        base_mana: 5,
        health: 1,
        buffs : Vec::new(),
        buffs_summary: [0,0,0,0,0]
    };

    let mut players = context.working_game_map.character.lock().await;
    players.insert(new_id, player_entity.clone());
    drop(players);

    let mut players = context.storage_game_map.character.lock().await;
    players.insert(new_id, player_entity);

    drop(players);

    let name_with_padding = format!("{: <5}", data.character_name);
    let name_data : Vec<u32> = name_with_padding.chars().into_iter().map(|c| c as u32).collect();
    let mut name_array = [0u32; 5];
    name_array.clone_from_slice(&name_data.as_slice()[0..5]);

    let player_presentation = CharacterPresentation 
    {
        player_id: new_id,
        character_name: name_array,
    };
    
    println!("Adding player data {}", data.character_name);
    let mut presentation_data_cache =  context.cached_presentation_data.lock().await;
    presentation_data_cache.extend(player_presentation.to_bytes());

    let new_character = CharacterCreationResponse
    {
        character_id: new_id,
    };

    context.server_state.total_players.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let response = serde_json::to_vec(&new_character).unwrap();
    Ok(Response::new(Body::from(response)))
}

pub async fn handle_login_character(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> 
{

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: JoinWithCharacterRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);

    let data_collection: mongodb::Collection<StoredPlayer> = context.db_client.database("game").collection::<StoredPlayer>("players");
    let data_from_db: Option<StoredPlayer> = data_collection
    .find_one(
        bson::doc! {
                "player_token": data.player_token.clone(),
        },
        None,
    ).await
    .unwrap();

    let is_valid = if let Some(player) = data_from_db 
    {
        player.player_token == data.player_token
    }
    else {
        false 
    };

    if !is_valid 
    {
        let mut response = Response::new(Body::from(String::from("player token is not valid")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }

    let players = context.working_game_map.character.lock().await;

    if let Some(player) = players.get(&data.character_id) 
    {
        println!("player login {:?} vertex id {}", player, player.vertex_id);

        println!("position {:?} {}", player.position, player.health);
        let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
        let session_id = current_time.as_secs();

        let saved_char = JoinWithCharacterResponse
        {
            character_id: data.character_id,
            faction:player.faction,
            position:  player.second_position.to_string(),
            vertex_id: player.vertex_id,
            level : player.level,
            experience : player.experience,
            health: player.health,
            session_id,
            available_points: player.available_skill_points,
            strength: player.base_strength,
            defense: player.base_defense,
            intelligence: player.base_intelligence,
            mana: player.base_mana,
        };
        drop(players);

        println!("creating session id {} for {}", session_id, data.character_id);
        let session = &context.working_game_map.logged_in_players[data.character_id as usize];
        session.store(session_id, std::sync::atomic::Ordering::Relaxed);

        let response = serde_json::to_vec(&saved_char).unwrap();
        Ok(Response::new(Body::from(response)))
    }
    else {
        Ok(Response::new(Body::from("error: char not found")))
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
    println!("handling request {:?}", data);
    
    let mut players = context.working_game_map.character.lock().await;

    if let Some(player) = players.get_mut(&data.character_id) 
    {
        println!("player points exchange {:?}", player);
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