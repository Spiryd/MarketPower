use actix_web::{get, post, web::{Data, ReqData}, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use sqlx::{self, FromRow};
use chrono::NaiveDate;

use crate::{AppState, TokenClaims};

#[derive(Serialize, Debug, Deserialize, FromRow)]
struct EoD {
    ticker: String,
    date: NaiveDate,
    open: f32,
    close: f32,
    volume: f64,
}

#[get("ledger")]
async fn fetch_ledger(state: Data<AppState>, req_user: Option<ReqData<TokenClaims>>) -> impl Responder {
    match req_user {
        Some(user) => {
            let db = match user.security_lvl {
                0 => &state.db_admin,
                1 => &state.db_moderator,
                2 => &state.db_user,
                _ => &state.db_auth
            };
            match sqlx::query_as::<_, EoD>("SELECT * FROM ledger")
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

#[get("/ledger_test")]
async fn fetch_ledg_test(state: Data<AppState>,) -> impl Responder{
    match sqlx::query_as::<_, EoD>("SELECT * FROM ledger LIMIT 5")
    .fetch_all(&state.db_admin)
    .await
    {
        Ok(eod) => HttpResponse::Ok().json(eod),
        Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
    }
}
