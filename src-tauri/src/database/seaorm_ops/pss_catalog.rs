use crate::{
    database::models::{PssAthlete, PssMatch, PssMatchAthlete},
    entity::{athlete, event, match_participant, matches, tournament},
};
use chrono::{TimeZone, Utc};
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use super::pss;

fn map_match(
    model: matches::Model,
    related_tournament: Option<tournament::Model>,
) -> PssMatch {
    PssMatch {
        id: Some(model.id as i64),
        uuid: Some(model.uuid.clone()),
        tournament_id: related_tournament.map(|t| t.uuid),
        tournament_day_id: None,
        match_id: model.match_code.clone(),
        match_number: model.match_number.clone(),
        category: model.category.clone(),
        weight_class: model.weight_class_code.clone(),
        division: model.division_code.clone(),
        total_rounds: model.total_rounds.unwrap_or(3),
        round_duration: model.round_duration,
        countdown_type: model.countdown_type.clone(),
        format_type: model.format_type,
        creation_mode: model
            .creation_mode
            .clone()
            .unwrap_or_else(|| "Automatic".to_string()),
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
        created: Some(model.created_at.timestamp()),
        updated: Some(model.updated_at.timestamp()),
    }
}

fn map_athlete(model: athlete::Model) -> PssAthlete {
    let now = Utc::now();
    let short_name = model
        .short_name
        .clone()
        .or_else(|| model.display_name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let athlete_code = model
        .pss_code
        .clone()
        .unwrap_or_else(|| model.uuid.clone());

    let created_at = Utc::from_utc_datetime(&model.created_at);
    let updated_at = Utc::from_utc_datetime(&model.updated_at);

    PssAthlete {
        id: Some(model.id as i64),
        athlete_code,
        short_name,
        long_name: model.display_name.clone(),
        country_code: model.country_code.clone(),
        flag_id: model.flag_id.map(|v| v as i64),
        created_at,
        updated_at,
    }
}

fn map_match_athlete(model: match_participant::Model) -> Result<PssMatchAthlete, DbErr> {
    let created_at = Utc::from_utc_datetime(&model.created_at);

    Ok(PssMatchAthlete {
        id: Some(model.id as i64),
        match_id: model.match_id as i64,
        athlete_id: model.athlete_id as i64,
        athlete_position: pss::side_to_position(&model.side),
        bg_color: model.bg_color.clone(),
        fg_color: model.fg_color.clone(),
        created_at,
    })
}

pub async fn get_match_by_id(
    conn: &DatabaseConnection,
    id: i64,
) -> Result<Option<PssMatch>, DbErr> {
    let record = matches::Entity::find_by_id(id as i32)
        .find_also_related(tournament::Entity)
        .one(conn)
        .await?;
    Ok(record.map(|(m, t)| map_match(m, t)))
}

pub async fn get_matches(
    conn: &DatabaseConnection,
    limit: Option<i64>,
) -> Result<Vec<PssMatch>, DbErr> {
    let mut query = matches::Entity::find()
        .order_by_desc(matches::Column::CreatedAt)
        .find_also_related(tournament::Entity);
    if let Some(limit) = limit {
        query = query.limit(limit as u64);
    }
    let records = query.all(conn).await?;
    Ok(records.into_iter().map(|(m, t)| map_match(m, t)).collect())
}

pub async fn insert_match(
    conn: &DatabaseConnection,
    match_data: &PssMatch,
) -> Result<i64, DbErr> {
    let uuid = match_data
        .uuid
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let tournament_id = if let Some(ref tournament_uuid) = match_data.tournament_id {
        tournament::Entity::find()
            .filter(tournament::Column::Uuid.eq(tournament_uuid.clone()))
            .one(conn)
            .await?
            .map(|t| t.id)
    } else {
        None
    };

    let active = matches::ActiveModel {
        uuid: Set(uuid),
        match_code: Set(match_data.match_id.clone()),
        match_number: Set(match_data.match_number.clone()),
        category: Set(match_data.category.clone()),
        division_code: Set(match_data.division.clone()),
        weight_class_code: Set(match_data.weight_class.clone()),
        total_rounds: Set(Some(match_data.total_rounds)),
        round_duration: Set(match_data.round_duration),
        countdown_type: Set(match_data.countdown_type.clone()),
        format_type: Set(match_data.format_type),
        creation_mode: Set(Some(match_data.creation_mode.clone())),
        created_at: Set(match_data.created_at.naive_utc()),
        updated_at: Set(match_data.updated_at.naive_utc()),
        tournament_id: Set(tournament_id),
        ..Default::default()
    };

    let inserted = active.insert(conn).await?;
    Ok(inserted.id as i64)
}

pub async fn rename_match_id(
    conn: &DatabaseConnection,
    match_db_id: i64,
    new_match_id: &str,
) -> Result<(), DbErr> {
    if let Some(model) = matches::Entity::find_by_id(match_db_id as i32)
        .one(conn)
        .await?
    {
        let mut active: matches::ActiveModel = model.into();
        active.match_code = Set(new_match_id.to_string());
        active.updated_at = Set(Utc::now().naive_utc());
        active.update(conn).await?;
    }
    Ok(())
}

pub async fn set_match_tournament_context(
    conn: &DatabaseConnection,
    match_db_id: i64,
    tournament_id: Option<i64>,
) -> Result<(), DbErr> {
    if let Some(model) = matches::Entity::find_by_id(match_db_id as i32)
        .one(conn)
        .await?
    {
        let mut active: matches::ActiveModel = model.into();
        active.tournament_id = Set(tournament_id.map(|v| v as i32));
        active.updated_at = Set(Utc::now().naive_utc());
        active.update(conn).await?;
    }
    Ok(())
}

pub async fn reassign_events_between_matches(
    conn: &DatabaseConnection,
    from_match_id: i64,
    to_match_id: i64,
) -> Result<usize, DbErr> {
    let result = event::Entity::update_many()
        .filter(event::Column::MatchId.eq(from_match_id as i32))
        .col_expr(event::Column::MatchId, Expr::value(to_match_id as i32))
        .exec(conn)
        .await?;
    Ok(result.rows_affected as usize)
}

pub async fn get_match_athletes(
    conn: &DatabaseConnection,
    match_id: i64,
) -> Result<Vec<(PssMatchAthlete, PssAthlete)>, DbErr> {
    let participants = match_participant::Entity::find()
        .filter(match_participant::Column::MatchId.eq(match_id as i32))
        .order_by_asc(match_participant::Column::Side)
        .find_also_related(athlete::Entity)
        .all(conn)
        .await?;

    let mut result = Vec::with_capacity(participants.len());
    for (participant, athlete_model) in participants {
        if let Some(athlete_model) = athlete_model {
            let match_athlete = map_match_athlete(participant)?;
            let athlete = map_athlete(athlete_model);
            result.push((match_athlete, athlete));
        }
    }
    Ok(result)
}

pub async fn insert_match_athlete(
    conn: &DatabaseConnection,
    match_athlete: &PssMatchAthlete,
) -> Result<i64, DbErr> {
    let side = pss::position_to_side(match_athlete.athlete_position)?;
    let active = match_participant::ActiveModel {
        match_id: Set(match_athlete.match_id as i32),
        athlete_id: Set(match_athlete.athlete_id as i32),
        side: Set(side),
        bg_color: Set(match_athlete.bg_color.clone()),
        fg_color: Set(match_athlete.fg_color.clone()),
        created_at: Set(match_athlete.created_at.naive_utc()),
        ..Default::default()
    };
    let inserted = active.insert(conn).await?;
    Ok(inserted.id as i64)
}
