use chrono::{Duration, Utc};
use rocket::{get, post, response::Redirect, serde::json::Json};
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;

use crate::{
    CONFIG, SQL,
    routes::api::{V1ApiError, V1ApiResponse, V1ApiResponseType, v1::guards::AuthenticatedUser},
    utils::snowflake::next_id,
};

#[derive(Deserialize)]
struct DiscordTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    scope: String,
}

#[derive(Deserialize)]
struct DiscordUser {
    id: String,
    username: String,
    avatar: Option<String>,
}

fn redirect_uri() -> String {
    format!("{}/auth/connect/discord", CONFIG.server_domain)
}

fn discord_config() -> Option<(&'static str, &'static str)> {
    CONFIG
        .discord
        .as_ref()
        .map(|d| (d.client_id.as_str(), d.client_secret.as_str()))
}

async fn exchange_code(code: &str) -> Result<DiscordTokenResponse, String> {
    let (client_id, client_secret) =
        discord_config().ok_or_else(|| "Discord OAuth is not configured".to_string())?;

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", &redirect_uri()),
    ];

    let client = reqwest::Client::new();
    let res = client
        .post("https://discord.com/api/oauth2/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            error!("Discord token exchange request failed: {e}");
            "Failed to contact Discord".to_string()
        })?;

    if !res.status().is_success() {
        error!("Discord token exchange returned {}", res.status());
        return Err("Discord rejected the authorisation code".to_string());
    }

    res.json::<DiscordTokenResponse>().await.map_err(|e| {
        error!("Failed to parse Discord token response: {e}");
        "Unexpected response from Discord".to_string()
    })
}

async fn fetch_discord_user(access_token: &str) -> Result<DiscordUser, String> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            error!("Discord /users/@me request failed: {e}");
            "Failed to contact Discord".to_string()
        })?;

    if !res.status().is_success() {
        error!("Discord /users/@me returned {}", res.status());
        return Err("Failed to retrieve your Discord account".to_string());
    }

    res.json::<DiscordUser>().await.map_err(|e| {
        error!("Failed to parse Discord user: {e}");
        "Unexpected response from Discord".to_string()
    })
}

pub async fn refresh_discord_token(user_id: i64, refresh_token: &str) -> Result<(), String> {
    let (client_id, client_secret) =
        discord_config().ok_or_else(|| "Discord OAuth is not configured".to_string())?;

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];

    let client = reqwest::Client::new();
    let res = client
        .post("https://discord.com/api/oauth2/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            error!("Discord token refresh failed: {e}");
            "Failed to contact Discord".to_string()
        })?;

    if !res.status().is_success() {
        error!("Discord token refresh returned {}", res.status());
        return Err("Discord rejected the refresh token".to_string());
    }

    let tokens: DiscordTokenResponse = res.json().await.map_err(|e| {
        error!("Failed to parse Discord refresh response: {e}");
        "Unexpected response from Discord".to_string()
    })?;

    let expires_at = Utc::now() + Duration::seconds(tokens.expires_in);

    sqlx::query!(
        "UPDATE discord_connections
         SET access_token  = $1,
             refresh_token = $2,
             expires_at    = $3
         WHERE user_id = $4",
        tokens.access_token,
        tokens.refresh_token,
        expires_at,
        user_id,
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to update refreshed Discord tokens for user {user_id}: {e}");
        "Internal error".to_string()
    })?;

    Ok(())
}

pub fn start_token_refresh_task() {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(6 * 60 * 60));

        loop {
            interval.tick().await;

            if SQL.get().is_none() {
                continue;
            }

            let rows = sqlx::query!(
                "SELECT user_id, refresh_token FROM discord_connections WHERE expires_at < NOW() + INTERVAL '3 days'"
            )
            .fetch_all(&*SQL)
            .await;

            if let Ok(connections) = rows {
                for conn in connections {
                    if let Err(e) = refresh_discord_token(conn.user_id, &conn.refresh_token).await {
                        error!(
                            "Background token refresh failed for user {}: {}",
                            conn.user_id, e
                        );
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    });
}

fn done_redirect(token: &str) -> Redirect {
    Redirect::to(format!("/auth/done?token={}", token))
}

fn error_redirect(msg: &str) -> Redirect {
    Redirect::to(format!("/auth/done?error={}", urlencoding::encode(msg)))
}

async fn handle_login(
    discord_id: &str,
    access_token: &str,
    refresh_token: &str,
    expires_at: chrono::DateTime<Utc>,
) -> Redirect {
    let user = match sqlx::query!(
        "SELECT user_id FROM discord_connections WHERE discord_id = $1",
        discord_id
    )
    .fetch_optional(&*SQL)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return error_redirect("No account is linked to this Discord profile"),
        Err(e) => {
            error!("DB error checking discord login: {e}");
            return error_redirect("Internal error");
        }
    };

    let _ = sqlx::query!(
        "UPDATE discord_connections SET access_token = $1, refresh_token = $2, expires_at = $3 WHERE discord_id = $4",
        access_token, refresh_token, expires_at, discord_id
    )
    .execute(&*SQL)
    .await;

    let token = Uuid::new_v4().to_string();
    let token_id = next_id();

    if let Err(e) = sqlx::query!(
        "INSERT INTO user_tokens (id, user_id, token, expires_at) VALUES ($1, $2, $3, NOW() + INTERVAL '30 days')",
        token_id,
        user.user_id,
        token,
    )
    .execute(&*SQL)
    .await
    {
        error!("Failed to issue session token for Discord login: {e}");
        return error_redirect("Account found but could not issue session token");
    }

    done_redirect(&token)
}

