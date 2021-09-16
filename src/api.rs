use crate::choose;
use itertools::Itertools;
use rand::Rng;
use rocket::http::{CookieJar, Status};
use rocket::response::status::BadRequest;
use rocket::serde::json::Json;
use rocket::{get, post, routes, Route};
use serde::Deserialize;
use serde_json::{json, Value};
use std::str::FromStr;

static ERROR_MESSAGES: &[&str] = &[
    "If you were meant to have that, you already would",
    "Monitor's on vacation, sorry",
    "You can't get ye flask!",
];

fn gen_tarot() -> Vec<i32> {
    let mut rng = rand::thread_rng();
    let mut res: Vec<i32> = Vec::with_capacity(3);
    while res.len() < 3 {
        let n = rng.gen_range(-1..20);
        if !res.contains(&n) {
            res.push(n);
        }
    }

    res
}

#[get("/api/getActiveBets")]
pub(crate) fn get_active_bets() -> Json<Vec<()>> {
    Json(vec![])
}

#[get("/api/getUser")]
pub(crate) fn get_user(cookies: &CookieJar<'_>) -> Json<Value> {
    Json(json!({
        "id": "be457c4e-79e6-4016-94f5-76c6705741bb",
        "email": "before@sibr.dev",
        // disable ability to change email on frontend
        "appleId": "what's umpdog",
        "lightMode": cookies.get_pending("light_mode")
            .and_then(|s| bool::from_str(s.value()).ok())
            .unwrap_or(false),
        "verified": true,
        "coins": "Infinity",
        "peanuts": cookies.get_pending("peanuts").and_then(|t| t.value().parse::<i32>().ok()).unwrap_or(0),
        "squirrels": cookies.get_pending("squirrels").and_then(|t| t.value().parse::<i32>().ok()).unwrap_or(0),
        "idol": choose(IDOL_CHOICES),
        "favoriteTeam": cookies.get_pending("favorite_team")
            .map(|s| {
                let s = s.value();
                if s == "_before_change_team" {
                    Value::Null
                } else {
                    Value::String(s.to_owned())
                }
            })
            .unwrap_or_else(|| Value::String(choose(TEAM_CHOICES).to_owned())),
        "unlockedShop": true,
        "unlockedElection": true,
        "spread": cookies.get_pending("tarot_spread")
                    .and_then(|t| t.value().split(',').map(|t| t.parse::<i32>().ok()).collect::<Option<Vec<i32>>>())
                    .unwrap_or_else(gen_tarot),
        "snacks": {
            "Forbidden_Knowledge_Access": 1,
            "Stadium_Access": 1,
            "Wills_Access": 1,
            "Flutes": 1,
            "Tarot_Reroll": 1,
            "Peanuts": cookies.get_pending("peanuts").and_then(|t| t.value().parse::<i32>().ok()).unwrap_or(0),
        },
        "snackOrder": [
            "Forbidden_Knowledge_Access",
            "Stadium_Access",
            "Wills_Access",
            "Flutes",
            "Tarot_Reroll",
            "Peanuts",
            "E",
            "E",
        ],
        "packSize": 8,
        // set all these to reasonably high values to avoid rendering the "what to do next" actions
        // in the bulletin
        "trackers": {
            "BEGS": 3,
            "BETS": 10,
            "VOTES_CAST": 1,
            "SNACKS_BOUGHT": 2,
            "SNACK_UPGRADES": 3,
        },
        "relics": {
            "Idol_Strikeouts": 0,
            "Idol_Shutouts": 0,
            "Idol_Homers": 0,
            "Idol_Hits": 0,
        },
    }))
}

#[get("/api/getUserRewards")]
pub(crate) fn get_user_rewards() -> Json<Option<()>> {
    Json(None)
}

#[get("/api/getUserNotifications")]
pub(crate) fn get_user_notifications() -> Json<Option<()>> {
    Json(None)
}

#[post("/api/clearUserNotifications")]
pub(crate) fn clear_user_notifications() -> Json<Option<()>> {
    Json(None)
}

#[derive(Deserialize)]
pub(crate) struct CardOrderUpdate {
    spread: Vec<i32>,
}

#[post("/api/reorderCards", data = "<order_update>")]
pub(crate) fn reorder_cards(
    cookies: &CookieJar<'_>,
    order_update: Json<CardOrderUpdate>,
) -> Json<Value> {
    cookies.add(crate::new_cookie(
        "tarot_spread",
        order_update.spread.iter().map(|i| i.to_string()).join(","),
    ));
    Json(json!({"message": "New Spread preserved"}))
}

