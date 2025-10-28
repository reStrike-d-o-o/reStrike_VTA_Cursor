use crate::{
    core::app::App,
    database::{
        models::{
            MedalCeremony, MedalCeremonyDetail, MedalCeremonyDivision, MedalCeremonyDivisionDetail,
            MedalCeremonyMedalist, OvrAnthemAsset, OvrFlagAnimationAsset,
        },
        operations::{
            MedalCeremonyAthleteOption, MedalCeremonyDivisionOption, MedalCeremonyOperations,
        },
    },
};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Error as TauriError, State};

fn map_db_error(context: &str, err: impl std::fmt::Display) -> TauriError {
    TauriError::from(anyhow!("{}: {}", context, err))
}

fn parse_optional_datetime(value: &Option<String>) -> Option<DateTime<Utc>> {
    value
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn datetime_to_string(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

fn optional_datetime_to_string(value: &Option<DateTime<Utc>>) -> Option<String> {
    value.as_ref().map(datetime_to_string)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedalCeremonyDto {
    pub id: Option<String>,
    pub tournament_id: Option<i64>,
    pub name: String,
    pub background_path: Option<String>,
    pub break_path: Option<String>,
    pub animation_duration: i64,
    pub animation_speed: f64,
    pub photo_time: i64,
    pub prepared_at: Option<String>,
    pub prepared_version: i64,
    pub show_external: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedalCeremonyMedalistDto {
    pub id: Option<String>,
    pub medal_type: String,
    pub medal_rank: i64,
    pub athlete_id: Option<i64>,
    pub athlete_name: String,
    pub athlete_short_name: Option<String>,
    pub ioc_code: Option<String>,
    pub flag_asset: Option<String>,
    pub anthem_asset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedalCeremonyDivisionDto {
    pub id: Option<String>,
    pub division_id: Option<i64>,
    pub division_name: String,
    pub order_index: i64,
    pub played_at: Option<String>,
    pub medalists: Vec<MedalCeremonyMedalistDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedalCeremonyDetailDto {
    pub ceremony: MedalCeremonyDto,
    pub divisions: Vec<MedalCeremonyDivisionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedalCeremonySummaryDto {
    pub id: String,
    pub tournament_id: Option<i64>,
    pub name: String,
    pub prepared_at: Option<String>,
    pub prepared_version: i64,
    pub show_external: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedalCeremonyDivisionOptionDto {
    pub name: String,
    pub category: Option<String>,
    pub gender: Option<String>,
    pub weight_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedalCeremonyAthleteOptionDto {
    pub id: i64,
    pub full_name: String,
    pub short_name: Option<String>,
    pub country_code: Option<String>,
    pub ioc_code: Option<String>,
    pub athlete_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagAnimationAssetDto {
    pub id: Option<String>,
    pub ioc_code: String,
    pub file_name: String,
    pub file_path: String,
    pub display_name: Option<String>,
    pub duration_ms: Option<i64>,
    pub is_default: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthemAssetDto {
    pub id: Option<String>,
    pub ioc_code: String,
    pub file_name: String,
    pub file_path: String,
    pub display_name: Option<String>,
    pub duration_ms: Option<i64>,
    pub is_default: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn ceremony_to_summary(model: &MedalCeremony) -> MedalCeremonySummaryDto {
    MedalCeremonySummaryDto {
        id: model.id.clone(),
        tournament_id: model.tournament_id,
        name: model.name.clone(),
        prepared_at: optional_datetime_to_string(&model.prepared_at),
        prepared_version: model.prepared_version,
        show_external: model.show_external,
        created_at: datetime_to_string(&model.created_at),
        updated_at: datetime_to_string(&model.updated_at),
    }
}

fn division_option_to_dto(model: &MedalCeremonyDivisionOption) -> MedalCeremonyDivisionOptionDto {
    MedalCeremonyDivisionOptionDto {
        name: model.name.clone(),
        category: model.category.clone(),
        gender: model.gender.clone(),
        weight_class: model.weight_class.clone(),
    }
}

fn athlete_option_to_dto(model: &MedalCeremonyAthleteOption) -> MedalCeremonyAthleteOptionDto {
    MedalCeremonyAthleteOptionDto {
        id: model.id,
        full_name: model.full_name.clone(),
        short_name: model.short_name.clone(),
        country_code: model.country_code.clone(),
        ioc_code: model.ioc_code.clone(),
        athlete_code: model.athlete_code.clone(),
    }
}

fn ceremony_to_dto(model: &MedalCeremony) -> MedalCeremonyDto {
    MedalCeremonyDto {
        id: Some(model.id.clone()),
        tournament_id: model.tournament_id,
        name: model.name.clone(),
        background_path: model.background_path.clone(),
        break_path: model.break_path.clone(),
        animation_duration: model.animation_duration,
        animation_speed: model.animation_speed,
        photo_time: model.photo_time,
        prepared_at: optional_datetime_to_string(&model.prepared_at),
        prepared_version: model.prepared_version,
        show_external: model.show_external,
        created_at: Some(datetime_to_string(&model.created_at)),
        updated_at: Some(datetime_to_string(&model.updated_at)),
    }
}

fn medalist_to_dto(model: &MedalCeremonyMedalist) -> MedalCeremonyMedalistDto {
    MedalCeremonyMedalistDto {
        id: Some(model.id.clone()),
        medal_type: model.medal_type.clone(),
        medal_rank: model.medal_rank,
        athlete_id: model.athlete_id,
        athlete_name: model.athlete_name.clone(),
        athlete_short_name: model.athlete_short_name.clone(),
        ioc_code: model.ioc_code.clone(),
        flag_asset: model.flag_asset.clone(),
        anthem_asset: model.anthem_asset.clone(),
    }
}

fn division_to_dto(detail: &MedalCeremonyDivisionDetail) -> MedalCeremonyDivisionDto {
    MedalCeremonyDivisionDto {
        id: Some(detail.division.id.clone()),
        division_id: detail.division.division_id,
        division_name: detail.division.division_name.clone(),
        order_index: detail.division.order_index,
        played_at: optional_datetime_to_string(&detail.division.played_at),
        medalists: detail
            .medalists
            .iter()
            .map(medalist_to_dto)
            .collect::<Vec<_>>(),
    }
}

fn detail_to_dto(detail: &MedalCeremonyDetail) -> MedalCeremonyDetailDto {
    MedalCeremonyDetailDto {
        ceremony: ceremony_to_dto(&detail.ceremony),
        divisions: detail
            .divisions
            .iter()
            .map(division_to_dto)
            .collect::<Vec<_>>(),
    }
}

fn dto_to_ceremony_detail(dto: &MedalCeremonyDetailDto) -> MedalCeremonyDetail {
    let ceremony = MedalCeremony {
        id: dto.ceremony.id.clone().unwrap_or_default(),
        tournament_id: dto.ceremony.tournament_id,
        name: dto.ceremony.name.clone(),
        background_path: dto.ceremony.background_path.clone(),
        break_path: dto.ceremony.break_path.clone(),
        animation_duration: dto.ceremony.animation_duration,
        animation_speed: dto.ceremony.animation_speed,
        photo_time: dto.ceremony.photo_time,
        prepared_at: parse_optional_datetime(&dto.ceremony.prepared_at),
        prepared_version: dto.ceremony.prepared_version,
        show_external: dto.ceremony.show_external,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let divisions = dto
        .divisions
        .iter()
        .map(|division_dto| {
            let division = MedalCeremonyDivision {
                id: division_dto.id.clone().unwrap_or_default(),
                ceremony_id: ceremony.id.clone(),
                division_id: division_dto.division_id,
                division_name: division_dto.division_name.clone(),
                order_index: division_dto.order_index,
                played_at: parse_optional_datetime(&division_dto.played_at),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let medalists = division_dto
                .medalists
                .iter()
                .map(|medalist_dto| MedalCeremonyMedalist {
                    id: medalist_dto.id.clone().unwrap_or_default(),
                    division_entry_id: division.id.clone(),
                    medal_type: medalist_dto.medal_type.clone(),
                    medal_rank: medalist_dto.medal_rank,
                    athlete_id: medalist_dto.athlete_id,
                    athlete_name: medalist_dto.athlete_name.clone(),
                    athlete_short_name: medalist_dto.athlete_short_name.clone(),
                    ioc_code: medalist_dto.ioc_code.clone(),
                    flag_asset: medalist_dto.flag_asset.clone(),
                    anthem_asset: medalist_dto.anthem_asset.clone(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
                .collect::<Vec<_>>();

            MedalCeremonyDivisionDetail {
                division,
                medalists,
            }
        })
        .collect::<Vec<_>>();

    MedalCeremonyDetail {
        ceremony,
        divisions,
    }
}

fn flag_asset_to_dto(asset: &OvrFlagAnimationAsset) -> FlagAnimationAssetDto {
    FlagAnimationAssetDto {
        id: Some(asset.id.clone()),
        ioc_code: asset.ioc_code.clone(),
        file_name: asset.file_name.clone(),
        file_path: asset.file_path.clone(),
        display_name: asset.display_name.clone(),
        duration_ms: asset.duration_ms,
        is_default: asset.is_default,
        created_at: Some(datetime_to_string(&asset.created_at)),
        updated_at: Some(datetime_to_string(&asset.updated_at)),
    }
}

fn dto_to_flag_asset(dto: &FlagAnimationAssetDto) -> OvrFlagAnimationAsset {
    OvrFlagAnimationAsset {
        id: dto.id.clone().unwrap_or_default(),
        ioc_code: dto.ioc_code.clone(),
        file_name: dto.file_name.clone(),
        file_path: dto.file_path.clone(),
        display_name: dto.display_name.clone(),
        duration_ms: dto.duration_ms,
        is_default: dto.is_default,
        created_at: parse_optional_datetime(&dto.created_at).unwrap_or_else(Utc::now),
        updated_at: parse_optional_datetime(&dto.updated_at).unwrap_or_else(Utc::now),
    }
}

fn anthem_to_dto(asset: &OvrAnthemAsset) -> AnthemAssetDto {
    AnthemAssetDto {
        id: Some(asset.id.clone()),
        ioc_code: asset.ioc_code.clone(),
        file_name: asset.file_name.clone(),
        file_path: asset.file_path.clone(),
        display_name: asset.display_name.clone(),
        duration_ms: asset.duration_ms,
        is_default: asset.is_default,
        created_at: Some(datetime_to_string(&asset.created_at)),
        updated_at: Some(datetime_to_string(&asset.updated_at)),
    }
}

fn dto_to_anthem(dto: &AnthemAssetDto) -> OvrAnthemAsset {
    OvrAnthemAsset {
        id: dto.id.clone().unwrap_or_default(),
        ioc_code: dto.ioc_code.clone(),
        file_name: dto.file_name.clone(),
        file_path: dto.file_path.clone(),
        display_name: dto.display_name.clone(),
        duration_ms: dto.duration_ms,
        is_default: dto.is_default,
        created_at: parse_optional_datetime(&dto.created_at).unwrap_or_else(Utc::now),
        updated_at: parse_optional_datetime(&dto.updated_at).unwrap_or_else(Utc::now),
    }
}

#[tauri::command]
pub async fn medal_ceremony_list_divisions(
    app: State<'_, Arc<App>>,
) -> Result<Vec<MedalCeremonyDivisionOptionDto>, TauriError> {
    let conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    let options = MedalCeremonyOperations::list_division_options(&*conn)
        .map_err(|e| map_db_error("Failed to load division options", e))?;
    Ok(options.iter().map(division_option_to_dto).collect())
}

#[tauri::command]
pub async fn medal_ceremony_list_athletes(
    division: String,
    app: State<'_, Arc<App>>,
) -> Result<Vec<MedalCeremonyAthleteOptionDto>, TauriError> {
    let conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    let options = MedalCeremonyOperations::list_athlete_options(&*conn, &division)
        .map_err(|e| map_db_error("Failed to load athlete options", e))?;
    Ok(options.iter().map(athlete_option_to_dto).collect())
}

#[tauri::command]
pub async fn medal_ceremony_list(
    app: State<'_, Arc<App>>,
) -> Result<Vec<MedalCeremonySummaryDto>, TauriError> {
    let conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    let ceremonies = MedalCeremonyOperations::list_ceremonies(&*conn)
        .map_err(|e| map_db_error("Failed to load medal ceremonies", e))?;
    Ok(ceremonies
        .iter()
        .map(ceremony_to_summary)
        .collect::<Vec<_>>())
}

#[tauri::command]
pub async fn medal_ceremony_get(
    ceremony_id: String,
    app: State<'_, Arc<App>>,
) -> Result<Option<MedalCeremonyDetailDto>, TauriError> {
    let conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    let detail = MedalCeremonyOperations::get_ceremony_detail(&*conn, &ceremony_id)
        .map_err(|e| map_db_error("Failed to load medal ceremony", e))?;
    Ok(detail.as_ref().map(detail_to_dto))
}

#[tauri::command]
pub async fn medal_ceremony_save(
    payload: MedalCeremonyDetailDto,
    app: State<'_, Arc<App>>,
) -> Result<String, TauriError> {
    let mut conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    let detail = dto_to_ceremony_detail(&payload);
    let id = MedalCeremonyOperations::upsert_ceremony(&mut *conn, &detail)
        .map_err(|e| map_db_error("Failed to save medal ceremony", e))?;
    Ok(id)
}

#[tauri::command]
pub async fn medal_ceremony_delete(
    ceremony_id: String,
    app: State<'_, Arc<App>>,
) -> Result<(), TauriError> {
    let mut conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    MedalCeremonyOperations::delete_ceremony(&mut *conn, &ceremony_id)
        .map_err(|e| map_db_error("Failed to delete medal ceremony", e))?;
    Ok(())
}

#[tauri::command]
pub async fn medal_ceremony_prepare(
    ceremony_id: String,
    app: State<'_, Arc<App>>,
) -> Result<Vec<MedalCeremonyDivisionDto>, TauriError> {
    let mut conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    let divisions = MedalCeremonyOperations::prepare_playlist(&mut *conn, &ceremony_id)
        .map_err(|e| map_db_error("Failed to prepare medal ceremony", e))?;
    Ok(divisions.iter().map(division_to_dto).collect())
}

#[tauri::command]
pub async fn medal_ceremony_mark_division_played(
    division_id: String,
    app: State<'_, Arc<App>>,
) -> Result<(), TauriError> {
    let mut conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    MedalCeremonyOperations::mark_division_played(&mut *conn, &division_id)
        .map_err(|e| map_db_error("Failed to update ceremony progress", e))?;
    Ok(())
}

#[tauri::command]
pub async fn medal_ceremony_reset_playback(
    ceremony_id: String,
    app: State<'_, Arc<App>>,
) -> Result<(), TauriError> {
    let mut conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    MedalCeremonyOperations::reset_playback(&mut *conn, &ceremony_id)
        .map_err(|e| map_db_error("Failed to reset ceremony playback", e))?;
    Ok(())
}

#[tauri::command]
pub async fn medal_ceremony_set_show_external(
    ceremony_id: String,
    enabled: bool,
    app: State<'_, Arc<App>>,
) -> Result<(), TauriError> {
    let mut conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    MedalCeremonyOperations::update_show_external(&mut *conn, &ceremony_id, enabled)
        .map_err(|e| map_db_error("Failed to update external screen state", e))?;
    Ok(())
}

#[tauri::command]
pub async fn medal_ceremony_list_flag_assets(
    app: State<'_, Arc<App>>,
) -> Result<Vec<FlagAnimationAssetDto>, TauriError> {
    let conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    let assets = MedalCeremonyOperations::list_flag_animations(&*conn)
        .map_err(|e| map_db_error("Failed to load flag animations", e))?;
    Ok(assets.iter().map(flag_asset_to_dto).collect())
}

#[tauri::command]
pub async fn medal_ceremony_save_flag_asset(
    asset: FlagAnimationAssetDto,
    app: State<'_, Arc<App>>,
) -> Result<String, TauriError> {
    let mut conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    let model = dto_to_flag_asset(&asset);
    let id = MedalCeremonyOperations::upsert_flag_animation(&mut *conn, &model)
        .map_err(|e| map_db_error("Failed to save flag animation", e))?;
    Ok(id)
}

#[tauri::command]
pub async fn medal_ceremony_delete_flag_asset(
    asset_id: String,
    app: State<'_, Arc<App>>,
) -> Result<(), TauriError> {
    let mut conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    MedalCeremonyOperations::delete_flag_animation(&mut *conn, &asset_id)
        .map_err(|e| map_db_error("Failed to delete flag animation", e))?;
    Ok(())
}

#[tauri::command]
pub async fn medal_ceremony_list_anthems(
    app: State<'_, Arc<App>>,
) -> Result<Vec<AnthemAssetDto>, TauriError> {
    let conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    let assets = MedalCeremonyOperations::list_anthems(&*conn)
        .map_err(|e| map_db_error("Failed to load anthem assets", e))?;
    Ok(assets.iter().map(anthem_to_dto).collect())
}

#[tauri::command]
pub async fn medal_ceremony_save_anthem(
    asset: AnthemAssetDto,
    app: State<'_, Arc<App>>,
) -> Result<String, TauriError> {
    let mut conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    let model = dto_to_anthem(&asset);
    let id = MedalCeremonyOperations::upsert_anthem(&mut *conn, &model)
        .map_err(|e| map_db_error("Failed to save anthem asset", e))?;
    Ok(id)
}

#[tauri::command]
pub async fn medal_ceremony_delete_anthem(
    asset_id: String,
    app: State<'_, Arc<App>>,
) -> Result<(), TauriError> {
    let mut conn = app
        .database_plugin()
        .get_pooled_connection()
        .map_err(|e| map_db_error("Failed to acquire database connection", e))?;
    MedalCeremonyOperations::delete_anthem(&mut *conn, &asset_id)
        .map_err(|e| map_db_error("Failed to delete anthem asset", e))?;
    Ok(())
}
