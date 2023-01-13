use actix_web::{web, App, HttpServer, dev::ServiceRequest, HttpMessage, error::Error};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use dotenv::dotenv;
use actix_web_httpauth::{extractors::{bearer::{self, BearerAuth}, AuthenticationError}, middleware::HttpAuthentication};
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

mod services;
use services::accounts;

extern crate argonautica;

pub struct AppState {
    db: Pool<Postgres>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    id: i32,
    security_lvl: i8,
}

async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)>{
    let jwt_secret: String = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).unwrap();
 
    let token_string = credentials.token();

    let claims: Result<TokenClaims, &str> = token_string.verify_with_key(&key).map_err(|_| "Invalid token");

    match claims {
        Ok(value) => {
            req.extensions_mut().insert(value);
            Ok(req)
        }
        Err(_) => {
            let config = req.app_data::<bearer::Config>().cloned().unwrap_or_default().scope("");

            Err((AuthenticationError::from(config).into(), req))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("Database url must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Error building a connection pool");

    HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(validator);
        App::new()
            .app_data(web::Data::new(AppState {db: pool.clone()}))
            .service(accounts::fetch_acconts)
            .service(
                web::scope("")
                    .wrap(bearer_middleware)
                    .service(accounts::fetch_acconts)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
