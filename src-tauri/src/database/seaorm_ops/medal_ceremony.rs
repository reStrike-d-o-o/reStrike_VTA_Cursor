use crate::database::models::{
    MedalCeremony, MedalCeremonyAthleteOption, MedalCeremonyDetail, MedalCeremonyDivision,
    MedalCeremonyDivisionDetail, MedalCeremonyDivisionOption, MedalCeremonyMedalist,
};
use crate::entity::{medal_ceremony, medal_ceremony_division, medal_ceremony_medalist};
use chrono::{DateTime, TimeZone, Utc};
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait,
    DatabaseConnection, DbBackend, DbErr, EntityTrait, FromQueryResult, QueryFilter, QueryOrder,
    Statement, TransactionError, TransactionTrait,
};
use uuid::Uuid;

fn parse_optional_datetime(raw: Option<String>) -> Option<DateTime<Utc>> {
    raw.and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn format_optional_datetime(value: Option<DateTime<Utc>>) -> Option<String> {
    value.map(|dt| dt.to_rfc3339())
}

fn resolve_id(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        trimmed.to_string()
    }
}

fn map_transaction_error(err: TransactionError<DbErr>) -> DbErr {
    match err {
        TransactionError::Connection(inner) | TransactionError::Transaction(inner) => inner,
    }
}

