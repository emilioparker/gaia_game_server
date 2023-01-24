use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct StoredRegion {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub world_id: Option<ObjectId>,
    pub world_name: String,
    pub region_id: String,
    pub compressed_data : bson::Bson,
}

#[cfg(test)]
mod tests {
    use bson::{Binary, doc, oid::ObjectId};
    use mongodb::{Client, options::{ClientOptions, ResolverConfig}, Collection};

    use crate::long_term_storage_service::db_region::StoredRegion;

    #[tokio::test]
    async fn test_serialize_deserialize_struct_with_bin() {
        let bin = vec![1, 2, 3, 4, 5];
        let bin_clone = bin.clone();

        let bson = bson::Bson::Binary(bson::Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: bin,
        });

        let data = StoredRegion {
            id : None,
            world_id : None,
            world_name : "test_world".to_owned(),
            region_id : "testing".to_owned(),
            compressed_data : bson
        };


        let binary_data: Vec<u8> = match data.compressed_data {
            bson::Bson::Binary(binary) => binary.bytes,
            _ => panic!("Expected Bson::Binary"),
        };

        println!("{:?}", binary_data);
        assert!(binary_data == bin_clone);



    }

    #[tokio::test]
    async fn test_insert_struct() {
        // let client_uri = env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
        let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";

        let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
        let client = Client::with_options(options).unwrap();


        let data_collection: mongodb::Collection<StoredRegion> = client.database("game").collection::<StoredRegion>("test_data");
        let bin = vec![1, 2, 3, 4, 5];
        let bson = bson::Bson::Binary(bson::Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: bin,
        });

        let data = StoredRegion {
            id : None,
            world_id : None,
            world_name : "test_world".to_owned(),
            region_id : "a".to_owned(),
            compressed_data : bson
        };

        let insert_result = data_collection.insert_one(data, None).await.unwrap();
        println!("New document ID: {}", insert_result.inserted_id);
    }

    #[tokio::test]
    async fn test_insert_and_read() {
        // let client_uri = env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
        let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";

        let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
        let client = Client::with_options(options).unwrap();


        let data_collection: mongodb::Collection<StoredRegion> = client.database("game").collection::<StoredRegion>("test_data");
        let bin = vec![1, 2, 3, 4, 5];
        let bson = bson::Bson::Binary(bson::Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: bin,
        });

        let data = StoredRegion {
            id : None,
            world_id : None,
            world_name : "test_world".to_owned(),
            region_id : "a".to_owned()
            ,
            compressed_data : bson
        };

        let insert_result = data_collection.insert_one(data, None).await.unwrap();
        println!("New document ID: {}", insert_result.inserted_id);

        // Look up one document:
        let data_from_db: StoredRegion = data_collection
        .find_one(
            doc! {
                    "region_id": "a"
            },
            None,
        ).await
        .unwrap()
        .expect("Missing 'region a' document.");
        println!("stored_data: {:?}", data_from_db);
    }
}