#[get("/auth/connect/discord?<code>&<state>")]
pub async fn discord_callback(code: Option<String>, state: Option<String>) -> Redirect {
    if discord_config().is_none() {
        return error_redirect("Discord OAuth is not configured on this server");
    }

    let code = match code {
        Some(c) if !c.is_empty() => c,
        _ => return error_redirect("Missing authorisation code from Discord"),
    };
    let state = match state {
        Some(s) if !s.is_empty() => s,
        _ => return error_redirect("Missing state parameter"),
    };

    let discord_tokens = match exchange_code(&code).await {
        Ok(t) => t,
        Err(msg) => return error_redirect(&msg),
    };

    let scopes: Vec<&str> = discord_tokens.scope.split_whitespace().collect();
    if !scopes.contains(&"identify")
        || !scopes.contains(&"openid")
        || !scopes.contains(&"sdk.social_layer")
    {
        return error_redirect("The required Discord permissions were not granted by the user");
    }

    let discord_user = match fetch_discord_user(&discord_tokens.access_token).await {
        Ok(u) => u,
        Err(msg) => return error_redirect(&msg),
    };

    let expires_at = Utc::now() + Duration::seconds(discord_tokens.expires_in);

    if state == "login" {
        handle_login(
            &discord_user.id,
            &discord_tokens.access_token,
            &discord_tokens.refresh_token,
            expires_at,
        )
        .await
    } else if let Some(invite_code) = state.strip_prefix("register:") {
        handle_register(
            discord_user,
            &discord_tokens.access_token,
            &discord_tokens.refresh_token,
            expires_at,
            invite_code,
        )
        .await
    } else if let Some(emunex_token) = state.strip_prefix("link:") {
        handle_link(
            &discord_user.id,
            &discord_tokens.access_token,
            &discord_tokens.refresh_token,
            expires_at,
            emunex_token,
        )
        .await
    } else {
        error_redirect("Invalid state parameter")
    }
}

