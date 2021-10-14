use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, test, http, put, get, post, delete};
use serde::{Deserialize,Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
struct AuctionHouse {
    id: String,
    group: String,
    endpoint: String,
}

#[derive(Deserialize, Serialize)]
struct AuctionHouseCreator {
    group: String,
    endpoint: String,
}

#[post("/")]
async fn post_auction_house(body: web::Json<AuctionHouseCreator>, auction_houses: web::Data<Mutex<Vec<AuctionHouse>>>) -> HttpResponse {
    let new_auction_house = body.into_inner();
    let mut auction_houses_vec = auction_houses.lock().unwrap();
    let auction_house = AuctionHouse {
        id: Uuid::new_v4().to_simple().to_string(),
        group: new_auction_house.group,
        endpoint: new_auction_house.endpoint,
    };
    (*auction_houses_vec).push(auction_house);
    return HttpResponse::Created().finish();
}

#[delete("/{uuid}")]
async fn delete_auction_house(web::Path(uuid): web::Path<String>, auction_houses: web::Data<Mutex<Vec<AuctionHouse>>>) -> HttpResponse {
    let mut auction_houses_vec = auction_houses.lock().unwrap();
    let mut index = 0;
    for a in &*auction_houses_vec {
        if a.id == uuid {
            auction_houses_vec.remove(index);
            return HttpResponse::NoContent().finish();
        }
        index += 1;
    }
    return HttpResponse::NotFound().finish();
}

#[get("/")]
async fn get_auction_houses(auction_houses: web::Data<Mutex<Vec<AuctionHouse>>>) -> HttpResponse {
    let mut auction_houses_vec = &*auction_houses.lock().unwrap();
    if auction_houses_vec.len() == 0 {
        return HttpResponse::NoContent().finish();
    }
    return HttpResponse::Ok().json(auction_houses_vec);
}

/// Setup and deploy the server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mut vec: Vec<AuctionHouse> = Vec::new();
    
    let auction_house_list = web::Data::new(Mutex::new(vec));

    HttpServer::new(move || {
        App::new()
            // Configuring al the differnt ressources
            .service(
                web::scope("/auction-houses")
                // Storing the data for the operator
                .app_data(auction_house_list.clone())
                .service(get_auction_houses)
                .service(delete_auction_house)
                .service(post_auction_house)
            )
        }
    )
    .bind("0.0.0.0:8030")?
    .run()
    .await
}

#[cfg(test)]

#[actix_rt::test]
async fn auction_house_test() {
    let mut vec: Vec<AuctionHouse> = Vec::new();
    let auction_house_list = web::Data::new(Mutex::new(vec));
    let mut app = test::init_service(
        App::new()
        .service(
            web::scope("/auction-houses")
            // Storing the data for the operator
            .app_data(auction_house_list.clone())
            // Configuring all of the differnt ressources
            .service(post_auction_house)
            .service(get_auction_houses)
        )
    ).await;

    let auction_house = AuctionHouseCreator {
        group: "one".to_string(),
        endpoint: "nothing".to_string(),
    };

    let req = test::TestRequest::post().uri("/auction-houses/").set_json(&auction_house).to_request();
    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), http::StatusCode::CREATED);
    
    let req = test::TestRequest::get().uri("/auction-houses/").to_request();
    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);
}