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
use services::companies;
use services::ledger;
use services::watch_list;

pub struct AppState {
    db_auth: Pool<Postgres>,
    db_user: Pool<Postgres>,
    db_moderator: Pool<Postgres>,
    db_admin: Pool<Postgres>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    id: i32,
    security_lvl: i32,
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
    let database_admin_url = std::env::var("DATABASE_URL").expect("Database url must be set");
    let database_auth_url = std::env::var("DATABASE_URL_AUTH").expect("Database url must be set");
    let database_user_url = std::env::var("DATABASE_URL_USER").expect("Database url must be set");
    let database_mod_url = std::env::var("DATABASE_URL_MOD").expect("Database url must be set");

    let pool_admin= PgPoolOptions::new()
        .max_connections(100)
        .connect(&database_admin_url)
        .await
        .expect("Error building a connection pool");

    let pool_auth= PgPoolOptions::new()
        .max_connections(100)
        .connect(&database_auth_url)
        .await
        .expect("Error building a connection pool");

    let pool_user = PgPoolOptions::new()
        .max_connections(100)
        .connect(&database_user_url)
        .await
        .expect("Error building a connection pool");

    let pool_mod = PgPoolOptions::new()
        .max_connections(100)
        .connect(&database_mod_url)
        .await
        .expect("Error building a connection pool");

    HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(validator);
        App::new()
            .app_data(web::Data::new(AppState {db_user: pool_user.clone(), db_admin: pool_admin.clone(), db_auth: pool_auth.clone(), db_moderator: pool_mod.clone()}))
            .service(accounts::fetch_acconts)
            .service(accounts::create_account)
            .service(accounts::basic_auth)
            .service(companies::fetch_comp_test)
            .service(ledger::fetch_ledg_test)
            .service(watch_list::fetch_watch_test)
            .service(
                web::scope("")
                    .wrap(bearer_middleware)
                    .service(companies::fetch_companies)
                    .service(ledger::fetch_ledger)
                    .service(watch_list::post_watchitem)
                    .service(watch_list::fetch_watch_list)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