fn map_ceremony_model(model: medal_ceremony::Model) -> MedalCeremony {
    MedalCeremony {
        id: model.id,
        tournament_id: model.tournament_id.map(|id| id as i64),
        name: model.name,
        background_path: model.background_path,
        break_path: model.break_path,
        animation_duration: i64::from(model.animation_duration),
        animation_speed: model.animation_speed,
        photo_time: i64::from(model.photo_time),
        prepared_at: parse_optional_datetime(model.prepared_at),
        prepared_version: i64::from(model.prepared_version),
        show_external: model.show_external,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

fn map_division_model(model: medal_ceremony_division::Model) -> MedalCeremonyDivision {
    MedalCeremonyDivision {
        id: model.id,
        ceremony_id: model.ceremony_id,
        division_id: model.division_id.map(|id| id as i64),
        division_name: model.division_name,
        order_index: i64::from(model.order_index),
        played_at: parse_optional_datetime(model.played_at),
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

fn map_medalist_model(model: medal_ceremony_medalist::Model) -> MedalCeremonyMedalist {
    MedalCeremonyMedalist {
        id: model.id,
        division_entry_id: model.division_entry_id,
        medal_type: model.medal_type,
        medal_rank: i64::from(model.medal_rank),
        athlete_id: model.athlete_id.map(|id| id as i64),
        athlete_name: model.athlete_name,
        athlete_short_name: model.athlete_short_name,
        ioc_code: model.ioc_code,
        flag_asset: model.flag_asset,
        anthem_asset: model.anthem_asset,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

#[derive(Debug, FromQueryResult)]
struct DivisionOptionRow {
    name: String,
    category: Option<String>,
    gender: Option<String>,
    weight_class: Option<String>,
}

#[derive(Debug, FromQueryResult)]
struct AthleteOptionRow {
    athlete_id: i64,
    full_name: Option<String>,
    short_name: Option<String>,
    country_code: Option<String>,
    ioc_code: Option<String>,
    athlete_code: Option<String>,
}

/// List distinct ceremony division options derived from matches and overlay categories.
pub async fn list_division_options(
    conn: &DatabaseConnection,
) -> Result<Vec<MedalCeremonyDivisionOption>, DbErr> {
    let sql = r#"
        SELECT DISTINCT name, category, gender, weight_class
        FROM (
            SELECT
                TRIM("division_code") AS name,
                TRIM("category") AS category,
                NULL AS gender,
                TRIM("weight_class_code") AS weight_class
            FROM "match"
            WHERE division_code IS NOT NULL AND TRIM(division_code) <> ''
            UNION
            SELECT
                TRIM(division) AS name,
                TRIM(age_group) AS category,
                TRIM(gender) AS gender,
                TRIM(weight_class) AS weight_class
            FROM overlay_category
            WHERE division IS NOT NULL AND TRIM(division) <> ''
        )
        WHERE name IS NOT NULL AND name <> ''
        ORDER BY LOWER(name), LOWER(IFNULL(category, '')), LOWER(IFNULL(gender, '')), LOWER(IFNULL(weight_class, ''))
    "#;

    let rows = DivisionOptionRow::find_by_statement(Statement::from_string(
        DbBackend::Sqlite,
        sql.to_string(),
    ))
    .all(conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| MedalCeremonyDivisionOption {
            name: row.name,
            category: row.category,
            gender: row.gender,
            weight_class: row.weight_class,
        })
        .collect())
}

/// List athletes associated with matches in the specified division.
pub async fn list_athlete_options(
    conn: &DatabaseConnection,
    division_name: &str,
) -> Result<Vec<MedalCeremonyAthleteOption>, DbErr> {
    let sql = r#"
        SELECT DISTINCT
            a.id AS athlete_id,
            COALESCE(
                NULLIF(TRIM(a.display_name), ''),
                NULLIF(TRIM(a.short_name), ''),
                NULLIF(TRIM(a.first_name || ' ' || a.last_name), '')
            ) AS full_name,
            a.short_name,
            a.country_code,
            a.ioc_code,
            a.pss_code AS athlete_code
        FROM match_participant mp
        INNER JOIN "match" m ON mp.match_id = m.id
        INNER JOIN athlete a ON mp.athlete_id = a.id
        WHERE m.division_code IS NOT NULL
          AND TRIM(m.division_code) <> ''
          AND LOWER(TRIM(m.division_code)) = LOWER(TRIM(?))
        ORDER BY LOWER(full_name), LOWER(IFNULL(a.short_name, ''))
    "#;

    let stmt = Statement::from_sql_and_values(DbBackend::Sqlite, sql, [division_name.into()]);

    let rows = AthleteOptionRow::find_by_statement(stmt).all(conn).await?;

    Ok(rows
        .into_iter()
        .map(|row| MedalCeremonyAthleteOption {
            id: row.athlete_id,
            full_name: row
                .full_name
                .unwrap_or_else(|| "Unknown athlete".to_string()),
            short_name: row.short_name,
            country_code: row.country_code,
            ioc_code: row.ioc_code,
            athlete_code: row.athlete_code,
        })
        .collect())
}

/// Fetch every medal ceremony ordered by creation timestamp.
pub async fn list_ceremonies(conn: &DatabaseConnection) -> Result<Vec<MedalCeremony>, DbErr> {
    let records = medal_ceremony::Entity::find()
        .order_by_desc(medal_ceremony::Column::CreatedAt)
        .order_by_asc(medal_ceremony::Column::Name)
        .all(conn)
        .await?;

    Ok(records.into_iter().map(map_ceremony_model).collect())
}

/// Retrieve a single medal ceremony with nested divisions and medalists.
pub async fn get_ceremony_detail(
    conn: &DatabaseConnection,
    ceremony_id: &str,
) -> Result<Option<MedalCeremonyDetail>, DbErr> {
    let Some(ceremony) = medal_ceremony::Entity::find_by_id(ceremony_id.to_string())
        .one(conn)
        .await?
    else {
        return Ok(None);
    };

    let divisions = fetch_divisions_with_medalists(conn, ceremony_id).await?;

    Ok(Some(MedalCeremonyDetail {
        ceremony: map_ceremony_model(ceremony),
        divisions,
    }))
}

async fn fetch_divisions_with_medalists<C>(
    conn: &C,
    ceremony_id: &str,
) -> Result<Vec<MedalCeremonyDivisionDetail>, DbErr>
where
    C: ConnectionTrait,
{
    let division_rows = medal_ceremony_division::Entity::find()
        .filter(medal_ceremony_division::Column::CeremonyId.eq(ceremony_id.to_string()))
        .order_by_asc(medal_ceremony_division::Column::OrderIndex)
        .order_by_asc(medal_ceremony_division::Column::CreatedAt)
        .find_with_related(medal_ceremony_medalist::Entity)
        .all(conn)
        .await?;

    let mut result = Vec::with_capacity(division_rows.len());
    for (division_model, medalist_models) in division_rows {
        let division = map_division_model(division_model);
        let medalists = medalist_models
            .into_iter()
            .map(map_medalist_model)
            .collect();
        result.push(MedalCeremonyDivisionDetail {
            division,
            medalists,
        });
    }
    Ok(result)
}

/// Insert or update a medal ceremony with its divisions and medalists.
pub async fn upsert_ceremony(
    conn: &DatabaseConnection,
    payload: &MedalCeremonyDetail,
) -> Result<String, DbErr> {
    let ceremony_id = resolve_id(&payload.ceremony.id);

    conn.transaction(|txn| {
        let payload = payload.clone();
        let ceremony_id = ceremony_id.clone();
        Box::pin(async move {
            let now = Utc::now().naive_utc();

            if let Some(existing) = medal_ceremony::Entity::find_by_id(ceremony_id.clone())
                .one(txn)
                .await?
            {
                let mut active: medal_ceremony::ActiveModel = existing.into();
                active.tournament_id = Set(payload.ceremony.tournament_id.map(|id| id as i32));
                active.name = Set(payload.ceremony.name.clone());
                active.background_path = Set(payload.ceremony.background_path.clone());
                active.break_path = Set(payload.ceremony.break_path.clone());
                active.animation_duration = Set(payload.ceremony.animation_duration as i32);
                active.animation_speed = Set(payload.ceremony.animation_speed);
                active.photo_time = Set(payload.ceremony.photo_time as i32);
                active.prepared_at = Set(None);
                active.prepared_version = Set(payload.ceremony.prepared_version as i32);
                active.show_external = Set(payload.ceremony.show_external);
                active.updated_at = Set(now);
                active.update(txn).await?;
            } else {
                let active = medal_ceremony::ActiveModel {
                    id: Set(ceremony_id.clone()),
                    tournament_id: Set(payload.ceremony.tournament_id.map(|id| id as i32)),
                    name: Set(payload.ceremony.name.clone()),
                    background_path: Set(payload.ceremony.background_path.clone()),
                    break_path: Set(payload.ceremony.break_path.clone()),
                    animation_duration: Set(payload.ceremony.animation_duration as i32),
                    animation_speed: Set(payload.ceremony.animation_speed),
                    photo_time: Set(payload.ceremony.photo_time as i32),
                    prepared_at: Set(None),
                    prepared_version: Set(payload.ceremony.prepared_version as i32),
                    show_external: Set(payload.ceremony.show_external),
                    created_at: Set(payload.ceremony.created_at.naive_utc()),
                    updated_at: Set(now),
                };
                active.insert(txn).await?;
            }

            medal_ceremony_division::Entity::delete_many()
                .filter(medal_ceremony_division::Column::CeremonyId.eq(ceremony_id.clone()))
                .exec(txn)
                .await?;

            for (idx, detail) in payload.divisions.iter().enumerate() {
                let division_id = resolve_id(&detail.division.id);
                let order_index = if detail.division.order_index > 0 {
                    detail.division.order_index as i32
                } else {
                    (idx as i32) + 1
                };

                let division_active = medal_ceremony_division::ActiveModel {
                    id: Set(division_id.clone()),
                    ceremony_id: Set(ceremony_id.clone()),
                    division_id: Set(detail.division.division_id.map(|id| id as i32)),
                    division_name: Set(detail.division.division_name.clone()),
                    order_index: Set(order_index),
                    played_at: Set(format_optional_datetime(detail.division.played_at)),
                    created_at: Set(detail.division.created_at.naive_utc()),
                    updated_at: Set(now),
                };
                division_active.insert(txn).await?;

                for medalist in &detail.medalists {
                    let medalist_id = resolve_id(&medalist.id);
                    let medalist_active = medal_ceremony_medalist::ActiveModel {
                        id: Set(medalist_id),
                        division_entry_id: Set(division_id.clone()),
                        medal_type: Set(medalist.medal_type.clone()),
                        medal_rank: Set(medalist.medal_rank as i32),
                        athlete_id: Set(medalist.athlete_id.map(|id| id as i32)),
                        athlete_name: Set(medalist.athlete_name.clone()),
                        athlete_short_name: Set(medalist.athlete_short_name.clone()),
                        ioc_code: Set(medalist.ioc_code.clone()),
                        flag_asset: Set(medalist.flag_asset.clone()),
                        anthem_asset: Set(medalist.anthem_asset.clone()),
                        created_at: Set(medalist.created_at.naive_utc()),
                        updated_at: Set(now),
                    };
                    medalist_active.insert(txn).await?;
                }
            }

            Ok(ceremony_id)
        })
    })
    .await
    .map_err(map_transaction_error)
}

/// Delete a medal ceremony by identifier.
pub async fn delete_ceremony(conn: &DatabaseConnection, ceremony_id: &str) -> Result<(), DbErr> {
    medal_ceremony::Entity::delete_by_id(ceremony_id.to_string())
        .exec(conn)
        .await?
        .rows_affected;
    Ok(())
}

/// Prepare the ceremony playlist by advancing the prepared version and returning divisions.
pub async fn prepare_playlist(
    conn: &DatabaseConnection,
    ceremony_id: &str,
) -> Result<Vec<MedalCeremonyDivisionDetail>, DbErr> {
    conn.transaction(|txn| {
        let ceremony_id = ceremony_id.to_string();
        Box::pin(async move {
            let Some(model) = medal_ceremony::Entity::find_by_id(ceremony_id.clone())
                .one(txn)
                .await?
            else {
                return Err(DbErr::RecordNotFound(format!(
                    "Medal ceremony {ceremony_id} not found"
                )));
            };

            let new_version = model.prepared_version + 1;
            let mut active: medal_ceremony::ActiveModel = model.into();
            let now = Utc::now();
            active.prepared_at = Set(Some(now.to_rfc3339()));
            active.prepared_version = Set(new_version);
            active.updated_at = Set(now.naive_utc());
            active.update(txn).await?;

            let divisions = fetch_divisions_with_medalists(txn, &ceremony_id).await?;
            Ok(divisions)
        })
    })
    .await
    .map_err(map_transaction_error)
}

/// Mark a division as played within a ceremony.
pub async fn mark_division_played(
    conn: &DatabaseConnection,
    division_id: &str,
) -> Result<(), DbErr> {
    conn.transaction(|txn| {
        let division_id = division_id.to_string();
        Box::pin(async move {
            let Some(division) = medal_ceremony_division::Entity::find_by_id(division_id.clone())
                .one(txn)
                .await?
            else {
                return Err(DbErr::RecordNotFound(format!(
                    "Medal ceremony division {division_id} not found"
                )));
            };

            let mut division_active: medal_ceremony_division::ActiveModel = division.into();
            let now = Utc::now();
            division_active.played_at = Set(Some(now.to_rfc3339()));
            division_active.updated_at = Set(now.naive_utc());
            let ceremony_id = division_active.ceremony_id.clone().unwrap();
            division_active.update(txn).await?;

            if let Some(ceremony) = medal_ceremony::Entity::find_by_id(ceremony_id.clone())
                .one(txn)
                .await?
            {
                let mut ceremony_active: medal_ceremony::ActiveModel = ceremony.into();
                ceremony_active.updated_at = Set(now.naive_utc());
                ceremony_active.update(txn).await?;
            }

            Ok(())
        })
    })
    .await
    .map_err(map_transaction_error)
}

/// Reset ceremony playback state.
pub async fn reset_playback(conn: &DatabaseConnection, ceremony_id: &str) -> Result<(), DbErr> {
    let now = Utc::now();
    medal_ceremony_division::Entity::update_many()
        .filter(medal_ceremony_division::Column::CeremonyId.eq(ceremony_id.to_string()))
        .col_expr(
            medal_ceremony_division::Column::PlayedAt,
            Expr::value(Option::<String>::None),
        )
        .col_expr(
            medal_ceremony_division::Column::UpdatedAt,
            Expr::value(now.naive_utc()),
        )
        .exec(conn)
        .await?;

    if let Some(model) = medal_ceremony::Entity::find_by_id(ceremony_id.to_string())
        .one(conn)
        .await?
    {
        let mut active: medal_ceremony::ActiveModel = model.into();
        active.prepared_at = Set(None);
        active.updated_at = Set(now.naive_utc());
        active.update(conn).await?;
    }

    Ok(())
}

/// Toggle the external display flag for a ceremony.
pub async fn update_show_external(
    conn: &DatabaseConnection,
    ceremony_id: &str,
    enabled: bool,
) -> Result<(), DbErr> {
    if let Some(model) = medal_ceremony::Entity::find_by_id(ceremony_id.to_string())
        .one(conn)
        .await?
    {
        let mut active: medal_ceremony::ActiveModel = model.into();
        let now = Utc::now().naive_utc();
        active.show_external = Set(enabled);
        active.updated_at = Set(now);
        active.update(conn).await?;
    }

    Ok(())
}
