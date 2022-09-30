
use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Deserialize, Serialize)]
struct Employee {
    name: String,
    rate: u32,
}

pub fn start_server(){

    tokio::spawn(async move {

        'receive_loop : loop {
            let promote = warp::post()
            .and(warp::path("employees"))
            .and(warp::path::param::<u32>())
            // Only accept bodies smaller than 16kb...
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            .map(|rate, mut employee: Employee| {
                employee.rate = rate;
                warp::reply::json(&employee)
            });
    
        warp::serve(promote).run(([127, 0, 0, 1], 3030)).await
        }
    });
}