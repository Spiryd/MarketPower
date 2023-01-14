use actix_web::{get, post, web::{Data, ReqData, Json}, Responder, HttpResponse, delete};
use serde::{Serialize, Deserialize};
use sqlx::{self, FromRow};

use crate::{AppState, TokenClaims};

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct PortfolioItem {
    account_id: i32,
    ticker: String,
    amount: f32,
    buy_price: f32,
}


#[derive(Debug, Serialize, Deserialize, FromRow)]
struct CreatePortfolioItem {
    ticker: String,
    amount: f32,
    buy_price: f32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct DeletePortfolioItem {
    ticker: String,
}

#[get("/portfolio_test")]
async fn fetch_portfolio_test(state: Data<AppState>) -> impl Responder{
    match sqlx::query_as::<_, PortfolioItem>("SELECT * FROM portfolio LIMIT 1")
    .fetch_all(&state.db_admin)
    .await
    {
        Ok(eod) => HttpResponse::Ok().json(eod),
        Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
    }
}

#[get("/portfolio")]
async fn fetch_portfolio(state: Data<AppState>, req_user: Option<ReqData<TokenClaims>>) -> impl Responder {

    match req_user {
        Some(user) => {
            let db = match user.security_lvl {
                0 => &state.db_admin,
                1 => &state.db_moderator,
                2 => &state.db_user,
                _ => &state.db_auth
            };
            match sqlx::query_as::<_, PortfolioItem>("SELECT * FROM portfolio WHERE account_id = $1")
            .bind(user.id)
            .fetch_all(db)
            .await
            {
                Ok(portfolio) => HttpResponse::Ok().json(portfolio),
                Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}

#[post("/portfolio_item")]
async fn post_portfolio_item(state: Data<AppState>, req_user: Option<ReqData<TokenClaims>>, body: Json<CreatePortfolioItem>) -> impl Responder {
    match req_user {
        Some(user) => {
            let db = match user.security_lvl {
                0 => &state.db_admin,
                1 => &state.db_moderator,
                2 => &state.db_user,
                _ => &state.db_auth
            };
            let portfolio_item_body: CreatePortfolioItem = body.into_inner();
            match sqlx::query_as::<_, PortfolioItem>("INSERT INTO portfolio VALUES ($1, $2, $3, $4) RETURNING account_id, ticker, amount, buy_price")
            .bind(user.id)
            .bind(portfolio_item_body.ticker)
            .bind(portfolio_item_body.amount)
            .bind(portfolio_item_body.buy_price)
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

#[delete("/portfolio_item")]
async fn delete_portfolio_item(state: Data<AppState>, req_user: Option<ReqData<TokenClaims>>, body: Json<DeletePortfolioItem>) -> impl Responder {
    match req_user {
        Some(user) => {
            let db = match user.security_lvl {
                0 => &state.db_admin,
                1 => &state.db_moderator,
                2 => &state.db_user,
                _ => &state.db_auth
            };
            let portfolio_item_body: DeletePortfolioItem = body.into_inner();
            match sqlx::query_as::<_, PortfolioItem>("DELETE FROM portfolio WHERE account_id = $1 AND ticker = $2 RETURNING * ")
            .bind(user.id)
            .bind(portfolio_item_body.ticker)
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
