//! Canonical SeaORM entity definitions for the unified database schema.
//! 
//! The application still exposes compatibility views for legacy table names,
//! but new Rust code should depend on these SeaORM entities instead of
//! issuing raw SQL against the old `pss_*` / `ovr_*` schemas.

pub mod age_group;
pub mod athlete;
pub mod division;
pub mod discipline;
pub mod flag;
pub mod gender;
pub mod medal_ceremony;
pub mod medal_ceremony_division;
pub mod medal_ceremony_medalist;
pub mod match_participant;
pub mod matches;
pub mod octagon;
pub mod round_config;
pub mod weight_class;
pub mod tournament;
pub mod tournament_champion;
pub mod tournament_day;
pub mod tournament_ranking;

/// Convenient re-export of all entity `Entity` types and SeaORM prelude items.
pub mod prelude {
    pub use super::age_group::Entity as AgeGroup;
    pub use super::athlete::Entity as Athlete;
    pub use super::division::Entity as Division;
    pub use super::discipline::Entity as Discipline;
    pub use super::flag::Entity as Flag;
    pub use super::gender::Entity as Gender;
    pub use super::medal_ceremony::Entity as MedalCeremony;
    pub use super::medal_ceremony_division::Entity as MedalCeremonyDivision;
    pub use super::medal_ceremony_medalist::Entity as MedalCeremonyMedalist;
    pub use super::match_participant::Entity as MatchParticipant;
    pub use super::matches::Entity as Match;
    pub use super::octagon::Entity as Octagon;
    pub use super::round_config::Entity as RoundConfig;
    pub use super::weight_class::Entity as WeightClass;
    pub use super::tournament::Entity as Tournament;
    pub use super::tournament_champion::Entity as TournamentChampion;
    pub use super::tournament_day::Entity as TournamentDay;
    pub use super::tournament_ranking::Entity as TournamentRanking;
    pub use sea_orm::entity::prelude::*;
}
