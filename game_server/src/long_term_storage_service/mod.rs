pub mod db_region;
pub mod db_world;
pub mod db_character;
pub mod db_player;
pub mod world_service;
pub mod players_service;



#[cfg(test)]
mod tests {
    use std::env;
    use bson::{oid::ObjectId, document};
    use mongodb::{Client, options::{ClientOptions, ResolverConfig}};
    use chrono::{TimeZone, Utc};
    use mongodb::bson::doc;
    use serde::{Serialize, Deserialize};

    #[test]
    fn test_doc() {

        let new_doc = doc! {
        "title": "Parasite",
        "year": 2020,
        "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
        "released": Utc.ymd(2020, 2, 7).and_hms_opt(0, 0, 0),
        };

        println!("{}", new_doc);
    }

    // fn insert_test()
    // {
    //     let new_doc = doc! {
    //     "title": "Parasite",
    //     "year": 2020,
    //     "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
    //     "released": Utc.ymd(2020, 2, 7).and_hms_opt(0, 0, 0),
    //     };

    //     println!("{}", new_doc);
    //     let insert_result = movies.insert_one(new_doc.clone(), None).await?;
    //     println!("New document ID: {}", insert_result.inserted_id);
    // }

    #[tokio::test]
    async fn test_something_async() {
        let new_doc = doc! {
            "title": "Parasite",
            "year": 2020,
            "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
            "released": Utc.with_ymd_and_hms(2020, 2, 7, 0,0,0).unwrap()
        };
        
        // let client_uri = env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
        let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";


        // A Client is needed to connect to MongoDB:
        // An extra line of code to work around a DNS issue on Windows:
        let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
        let client = Client::with_options(options).unwrap();
        // Print the databases in our MongoDB cluster:
        println!("Databases:");
        for name in client.list_database_names(None, None).await.unwrap() {
            println!("- {}", name);
        }
    }

    #[tokio::test]
    async fn test_insert() {
        let new_doc = doc! {
            "title": "Parasite",
            "year": 2020,
            "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
            "released": Utc.with_ymd_and_hms(2020, 2, 7, 0,0,0).unwrap()
        };
        
        // let client_uri = env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
        let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";


        // A Client is needed to connect to MongoDB:
        // An extra line of code to work around a DNS issue on Windows:
        let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
        let client = Client::with_options(options).unwrap();
        // Print the databases in our MongoDB cluster:
        println!("Databases:");
        for name in client.list_database_names(None, None).await.unwrap() {
            println!("- {}", name);
        }
        let movies = client.database("sample_mflix").collection("movies");

        let new_doc = doc! {
        "title": "Parasite",
        "year": 2021,
        "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
        };

        println!("{}", new_doc);
        let insert_result = movies.insert_one(new_doc.clone(), None).await.unwrap();
        println!("New document ID: {}", insert_result.inserted_id);
    }

    // You use `serde` to create structs which can serialize & deserialize between BSON:
    #[derive(Serialize, Deserialize, Debug)]
    struct Data {
        #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
        id: Option<ObjectId>,
        title: String,
        year: i32,
        plot: String,
        compressed_data : Vec<u8>,
    }


    #[tokio::test]
    async fn test_insert_struct() {
        let new_doc = doc! {
            "title": "Parasite",
            "year": 2020,
            "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
            "released": Utc.with_ymd_and_hms(2020, 2, 7, 0,0,0).unwrap()
        };
        
        // let client_uri = env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
        let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";

        // A Client is needed to connect to MongoDB:
        // An extra line of code to work around a DNS issue on Windows:
        let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
        let client = Client::with_options(options).unwrap();
        // Print the databases in our MongoDB cluster:
        println!("Databases:");
        for name in client.list_database_names(None, None).await.unwrap() {
            println!("- {}", name);
        }

        let data_collection: mongodb::Collection<bson::Document> = client.database("game").collection("main_data");

        let bin = vec![1, 2, 3, 4, 5];
// let binary_data = Bson::Binary(bin);

// let insert_doc = doc! { "binary_field": binary_data };

        let data = Data {
            id : None,
            title : "A".to_owned(),
            year : 2,
            plot : "something boring".to_owned(),
            compressed_data : bin
        };
        let serialized_data= bson::to_bson(&data).unwrap();
        let document = serialized_data.as_document().unwrap();

        let insert_result = data_collection.insert_one(document.to_owned(), None).await.unwrap();

        println!("New document ID: {}", insert_result.inserted_id);

    }
}



