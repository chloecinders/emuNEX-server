use rocket::{
    form::{Form, FromForm},
    fs::TempFile,
    get, post,
};
use serde::Serialize;
use tracing::error;

use crate::{
    SQL,
    routes::api::{
        V1ApiError, V1ApiResponse, V1ApiResponseType,
        api_response::V1ApiResponseTrait,
        v1::guards::{AuthenticatedUser, UserRole},
    },
};

#[derive(Serialize, sqlx::FromRow)]
pub struct NoIntroGame {
    pub id: String,
    pub dat_name: String,
    pub name: String,
    pub serial: Option<String>,
    pub md5: Option<String>,
    pub crc: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<i64>,
    pub clone_of: Option<String>,
    pub status: Option<String>,
}

impl V1ApiResponseTrait for Vec<NoIntroGame> {}

#[derive(Serialize, sqlx::FromRow)]
pub struct V1NoIntroDat {
    pub name: String,
    pub notes: Option<String>,
}

impl V1ApiResponseTrait for V1NoIntroDat {}
impl V1ApiResponseTrait for Vec<V1NoIntroDat> {}

#[derive(Serialize)]
pub struct ImportResult {
    pub imported: usize,
}
impl V1ApiResponseTrait for ImportResult {}

#[derive(FromForm)]
pub struct V1NoIntroImportForm<'r> {
    dat_file: TempFile<'r>,
}

#[post(
    "/api/v1/nointro/import",
    format = "multipart/form-data",
    data = "<data>"
)]
pub async fn import_nointro(
    data: Form<V1NoIntroImportForm<'_>>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<ImportResult> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    let bytes = tokio::fs::read(data.dat_file.path().unwrap())
        .await
        .map_err(|e| {
            error!("Failed to read no-intro dat file: {:?}", e);
            V1ApiError::DatabaseError
        })?;

    let content = String::from_utf8_lossy(&bytes);
    let games = parse_nointro_xml(&content).map_err(|e| {
        error!("Failed to parse no-intro XML: {:?}", e);
        V1ApiError::BadRequest
    })?;

    let count = games.len();

    for game in games {
        sqlx::query!(
            r#"INSERT INTO nointro_games (id, dat_name, name, serial, md5, crc, sha1, size, clone_of, status)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               ON CONFLICT (id) DO UPDATE SET
                   dat_name = EXCLUDED.dat_name,
                   name = EXCLUDED.name,
                   serial = EXCLUDED.serial,
                   md5 = EXCLUDED.md5,
                   crc = EXCLUDED.crc,
                   sha1 = EXCLUDED.sha1,
                   size = EXCLUDED.size,
                   clone_of = EXCLUDED.clone_of,
                   status = EXCLUDED.status"#,
            game.id,
            game.dat_name,
            game.name,
            game.serial,
            game.md5,
            game.crc,
            game.sha1,
            game.size,
            game.clone_of,
            game.status
        )
        .execute(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to upsert nointro game '{}': {:?}", game.id, e);
            V1ApiError::DatabaseError
        })?;
    }

    Ok(V1ApiResponse(ImportResult { imported: count }))
}

#[get("/api/v1/nointro/dats")]
pub async fn get_nointro_dats(_user: AuthenticatedUser) -> V1ApiResponseType<Vec<V1NoIntroDat>> {
    let rows = sqlx::query_as!(
        V1NoIntroDat,
        "SELECT DISTINCT g.dat_name as name, d.notes 
         FROM nointro_games g
         LEFT JOIN nointro_dats d ON g.dat_name = d.name
         ORDER BY g.dat_name ASC"
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to fetch nointro dats: {:?}", e);
        V1ApiError::DatabaseError
    })?;

    Ok(V1ApiResponse(rows))
}

#[derive(serde::Deserialize)]
pub struct V1DatNotesUpdate {
    pub notes: String,
}

#[post("/api/v1/nointro/dats/<name>/notes", format = "json", data = "<data>")]
pub async fn update_dat_notes(
    name: String,
    data: rocket::serde::json::Json<V1DatNotesUpdate>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    sqlx::query!(
        "INSERT INTO nointro_dats (name, notes) VALUES ($1, $2)
         ON CONFLICT (name) DO UPDATE SET notes = EXCLUDED.notes",
        name,
        data.notes
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to update notes for dat {}: {:?}", name, e);
        V1ApiError::DatabaseError
    })?;

    Ok(V1ApiResponse(()))
}

