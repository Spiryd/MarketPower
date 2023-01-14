use actix_web::{get, post, web::{Data, ReqData, Json}, Responder, HttpResponse, delete};
use serde::{Serialize, Deserialize};
use sqlx::{self, FromRow};

use crate::{AppState, TokenClaims};

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct WatchItem {
    account_id: i32,
    ticker: String,
}


#[derive(Debug, Serialize, Deserialize, FromRow)]
struct WatchItemBody {
    ticker: String,
}


#[get("/watchlist_test")]
async fn fetch_watch_test(state: Data<AppState>) -> impl Responder{
    match sqlx::query_as::<_, WatchItem>("SELECT * FROM watch_list LIMIT 1")
    .fetch_all(&state.db_admin)
    .await
    {
        Ok(eod) => HttpResponse::Ok().json(eod),
        Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
    }
}

#[get("/watchlist")]
async fn fetch_watch_list(state: Data<AppState>, req_user: Option<ReqData<TokenClaims>>) -> impl Responder {

    match req_user {
        Some(user) => {
            let db = match user.security_lvl {
                0 => &state.db_admin,
                1 => &state.db_moderator,
                2 => &state.db_user,
                _ => &state.db_auth
            };
            match sqlx::query_as::<_, WatchItem>("SELECT * FROM watch_list WHERE account_id = $1")
            .bind(user.id)
            .fetch_all(db)
            .await
            {
                Ok(watchlist) => HttpResponse::Ok().json(watchlist),
                Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}

#[post("/watchitem")]
async fn post_watchitem(state: Data<AppState>, req_user: Option<ReqData<TokenClaims>>, body: Json<WatchItemBody>) -> impl Responder {
    match req_user {
        Some(user) => {
            let db = match user.security_lvl {
                0 => &state.db_admin,
                1 => &state.db_moderator,
                2 => &state.db_user,
                _ => &state.db_auth
            };
            let watchitem_body: WatchItemBody = body.into_inner();
            match sqlx::query_as::<_, WatchItem>("INSERT INTO watch_list VALUES ($1, $2) RETURNING account_id, ticker")
            .bind(user.id)
            .bind(watchitem_body.ticker)
            .fetch_all(db)
            .await
            {
                Ok(watchlist) => HttpResponse::Ok().json(watchlist),
                Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}

#[delete("/watchitem")]
async fn delete_portfolio_item(state: Data<AppState>, req_user: Option<ReqData<TokenClaims>>, body: Json<WatchItemBody>) -> impl Responder {
    match req_user {
        Some(user) => {
            let db = match user.security_lvl {
                0 => &state.db_admin,
                1 => &state.db_moderator,
                2 => &state.db_user,
                _ => &state.db_auth
            };
            let watch_item_body: WatchItemBody = body.into_inner();
            match sqlx::query_as::<_, WatchItem>("DELETE FROM portfolio WHERE account_id = $1 AND ticker = $2 RETURNING * ")
            .bind(user.id)
            .bind(watch_item_body.ticker)
            .fetch_all(db)
            .await
            {
                Ok(portfolioitem) => HttpResponse::Ok().json(portfolioitem),
                Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}
