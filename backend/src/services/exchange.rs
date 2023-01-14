use actix_web::{get, post, web::{Data, ReqData, Json}, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use sqlx::{self, FromRow};

use crate::{AppState, TokenClaims};

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Exchange {
    mic: String,
    name: String,
}

#[get("/exchange")]
async fn fetch_exchange(state: Data<AppState>, req_user: Option<ReqData<TokenClaims>>) -> impl Responder {
    match req_user {
        Some(user) => {
            let db = match user.security_lvl {
                0 => &state.db_admin,
                1 => &state.db_moderator,
                2 => &state.db_user,
                _ => &state.db_auth
            };
            match sqlx::query_as::<_, Exchange>("SELECT * FROM exchange")
            .fetch_all(db)
            .await
            {
                Ok(exchange) => HttpResponse::Ok().json(exchange),
                Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}
