use actix_web::{get, post, web::{Data, Json, ReqData}, Responder, HttpResponse};
use actix_web_httpauth::extractors::basic::BasicAuth;
use serde::{Serialize, Deserialize};
use sqlx::{self, FromRow, Postgres, Pool};
use chrono::NaiveDateTime;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use sha2::Sha256;
use argonautica::{Hasher, Verifier};
use rand::Rng;

use crate::{AppState, TokenClaims};

#[derive(Deserialize)]
struct CreateAccountBody {
    login: String,
    password: String,
}

#[derive(Serialize, FromRow)]
struct AccountNoPassword {
    id: i32,
    login: String,
}

#[derive(Serialize, FromRow)]
struct AuthUser {
    id: i32,
    login: String,
    password: String,
    security_lvl: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Account {
    id: i32,
    login: String,
    hashed_password: String,
    salt: String,
    security_lvl: i32,
}

fn generate_salt() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
    abcdefghijklmnopqrstuvwxyz\
    0123456789";
    const SALT_LEN: usize = 8;
    let mut rng = rand::thread_rng();

    let salt: String = (0..SALT_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    salt
}

#[get("/auth")]
async fn basic_auth(state: Data<AppState>, credentials: BasicAuth) -> impl Responder {
    let jwt_secret: Hmac<Sha256> = Hmac::new_from_slice(
        std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set!")
            .as_bytes()
    ).unwrap();
    let login = credentials.user_id();
    let password = credentials.password();
    match password {
        None => HttpResponse::Unauthorized().json("Must provide longin and password"),
        Some(pass) => {
            match sqlx::query_as::<_, AuthUser>(
                "SELECT id, login, hashed_password, salt FROM account WHERE login = $1"
            )
            .bind(login.to_string())
            .fetch_one(&state.db)
            .await 
            {
                Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
                Ok(user) => {
                    let hash_secret = std::env::var("HASH_SECRET").expect("HASH_SECRET must be set!");
                    let mut verifier = Verifier::default();
                    let is_valid = verifier
                        .with_hash(user.password)
                        .with_password(pass)
                        .with_secret_key(hash_secret)
                        .with_additional_data("test")
                        .verify()
                        .unwrap();
                    if is_valid {
                        let claims = TokenClaims {id: user.id, security_lvl: user.security_lvl};
                        let token_str = claims.sign_with_key(&jwt_secret).unwrap();
                        HttpResponse::Ok().json(token_str)
                    } else {
                        HttpResponse::Unauthorized().json("incorrect login or password")
                    }
                }
            }
        },
    }
}

#[get("/accounts")]
async fn fetch_acconts(state: Data<AppState>) -> impl Responder {
    match sqlx::query_as::<_, Account>("SELECT * FROM account").fetch_all(&state.db).await {
        Ok(account)=> HttpResponse::Ok().json(account),
        Err(_) => HttpResponse::NotFound().json("No accounts found"),
    }
}

async fn chceck_for_login_avaliablility(login: String, database: Pool<Postgres>) -> bool {
    let resault = sqlx::query_as::<_, Account>("SELECT * FROM account WHERE login = $1").bind(login).fetch_all(&database).await;
    match resault {
        Ok(x) => {
            if x.len() != 0 {
                false
            } else {
                true
            }
        },
        Err(_) => false,
    }
}

#[post("/account")]
async fn create_account(state: Data<AppState>, body: Json<CreateAccountBody>) -> impl Responder {
    let account: CreateAccountBody = body.into_inner();
    let hash_secret = std::env::var("HASH_SECRET").expect("HASH_SECRET must be set!");
    let mut hasher = Hasher::default();
    let salt = generate_salt();

    if !chceck_for_login_avaliablility(account.login.clone(), state.db.clone()).await { 
        return HttpResponse::InternalServerError().json("login not avaliable");
    }

    let hash = hasher
        .with_password(account.password)
        .with_additional_data(&salt)
        .with_secret_key(hash_secret)
        .hash()
        .unwrap();

    match sqlx::query_as::<_, AccountNoPassword>(
        "Insert INTO account (login, hashed_password, salt, security_lvl)
        VALUES ($1, $2, $3, $4)
        RETURNING id, login"
    )
    .bind(account.login)
    .bind(hash)
    .bind(salt)
    .bind(2)
    .fetch_one(&state.db)
    .await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error))
    }
}

#[derive(Deserialize)]
struct CreateArticleBody {
    title: String,
    content: String,
}

#[derive(Serialize, FromRow)]
struct Article {
    id: i32,
    title: String,
    content: String,
    published_by: i32,
    published_on: Option<NaiveDateTime>,
}

//wxapmple for future
#[post("/article")]
async fn create_article(state: Data<AppState>, req_user: Option<ReqData<TokenClaims>>, body: Json<CreateArticleBody>) -> impl Responder {
    match req_user {
        Some(user) => {
            let article: CreateArticleBody = body.into_inner();
            match sqlx::query_as::<_, Article>(
                "INSERT INTO articles (title, content, published_by)
                VALUES ($1, $2, $3)
                RETURNING id, title, content, published_by, published_on",
            )
            .bind(article.title)
            .bind(article.content)
            .bind(user.id)
            .fetch_one(&state.db)
            .await
            {
                Ok(articles) => HttpResponse::Ok().json(articles),
                Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}