#[post("/api/dealCards")]
pub(crate) fn deal_cards(cookies: &CookieJar<'_>) -> Json<Value> {
    let spread = gen_tarot();
    cookies.add(crate::new_cookie(
        "tarot_spread",
        spread.iter().map(|i| i.to_string()).join(","),
    ));
    Json(json!({"spread": spread, "message": "New Spread preserved"}))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SnackPurchase {
    pub(crate) snack_id: String,
}

#[post("/api/buySnackNoUpgrade", data = "<purchase>")]
pub(crate) fn buy_snack(
    cookies: &CookieJar<'_>,
    purchase: Json<SnackPurchase>,
) -> (Status, Json<Value>) {
    if purchase.snack_id == "Peanuts" {
        let peanuts = cookies
            .get_pending("peanuts")
            .and_then(|t| t.value().parse::<i32>().ok())
            .unwrap_or(0)
            + 1000;

        cookies.add(crate::new_cookie("peanuts", peanuts.to_string()));
        (
            Status::Ok,
            Json(json!({
                "message": "Peanuts purchased"
            })),
        )
    } else {
        let message = choose(ERROR_MESSAGES);
        (
            Status::BadRequest,
            Json(json!({
                "error": message,
                "message": message,
            })),
        )
    }
}

#[post("/api/buyADangSquirrel")]
pub(crate) fn buy_a_dang_squirrel(cookies: &CookieJar<'_>) -> Json<Value> {
    cookies.add(crate::new_cookie(
        "squirrels",
        (cookies
            .get_pending("squirrels")
            .and_then(|t| t.value().parse::<i32>().ok())
            .unwrap_or(0)
            + 1)
        .to_string(),
    ));
    Json(json!({"message": "Bought a squirrel."}))
}

#[derive(Deserialize)]
pub(crate) struct EatADangPeanut {
    pub(crate) amount: i32,
}

#[post("/api/eatADangPeanut", data = "<dang_peanut>")]
pub(crate) fn eat_a_dang_peanut(
    cookies: &CookieJar<'_>,
    dang_peanut: Json<EatADangPeanut>,
) -> Json<Value> {
    cookies.add(crate::new_cookie(
        "peanuts",
        (cookies
            .get_pending("peanuts")
            .and_then(|t| t.value().parse::<i32>().ok())
            .unwrap_or(0)
            - dang_peanut.amount)
            .to_string(),
    ));
    Json(json!({}))
}

#[post("/api/buyADangPeanut")]
pub(crate) fn buy_a_dang_peanut(cookies: &CookieJar<'_>) -> Json<Value> {
    let peanuts = cookies
        .get_pending("peanuts")
        .and_then(|t| t.value().parse::<i32>().ok())
        .unwrap_or(0)
        + 1000;

    cookies.add(crate::new_cookie("peanuts", peanuts.to_string()));
    Json(json!({
        "message": format!("You receive 1000 peanuts. You now have {} peanuts", peanuts)
    }))
}

#[post("/api/buyUpdateFavoriteTeam")]
pub(crate) fn buy_flute(cookies: &CookieJar<'_>) -> Json<Value> {
    cookies.add(crate::new_cookie("favorite_team", "_before_change_team"));
    Json(json!({"message": "Reload this page to choose a new team."}))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FavoriteTeamUpdate {
    #[serde(alias = "newTeamId")]
    pub(crate) team_id: String,
}

#[post("/api/updateFavoriteTeam", data = "<new_favorite>")]
pub(crate) fn update_favourite_team(
    cookies: &CookieJar<'_>,
    new_favorite: Json<FavoriteTeamUpdate>,
) -> Json<Value> {
    cookies.add(crate::new_cookie(
        "favorite_team",
        new_favorite.team_id.to_string(),
    ));
    Json(json!({ "message": "You now remember the Before of a new team." }))
}

pub(crate) fn mocked_error_routes() -> Vec<Route> {
    macro_rules! mock {
        ($uri:expr) => {{
            #[post($uri)]
            pub(crate) fn mock_error() -> BadRequest<Json<Value>> {
                let message = choose(ERROR_MESSAGES);
                BadRequest(Some(Json(json!({
                    "error": message,
                    "message": message,
                }))))
            }
            routes![mock_error]
        }};
    }

    vec![
        mock!("/api/buySlot"),
        mock!("/api/buySnack"),
        mock!("/api/logBeg"),
        mock!("/api/reorderSnacks"),
        mock!("/api/sellSlot"),
        mock!("/api/sellSnack"),
        mock!("/api/buyIncreaseMaxBet"),
        mock!("/api/buyIncreaseDailyCoins"),
        mock!("/api/buyRelic"),
        mock!("/api/buyUnlockShop"),
        mock!("/api/buyVote"),
    ]
    .concat()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Settings {
    pub(crate) light_mode: bool,
}

#[post("/api/updateSettings", data = "<settings>")]
pub(crate) fn update_settings(cookies: &CookieJar<'_>, settings: Json<Settings>) -> Json<Value> {
    cookies.add(crate::new_cookie(
        "light_mode",
        settings.light_mode.to_string(),
    ));
    Json(json!({ "message": "Settings updated" }))
}

// Should be a list of players that have been around (in the database) since Season 1
static IDOL_CHOICES: &[&str] = &[
    "04e14d7b-5021-4250-a3cd-932ba8e0a889", // Jaylen Hotdogfingers
    "083d09d4-7ed3-4100-b021-8fbe30dd43e8", // Jessica Telephone
    "1f159bab-923a-4811-b6fa-02bfde50925a", // NaN
    "20be1c34-071d-40c6-8824-dde2af184b4d", // Qais Dogwalker
    "20fd71e7-4fa0-4132-9f47-06a314ed539a", // Lars Taylor
    "338694b7-6256-4724-86b6-3884299a5d9e", // PolkaDot Patterson
    "493a83de-6bcf-41a1-97dd-cc5e150548a3", // Boyfriend Monreal
    "53e701c7-e3c8-4e18-ba05-9b41b4b64cda", // Marquez Clark
    "a3947fbc-50ec-45a4-bca4-49ffebb77dbe", // Chorby Short
    "c675fcdf-6117-49a6-ac32-99a89a3a88aa", // Valentine Games
    "c6a277c3-d2b5-4363-839b-950896a5ec5e", // Mike Townsend
    "d4a10c2a-0c28-466a-9213-38ba3339b65e", // Richmond Harrison
    "f2a27a7e-bf04-4d31-86f5-16bfa3addbe7", // Winnie Hess
    "f70dd57b-55c4-4a62-a5ea-7cc4bf9d8ac1", // Tillman Henderson
];

// All 20 original Season 1 teams, no Breach/Lift
static TEAM_CHOICES: &[&str] = &[
    "105bc3ff-1320-4e37-8ef0-8d595cb95dd0", // Garages
    "23e4cbc1-e9cd-47fa-a35b-bfa06f726cb7", // Pies
    "36569151-a2fb-43c1-9df7-2df512424c82", // Millennials
    "3f8bbb15-61c0-4e3f-8e4a-907a5fb1565e", // Flowers
    "57ec08cc-0411-4643-b304-0e80dbc15ac7", // Wild Wings
    "747b8e4a-7e50-4638-a973-ea7950a3e739", // Tigers
    "7966eb04-efcc-499b-8f03-d13916330531", // Magic
    "878c1bf6-0d21-4659-bfee-916c8314d69c", // Tacos
    "8d87c468-699a-47a8-b40d-cfb73a5660ad", // Crabs
    "979aee4a-6d80-4863-bf1c-ee1a78e06024", // Fridays
    "9debc64f-74b7-4ae1-a4d6-fce0144b6ea5", // Spies
    "a37f9158-7f82-46bc-908c-c9e2dda7c33b", // Jazz Hands
    "adc5b394-8f76-416d-9ce9-813706877b84", // The Breath Mints.
    "b024e975-1c4a-4575-8936-a3754a08806a", // Steaks
    "b63be8c2-576a-4d6e-8daf-814f8bcea96f", // Dale
    "b72f3061-f573-40d7-832a-5ad475bd7909", // Lovers
    "bfd38797-8404-4b38-8b82-341da28b1f83", // Shoe Thieves
    "ca3f1c8c-c025-4d8e-8eef-5be6accbeb16", // Firefighters
    "eb67ae5e-c4bf-46ca-bbbc-425cd34182ff", // Moist Talkers
    "f02aeae2-5e6a-4098-9842-02d2273f25c7", // Sunbeams
];