async fn handle_register(
    discord_user: DiscordUser,
    access_token: &str,
    refresh_token: &str,
    expires_at: chrono::DateTime<Utc>,
    invite_code: &str,
) -> Redirect {
    let discord_id = &discord_user.id;
    let invite = match sqlx::query!(
        "SELECT id FROM invite_codes WHERE code = $1 AND used_by IS NULL",
        invite_code
    )
    .fetch_optional(&*SQL)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return error_redirect("That invite code is invalid or has already been used"),
        Err(e) => {
            error!("DB error checking invite code: {e}");
            return error_redirect("Internal error");
        }
    };

    match sqlx::query!(
        "SELECT user_id FROM discord_connections WHERE discord_id = $1",
        discord_id
    )
    .fetch_optional(&*SQL)
    .await
    {
        Ok(Some(_)) => return error_redirect("This Discord account is already registered"),
        Err(e) => {
            error!("DB error checking discord_connections: {e}");
            return error_redirect("Internal error");
        }
        Ok(None) => {}
    }

    let taken = sqlx::query!(
        "SELECT id FROM users WHERE username = $1",
        discord_user.username
    )
    .fetch_optional(&*SQL)
    .await
    .unwrap_or(None);

    let username = match taken {
        Some(_) => format!("user_{}", &discord_id[..discord_id.len().min(8)]),
        None => discord_user.username.clone(),
    };
    let user_id = next_id();

    let mut db_avatar_hash: Option<String> = None;
    if let Some(av) = discord_user.avatar {
        let url = format!(
            "https://cdn.discordapp.com/avatars/{}/{}.webp",
            discord_id, av
        );
        if let Ok(res) = reqwest::get(&url).await {
            if let Ok(bytes) = res.bytes().await {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let mut buf = std::io::Cursor::new(Vec::new());
                    if img.write_to(&mut buf, image::ImageFormat::WebP).is_ok() {
                        let webp_bytes = buf.into_inner();
                        let hash = crate::utils::s3::compute_md5(&webp_bytes);
                        let path = format!("/avatars/{}/{}.webp", user_id, hash);
                        if crate::utils::s3::upload_object(&path, &webp_bytes)
                            .await
                            .is_ok()
                        {
                            db_avatar_hash = Some(hash);
                        }
                    }
                }
            }
        }
    }

    let mut tx = match SQL.begin().await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to begin transaction: {e}");
            return error_redirect("Internal error");
        }
    };

    if let Err(e) = sqlx::query("INSERT INTO users (id, username, avatar_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(username)
        .bind(db_avatar_hash)
        .execute(&mut *tx)
        .await
    {
        error!("Failed to insert Discord user: {e}");
        return error_redirect("Internal error — could not create account");
    }

    if let Err(e) = sqlx::query!(
        "INSERT INTO discord_connections (user_id, discord_id, access_token, refresh_token, expires_at)
         VALUES ($1, $2, $3, $4, $5)",
        user_id,
        discord_id,
        access_token,
        refresh_token,
        expires_at,
    )
    .execute(&mut *tx)
    .await
    {
        error!("Failed to insert discord_connections row: {e}");
        return error_redirect("Internal error");
    }

    if let Err(e) = sqlx::query!(
        "UPDATE invite_codes SET used_by = $1, used_at = NOW() WHERE id = $2",
        user_id,
        invite.id
    )
    .execute(&mut *tx)
    .await
    {
        error!("Failed to mark invite as used: {e}");
        return error_redirect("Internal error");
    }

    if let Err(e) = tx.commit().await {
        error!("Failed to commit registration transaction: {e}");
        return error_redirect("Internal error");
    }

    let token = Uuid::new_v4().to_string();
    let token_id = next_id();

    if let Err(e) = sqlx::query!(
        "INSERT INTO user_tokens (id, user_id, token, expires_at)
         VALUES ($1, $2, $3, NOW() + INTERVAL '30 days')",
        token_id,
        user_id,
        token,
    )
    .execute(&*SQL)
    .await
    {
        error!("Failed to insert session token for new Discord user: {e}");
        return error_redirect("Account created but could not issue session token");
    }

    done_redirect(&token)
}

async fn handle_link(
    discord_id: &str,
    access_token: &str,
    refresh_token: &str,
    expires_at: chrono::DateTime<Utc>,
    emunex_token: &str,
) -> Redirect {
    let user = match sqlx::query!(
        "SELECT u.id FROM users u
         JOIN user_tokens ut ON u.id = ut.user_id
         WHERE ut.token = $1 AND ut.expires_at > NOW()",
        emunex_token
    )
    .fetch_optional(&*SQL)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return error_redirect("Your emuNEX session is invalid or has expired"),
        Err(e) => {
            error!("DB error looking up emuNEX token: {e}");
            return error_redirect("Internal error");
        }
    };

    if let Err(e) = sqlx::query!(
        "INSERT INTO discord_connections (user_id, discord_id, access_token, refresh_token, expires_at)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (user_id) DO UPDATE
           SET discord_id    = EXCLUDED.discord_id,
               access_token  = EXCLUDED.access_token,
               refresh_token = EXCLUDED.refresh_token,
               expires_at    = EXCLUDED.expires_at",
        user.id,
        discord_id,
        access_token,
        refresh_token,
        expires_at,
    )
    .execute(&*SQL)
    .await
    {
        if let Some(db_err) = e.as_database_error() {
            if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                return error_redirect("This Discord account is already linked to another user");
            }
        }
        error!("Failed to upsert discord_connections for user {}: {e}", user.id);
        return error_redirect("Internal error");
    }

    done_redirect(emunex_token)
}

#[get("/auth/discord/authorize?<action>&<invite_code>")]
pub async fn discord_authorize(
    action: String,
    invite_code: Option<String>,
    auth_token: Option<crate::routes::api::v1::guards::AuthToken>,
) -> Redirect {
    let (client_id, _) = match discord_config() {
        Some(c) => c,
        None => return error_redirect("Discord OAuth is not configured on this server"),
    };

    let state = match action.as_str() {
        "login" => "login".to_string(),
        "register" => {
            let code = invite_code.unwrap_or_default();
            if code.is_empty() {
                return error_redirect("An invite code is required to register with Discord");
            }
            format!("register:{}", code)
        }
        "link" => {
            let t = match auth_token {
                Some(tok) => tok.0,
                None => return error_redirect("Missing session token for Discord linking"),
            };
            format!("link:{}", t)
        }
        _ => return error_redirect("Invalid Discord authorization action"),
    };

    let authorize_url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=identify+openid+sdk.social_layer&state={}",
        urlencoding::encode(client_id),
        urlencoding::encode(&redirect_uri()),
        urlencoding::encode(&state)
    );

    Redirect::to(authorize_url)
}

