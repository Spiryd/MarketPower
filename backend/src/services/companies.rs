use actix_web::{get, web::{Data, ReqData}, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use sqlx::{self, FromRow};

use crate::{AppState, TokenClaims};

#[derive(Serialize, Deserialize, Debug, FromRow)]
struct Company {
    ticker: String,
    name: String,
    sector: String,
    industry: String,
    mic: String,
}

#[get("/companies")]
async fn fetch_companies(state: Data<AppState>, req_user: Option<ReqData<TokenClaims>>) -> impl Responder{
    match req_user {
        Some(user) => {
            let db = match user.security_lvl {
                0 => &state.db_admin,
                1 => &state.db_moderator,
                2 => &state.db_user,
                _ => &state.db_auth
            };
            match sqlx::query_as::<_, Company>("SELECT * FROM company")
            .fetch_all(db)
            .await
            {
                Ok(companies) => HttpResponse::Ok().json(companies),
                Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}

#[get("/companies_test")]
async fn fetch_comp_test(state: Data<AppState>,) -> impl Responder{
    match sqlx::query_as::<_, Company>("SELECT * FROM company LIMIT 5")
    .fetch_all(&state.db_admin)
    .await
    {
        Ok(companies) => HttpResponse::Ok().json(companies),
        Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
    }
}