#[get("/api/v1/nointro/search?<dat>&<query>&<limit>")]
pub async fn search_nointro(
    dat: Option<String>,
    query: Option<String>,
    limit: Option<i64>,
    _user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<NoIntroGame>> {
    let final_limit = limit.unwrap_or(50).min(200);
    let q = query.unwrap_or_default();

    let results = if q.is_empty() {
        sqlx::query_as!(
            NoIntroGame,
            "SELECT id, dat_name, name, serial, md5, crc, sha1, size, clone_of, status
             FROM nointro_games
             WHERE (dat_name = $1 OR $1 IS NULL)
             ORDER BY name ASC
             LIMIT $2",
            dat,
            final_limit
        )
        .fetch_all(&*SQL)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::DatabaseError
        })?
    } else {
        sqlx::query_as!(
            NoIntroGame,
            r#"SELECT id, dat_name, name, serial, md5, crc, sha1, size, clone_of, status
               FROM nointro_games
               WHERE (dat_name = $1 OR $1 IS NULL)
                 AND (
                     name ILIKE '%' || $2 || '%'
                     OR serial ILIKE $2 || '%'
                 )
               ORDER BY
                   CASE WHEN serial ILIKE $2 THEN 0 ELSE 1 END,
                   name ASC
               LIMIT $3"#,
            dat,
            q,
            final_limit
        )
        .fetch_all(&*SQL)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::DatabaseError
        })?
    };

    Ok(V1ApiResponse(results))
}

struct ParsedGame {
    id: String,
    dat_name: String,
    name: String,
    serial: Option<String>,
    md5: Option<String>,
    crc: Option<String>,
    sha1: Option<String>,
    size: Option<i64>,
    clone_of: Option<String>,
    status: Option<String>,
}

fn parse_nointro_xml(xml: &str) -> Result<Vec<ParsedGame>, quick_xml::Error> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut games = Vec::new();
    let mut dat_name = String::new();
    let mut current_game_name: Option<String> = None;
    let mut current_game_id: Option<String> = None;
    let mut current_clone_of: Option<String> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"game" => {
                    current_game_name = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"name")
                        .and_then(|a| String::from_utf8(a.value.to_vec()).ok());
                    current_game_id = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"id")
                        .and_then(|a| String::from_utf8(a.value.to_vec()).ok());
                    current_clone_of = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"cloneofid")
                        .and_then(|a| String::from_utf8(a.value.to_vec()).ok());
                }
                b"rom" => {
                    if let (Some(gname), Some(gid)) =
                        (current_game_name.as_ref(), current_game_id.as_ref())
                    {
                        let attrs: std::collections::HashMap<Vec<u8>, String> = e
                            .attributes()
                            .filter_map(|a| a.ok())
                            .map(|a| {
                                (
                                    a.key.as_ref().to_vec(),
                                    String::from_utf8(a.value.to_vec()).unwrap_or_default(),
                                )
                            })
                            .collect();

                        games.push(ParsedGame {
                            id: gid.clone(),
                            dat_name: dat_name.clone(),
                            name: gname.clone(),
                            serial: attrs.get(b"serial".as_ref()).cloned(),
                            md5: attrs.get(b"md5".as_ref()).cloned(),
                            crc: attrs.get(b"crc".as_ref()).cloned(),
                            sha1: attrs.get(b"sha1".as_ref()).cloned(),
                            size: attrs.get(b"size".as_ref()).and_then(|s| s.parse().ok()),
                            clone_of: current_clone_of.clone(),
                            status: attrs.get(b"status".as_ref()).cloned(),
                        });
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(e)) => {
                if e.name().as_ref() == b"rom" {
                    if let (Some(gname), Some(gid)) =
                        (current_game_name.as_ref(), current_game_id.as_ref())
                    {
                        let attrs: std::collections::HashMap<Vec<u8>, String> = e
                            .attributes()
                            .filter_map(|a| a.ok())
                            .map(|a| {
                                (
                                    a.key.as_ref().to_vec(),
                                    String::from_utf8(a.value.to_vec()).unwrap_or_default(),
                                )
                            })
                            .collect();

                        games.push(ParsedGame {
                            id: gid.clone(),
                            dat_name: dat_name.clone(),
                            name: gname.clone(),
                            serial: attrs.get(b"serial".as_ref()).cloned(),
                            md5: attrs.get(b"md5".as_ref()).cloned(),
                            crc: attrs.get(b"crc".as_ref()).cloned(),
                            sha1: attrs.get(b"sha1".as_ref()).cloned(),
                            size: attrs.get(b"size".as_ref()).and_then(|s| s.parse().ok()),
                            clone_of: current_clone_of.clone(),
                            status: attrs.get(b"status".as_ref()).cloned(),
                        });
                    }
                }
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"game" {
                    current_game_name = None;
                    current_game_id = None;
                    current_clone_of = None;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e),
            _ => {}
        }
    }

    let mut reader2 = Reader::from_str(xml);
    reader2.config_mut().trim_text(true);
    let mut in_header2 = false;
    let mut next_is_name = false;

    loop {
        match reader2.read_event() {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"header" {
                    in_header2 = true;
                } else if in_header2 && e.name().as_ref() == b"name" {
                    next_is_name = true;
                }
            }
            Ok(Event::Text(e)) if next_is_name => {
                let s = String::from_utf8_lossy(e.as_ref()).to_string();
                dat_name = quick_xml::escape::unescape(&s)
                    .map(|s| s.to_string())
                    .unwrap_or(s);
                next_is_name = false;
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"header" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e),
            _ => {}
        }
    }

    for game in &mut games {
        game.dat_name = dat_name.clone();
    }

    Ok(games)
}
