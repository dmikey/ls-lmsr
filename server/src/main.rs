use tiny_http::{Server, Response, Method, StatusCode, Header};
use serde_json::json;
use std::sync::{Arc, Mutex};
use lslmsr::market::MarketEngine;
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
struct BuyRequest {
    outcome: String, // "YES" or "NO"
    amount: String,  // in fixed-point string form, e.g. "1000000000000000000"
}

fn main() {
    let server = Server::http("0.0.0.0:8000").unwrap();
    let market = Arc::new(Mutex::new(MarketEngine::new(1_000_000_000_000_000_000)));

    println!("Server running at http://0.0.0.0:8000");

    for mut request in server.incoming_requests() {
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
                use std::fs;

                match fs::read_to_string("./client/index.html") {
                    Ok(contents) => {
                        let response = Response::from_string(contents)
                            .with_header(Header::from_bytes("Content-Type", "text/html").unwrap());
                        request.respond(response).unwrap();
                    }
                    Err(_) => {
                        let response = Response::from_string("index.html not found")
                            .with_status_code(StatusCode(500));
                        request.respond(response).unwrap();
                    }
                }
            }

            (&Method::Post, "/buy") => {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).unwrap();
            
                let parsed: Result<BuyRequest, _> = serde_json::from_str(&body);
                if let Ok(buy) = parsed {
                    let outcome = match buy.outcome.to_uppercase().as_str() {
                        "YES" => lslmsr::types::Outcome::Yes,
                        "NO" => lslmsr::types::Outcome::No,
                        _ => {
                            let response = Response::from_string("Invalid outcome")
                                .with_status_code(StatusCode(400));
                            request.respond(response).unwrap();
                            return;
                        }
                    };
            
                    let amount: u128 = match buy.amount.parse() {
                        Ok(val) => val,
                        Err(_) => {
                            let response = Response::from_string("Invalid amount")
                                .with_status_code(StatusCode(400));
                            request.respond(response).unwrap();
                            return;
                        }
                    };
            
                    let mut engine = market.lock().unwrap();
                    match engine.buy(outcome, amount) {
                        Ok(new_price) => {
                            let body = json!({
                                "yes": (new_price.yes as f64 / 1e18),
                                "no": (new_price.no as f64 / 1e18)
                            })
                            .to_string();
            
                            let response = Response::from_string(body)
                                .with_header(Header::from_bytes("Content-Type", "application/json").unwrap());
                            request.respond(response).unwrap();
                        }
                        Err(err) => {
                            let response = Response::from_string(format!("Trade failed: {:?}", err))
                                .with_status_code(StatusCode(500));
                            request.respond(response).unwrap();
                        }
                    }
                } else {
                    let response = Response::from_string("Malformed JSON")
                        .with_status_code(StatusCode(400));
                    request.respond(response).unwrap();
                }
            }

            (&Method::Post, "/sell") => {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).unwrap();

                let parsed: Result<BuyRequest, _> = serde_json::from_str(&body);
                if let Ok(sell) = parsed {
                    let outcome = match sell.outcome.to_uppercase().as_str() {
                        "YES" => lslmsr::types::Outcome::Yes,
                        "NO" => lslmsr::types::Outcome::No,
                        _ => {
                            let response = Response::from_string("Invalid outcome")
                                .with_status_code(StatusCode(400));
                            request.respond(response).unwrap();
                            return;
                        }
                    };

                    let amount: u128 = match sell.amount.parse() {
                        Ok(val) => val,
                        Err(_) => {
                            let response = Response::from_string("Invalid amount")
                                .with_status_code(StatusCode(400));
                            request.respond(response).unwrap();
                            return;
                        }
                    };

                    let mut engine = market.lock().unwrap();
                    match engine.sell(outcome, amount) {
                        Ok(new_price) => {
                            let body = json!({
                                "yes": (new_price.yes as f64 / 1e18),
                                "no": (new_price.no as f64 / 1e18)
                            })
                            .to_string();

                            let response = Response::from_string(body)
                                .with_header(Header::from_bytes("Content-Type", "application/json").unwrap());
                            request.respond(response).unwrap();
                        }
                        Err(err) => {
                            let response = Response::from_string(format!("Sell failed: {:?}", err))
                                .with_status_code(StatusCode(500));
                            request.respond(response).unwrap();
                        }
                    }
                } else {
                    let response = Response::from_string("Malformed JSON")
                        .with_status_code(StatusCode(400));
                    request.respond(response).unwrap();
                }
            }

            
            (&Method::Post, "/simulate") => {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).unwrap();

                let parsed: Result<BuyRequest, _> = serde_json::from_str(&body);
                if let Ok(buy) = parsed {
                    let outcome = match buy.outcome.to_uppercase().as_str() {
                        "YES" => lslmsr::types::Outcome::Yes,
                        "NO" => lslmsr::types::Outcome::No,
                        _ => {
                            let response = Response::from_string("Invalid outcome")
                                .with_status_code(StatusCode(400));
                            request.respond(response).unwrap();
                            return;
                        }
                    };

                    let amount: u128 = match buy.amount.parse() {
                        Ok(val) => val,
                        Err(_) => {
                            let response = Response::from_string("Invalid amount")
                                .with_status_code(StatusCode(400));
                            request.respond(response).unwrap();
                            return;
                        }
                    };

                    let engine = market.lock().unwrap();
                    let cost = engine.simulate(outcome, amount);

                    let body = json!({
                        "simulated_cost": cost.to_string(),
                        "simulated_cost_float": (cost as f64 / 1e18)
                    })
                    .to_string();

                    let response = Response::from_string(body)
                        .with_header(Header::from_bytes("Content-Type", "application/json").unwrap());
                    request.respond(response).unwrap();
                } else {
                    let response = Response::from_string("Malformed JSON")
                        .with_status_code(StatusCode(400));
                    request.respond(response).unwrap();
                }
            }

            _ => {
                let response = Response::empty(StatusCode(404));
                request.respond(response).unwrap();
            }
        }
    }
}