pub async fn sync_user_discord_widget(user_id: i64) -> Result<(), String> {
    let (client_id, _) =
        discord_config().ok_or_else(|| "Discord OAuth is not configured".to_string())?;
    let bot_token = CONFIG
        .discord
        .as_ref()
        .map(|d| d.bot_token.as_str())
        .unwrap_or("");
    if bot_token.is_empty() {
        return Err("Discord bot token is missing".to_string());
    }

    let user_info = sqlx::query!(
        "SELECT u.username, d.discord_id FROM users u 
         JOIN discord_connections d ON u.id = d.user_id 
         WHERE u.id = $1",
        user_id
    )
    .fetch_optional(&*SQL)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "User or connection not found".to_string())?;

    let username = user_info.username;
    let discord_id = user_info.discord_id;

    let games = sqlx::query!(
        r#"SELECT r.title, r.id, r.console, r.image_hash, ur.play_count, ur.last_played 
           FROM user_roms ur
           JOIN roms r ON ur.rom_id = r.id
           WHERE ur.user_id = $1
           ORDER BY ur.play_count DESC"#,
        user_id
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| e.to_string())?;

    let total_plays: i64 = games.iter().map(|g| g.play_count as i64).sum();

    let mut dynamic =
        vec![serde_json::json!({ "type": 1, "name": "username", "value": username.clone() })];

    if total_plays > 0 {
        dynamic.push(serde_json::json!({ "type": 2, "name": "num_plays", "value": total_plays }));
    }

    if let Some(most_played) = games.get(0) {
        dynamic.push(
            serde_json::json!({ "type": 1, "name": "most_played", "value": most_played.title }),
        );
        let cover_url = format!(
            "{}/storage/covers_small/{}/{}/{}.webp",
            CONFIG.server_domain, most_played.console, most_played.id, most_played.image_hash
        );
        dynamic.push(
            serde_json::json!({ "type": 3, "name": "hero_image", "value": { "url": cover_url } }),
        );
    }
    if let Some(mp2) = games.get(1) {
        dynamic.push(serde_json::json!({ "type": 1, "name": "most_played2", "value": mp2.title }));
    }
    if let Some(mp3) = games.get(2) {
        dynamic.push(serde_json::json!({ "type": 1, "name": "most_played3", "value": mp3.title }));
    }

    let recent_played = games
        .iter()
        .filter(|g| g.last_played.is_some())
        .max_by_key(|g| g.last_played);

    if let Some(recent) = recent_played {
        dynamic
            .push(serde_json::json!({ "type": 1, "name": "recent_played", "value": recent.title }));
        if let Some(lp) = recent.last_played {
            let pretty_date = lp.format("%B %d, %Y").to_string();
            dynamic.push(
                serde_json::json!({ "type": 1, "name": "last_date_played", "value": pretty_date }),
            );
        }
    }

    let payload = serde_json::json!({
        "username": username,
        "data": {
            "dynamic": dynamic
        }
    });

    let url = format!(
        "https://discord.com/api/v9/applications/{}/users/{}/identities/{}/profile",
        client_id, discord_id, user_id
    );
    let client = reqwest::Client::new();
    let res = client
        .patch(&url)
        .header("Authorization", format!("Bot {}", bot_token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Discord API error: {}", text));
    }

    Ok(())
}

pub fn start_discord_widget_task() {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(12 * 60 * 60));

        loop {
            interval.tick().await;

            if SQL.get().is_none() {
                continue;
            }

            let rows = sqlx::query!("SELECT user_id FROM discord_connections")
                .fetch_all(&*SQL)
                .await;

            if let Ok(connections) = rows {
                for conn in connections {
                    if let Err(e) = sync_user_discord_widget(conn.user_id).await {
                        error!(
                            "Background discord widget sync failed for user {}: {}",
                            conn.user_id, e
                        );
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    });
}

#[post("/api/v1/users/@me/sync-discord-widget")]
pub async fn sync_widget_endpoint(auth: AuthenticatedUser) -> V1ApiResponseType<()> {
    sync_user_discord_widget(auth.id.0).await.map_err(|e| {
        error!("Manual discord widget sync failed: {}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(()))
}
