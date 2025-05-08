use tiny_http::{Server, Response, Method, StatusCode, Header};
use serde_json::json;
use std::sync::{Arc, Mutex};

// Simple price structure
struct Price {
    yes: f64,
    no: f64,
}

// Market engine structure
struct MarketEngine {
    price: Price,
}

impl MarketEngine {
    fn new() -> Self {
        MarketEngine {
            price: Price {
                yes: 0.5,
                no: 0.5,
            },
        }
    }

    fn get_price(&self) -> &Price {
        &self.price
    }
}

fn main() {
    let server = Server::http("0.0.0.0:8000").unwrap();
    let market = Arc::new(Mutex::new(MarketEngine::new()));

    println!("Server running at http://0.0.0.0:8000");

    for request in server.incoming_requests() {
        println!("received request! method: {:?}, url: {:?}, headers: {:?}",
            request.method(),
            request.url(),
            request.headers()
        );

        let method = request.method();
        let path = request.url();
        let market = Arc::clone(&market);

        match (method, path) {
            (&Method::Get, "/price") => {
                let engine = market.lock().unwrap();
                let price = engine.get_price();
                let body = json!({
                    "yes": price.yes.to_string(),
                    "no": price.no.to_string()
                })
                .to_string();

                let content_type = Header::from_bytes("Content-Type", "application/json")
                    .unwrap();
                let response = Response::from_string(body)
                    .with_header(content_type);
                request.respond(response).unwrap();
            }

            (&Method::Get, "/") => {
                let response = Response::from_string("LS-LMSR server running.");
                request.respond(response).unwrap();
            }

            _ => {
                let response = Response::empty(StatusCode(404));
                request.respond(response).unwrap();
            }
        }
    }
}