//! Canonical SeaORM entity definitions for the unified database schema.
//!
//! The application still exposes compatibility views for legacy table names,
//! but new Rust code should depend on these SeaORM entities instead of
//! issuing raw SQL against the old `pss_*` / `ovr_*` schemas.

pub mod age_group;
pub mod athlete;
pub mod discipline;
pub mod division;
pub mod event;
pub mod event_detail;
pub mod event_recognition_history;
pub mod event_statistic;
pub mod event_type;
pub mod event_unknown;
pub mod event_validation_result;
pub mod event_validation_rule;
pub mod event_warning;
pub mod flag;
pub mod gender;
pub mod match_participant;
pub mod matches;
pub mod medal_ceremony;
pub mod medal_ceremony_division;
pub mod medal_ceremony_medalist;
pub mod obs_connection;
pub mod obs_recording_config;
pub mod obs_recording_session;
pub mod obs_scene;
pub mod octagon;
pub mod overlay_anthem;
pub mod overlay_category;
pub mod overlay_flag_animation;
pub mod overlay_provider;
pub mod overlay_tournament;
pub mod overlay_tournament_map;
pub mod round;
pub mod round_config;
pub mod score;
pub mod tournament;
pub mod tournament_champion;
pub mod tournament_day;
pub mod tournament_ranking;
pub mod udp_client_connection;
pub mod udp_server_config;
pub mod udp_server_session;
pub mod video;
pub mod video_event;
pub mod weight_class;

/// Convenient re-export of all entity `Entity` types and SeaORM prelude items.
pub mod prelude {
    pub use super::age_group::Entity as AgeGroup;
    pub use super::athlete::Entity as Athlete;
    pub use super::discipline::Entity as Discipline;
    pub use super::division::Entity as Division;
    pub use super::event::Entity as Event;
    pub use super::event_detail::Entity as EventDetail;
    pub use super::event_recognition_history::Entity as EventRecognitionHistory;
    pub use super::event_statistic::Entity as EventStatistic;
    pub use super::event_type::Entity as EventType;
    pub use super::event_unknown::Entity as EventUnknown;
    pub use super::event_validation_result::Entity as EventValidationResult;
    pub use super::event_validation_rule::Entity as EventValidationRule;
    pub use super::event_warning::Entity as EventWarning;
    pub use super::flag::Entity as Flag;
    pub use super::gender::Entity as Gender;
    pub use super::match_participant::Entity as MatchParticipant;
    pub use super::matches::Entity as Match;
    pub use super::medal_ceremony::Entity as MedalCeremony;
    pub use super::medal_ceremony_division::Entity as MedalCeremonyDivision;
    pub use super::medal_ceremony_medalist::Entity as MedalCeremonyMedalist;
    pub use super::obs_connection::Entity as ObsConnection;
    pub use super::obs_recording_config::Entity as ObsRecordingConfig;
    pub use super::obs_recording_session::Entity as ObsRecordingSession;
    pub use super::obs_scene::Entity as ObsScene;
    pub use super::octagon::Entity as Octagon;
    pub use super::overlay_anthem::Entity as OverlayAnthem;
    pub use super::overlay_category::Entity as OverlayCategory;
    pub use super::overlay_flag_animation::Entity as OverlayFlagAnimation;
    pub use super::overlay_provider::Entity as OverlayProvider;
    pub use super::overlay_tournament::Entity as OverlayTournament;
    pub use super::overlay_tournament_map::Entity as OverlayTournamentMap;
    pub use super::round::Entity as Round;
    pub use super::round_config::Entity as RoundConfig;
    pub use super::score::Entity as Score;
    pub use super::tournament::Entity as Tournament;
    pub use super::tournament_champion::Entity as TournamentChampion;
    pub use super::tournament_day::Entity as TournamentDay;
    pub use super::tournament_ranking::Entity as TournamentRanking;
    pub use super::udp_client_connection::Entity as UdpClientConnection;
    pub use super::udp_server_config::Entity as UdpServerConfig;
    pub use super::udp_server_session::Entity as UdpServerSession;
    pub use super::video::Entity as Video;
    pub use super::video_event::Entity as VideoEvent;
    pub use super::weight_class::Entity as WeightClass;
    pub use sea_orm::entity::prelude::*;
}
