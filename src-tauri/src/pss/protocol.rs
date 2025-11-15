use crate::types::{AppError, AppResult};
use serde::{Deserialize, Serialize};

/// Parsed PSS event produced directly from WT UDP payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PssEvent {
    Points {
        athlete: u8,
        point_type: u8,
    },
    HitLevel {
        athlete: u8,
        level: u8,
    },
    Warnings {
        athlete1_warnings: u8,
        athlete2_warnings: u8,
    },
    Injury {
        athlete: u8,
        time: String,
        action: Option<String>,
    },
    Challenge {
        source: u8,
        accepted: Option<bool>,
        won: Option<bool>,
        canceled: bool,
    },
    Break {
        time: String,
        action: Option<String>,
    },
    WinnerRounds {
        round1_winner: u8,
        round2_winner: u8,
        round3_winner: u8,
    },
    Winner {
        name: String,
        classification: Option<String>,
    },
    Athletes {
        athlete1_short: String,
        athlete1_long: String,
        athlete1_country: String,
        athlete2_short: String,
        athlete2_long: String,
        athlete2_country: String,
    },
    MatchConfig {
        number: String,
        category: String,
        weight: String,
        rounds: u8,
        colors: (String, String, String, String),
        match_id: String,
        division: String,
        total_rounds: u8,
        round_duration: u32,
        countdown_type: String,
        count_up: u32,
        format: u8,
    },
    Scores {
        athlete1_r1: u8,
        athlete2_r1: u8,
        athlete1_r2: u8,
        athlete2_r2: u8,
        athlete1_r3: u8,
        athlete2_r3: u8,
    },
    CurrentScores {
        athlete1_score: u8,
        athlete2_score: u8,
    },
    Clock {
        time: String,
        action: Option<String>,
    },
    Round {
        current_round: u8,
    },
    FightLoaded,
    FightReady,
    Supremacy {
        value: u8,
    },
    VideoTime {
        value: u8,
    },
    Raw(String),
}

/// Parser for WT UDP payloads.
pub struct PssProtocol;

impl PssProtocol {
    #[allow(clippy::unused_self)]
    pub fn new() -> Self {
        Self
    }

    /// Parse a raw UDP packet into a structured `PssEvent`.
    pub fn parse_message(&self, message: &str) -> AppResult<PssEvent> {
        log::debug!("Parsing PSS message: '{message}'");

        let clean = message.trim().trim_end_matches(';').to_string();
        if clean.is_empty() {
            return Ok(PssEvent::Raw(message.to_string()));
        }

        if message.contains("Udp Port")
            && (message.contains("connected") || message.contains("disconnected"))
        {
            return Ok(PssEvent::Raw(message.to_string()));
        }

        let parts: Vec<&str> = clean.split(';').filter(|p| !p.is_empty()).collect();
        if parts.is_empty() {
            return Ok(PssEvent::Raw(message.to_string()));
        }

        let get_part = |index: usize| -> Option<&str> {
            if index < parts.len() {
                Some(parts[index])
            } else {
                None
            }
        };

        let parse_u8 = |index: usize, field_name: &str, min: u8, max: u8| -> AppResult<u8> {
            let value = get_part(index)
                .ok_or_else(|| AppError::ConfigError(format!("Missing {field_name} at {index}")))?;
            let parsed = value.parse::<u8>().map_err(|_| {
                AppError::ConfigError(format!("Invalid {field_name}: '{value}' (not u8)"))
            })?;
            if parsed < min || parsed > max {
                return Err(AppError::ConfigError(format!(
                    "{field_name} value {parsed} is out of range [{min}, {max}]"
                )));
            }
            Ok(parsed)
        };

        let parse_u32 = |index: usize, field_name: &str, min: u32, max: u32| -> AppResult<u32> {
            let value = get_part(index)
                .ok_or_else(|| AppError::ConfigError(format!("Missing {field_name} at {index}")))?;
            let parsed = value.parse::<u32>().map_err(|_| {
                AppError::ConfigError(format!("Invalid {field_name}: '{value}' (not u32)"))
            })?;
            if parsed < min || parsed > max {
                return Err(AppError::ConfigError(format!(
                    "{field_name} value {parsed} is out of range [{min}, {max}]"
                )));
            }
            Ok(parsed)
        };

        let get_string =
            |index: usize, field_name: &str, max_length: usize| -> AppResult<String> {
                let value = get_part(index).ok_or_else(|| {
                    AppError::ConfigError(format!("Missing {field_name} at {index}"))
                })?;
                if value.len() > max_length {
                    return Err(AppError::ConfigError(format!(
                        "{field_name} too long: {} chars (max {max_length})",
                        value.len()
                    )));
                }
                Ok(value.to_string())
            };

        let validate_time_format = |time: &str| -> bool {
            if time.contains(':') {
                let segments: Vec<&str> = time.split(':').collect();
                if segments.len() != 2 {
                    return false;
                }
                segments[0].parse::<u8>().is_ok() && segments[1].parse::<u8>().is_ok()
            } else {
                time.parse::<u8>().is_ok()
            }
        };

        let validate_color_format =
            |color: &str| color.starts_with('#') && color.len() == 7 && color[1..].chars().all(|c| c.is_ascii_hexdigit());

        let parse_with_fallback = |result: AppResult<PssEvent>| -> AppResult<PssEvent> {
            match result {
                Ok(event) => Ok(event),
                Err(e) => {
                    log::warn!("Parsing failed for '{message}': {e}. Returning Raw event.");
                    Ok(PssEvent::Raw(message.to_string()))
                }
            }
        };

        let event = match *parts.first().unwrap_or(&"") {
            "pt1" => {
                let point_type = parse_u8(1, "point type", 1, 5)?;
                Ok(PssEvent::Points {
                    athlete: 1,
                    point_type,
                })
            }
            "pt2" => {
                let point_type = parse_u8(1, "point type", 1, 5)?;
                Ok(PssEvent::Points {
                    athlete: 2,
                    point_type,
                })
            }
            "hl1" => {
                let level = parse_u8(1, "hit level", 1, 100)?;
                Ok(PssEvent::HitLevel { athlete: 1, level })
            }
            "hl2" => {
                let level = parse_u8(1, "hit level", 1, 100)?;
                Ok(PssEvent::HitLevel { athlete: 2, level })
            }
            "wg1" => {
                let athlete1_warnings = parse_u8(1, "athlete1 warnings", 0, 10)?;
                let athlete2_warnings = if parts.len() >= 4 && *parts.get(2).unwrap_or(&"") == "wg2"
                {
                    parse_u8(3, "athlete2 warnings", 0, 10)?
                } else {
                    0
                };
                Ok(PssEvent::Warnings {
                    athlete1_warnings,
                    athlete2_warnings,
                })
            }
            "wg2" => {
                if parts.len() >= 2 {
                    let athlete2_warnings = parse_u8(1, "athlete2 warnings", 0, 10)?;
                    Ok(PssEvent::Warnings {
                        athlete1_warnings: 0,
                        athlete2_warnings,
                    })
                } else {
                    Ok(PssEvent::Raw(message.to_string()))
                }
            }
            "ij0" | "ij1" | "ij2" => {
                let athlete = match parts[0] {
                    "ij0" => 0,
                    "ij1" => 1,
                    _ => 2,
                };
                let time = get_string(1, "injury time", 8)?;
                if !validate_time_format(&time) {
                    return parse_with_fallback(Err(AppError::ConfigError(format!(
                        "Invalid injury time format: {time}"
                    ))));
                }
                let action = get_part(2).map(|s| s.to_string());
                Ok(PssEvent::Injury {
                    athlete,
                    time,
                    action,
                })
            }
            "ch0" | "ch1" | "ch2" => {
                let source = match parts[0] {
                    "ch0" => 0,
                    "ch1" => 1,
                    _ => 2,
                };
                let canceled = parts
                    .iter()
                    .any(|token| token.trim().eq_ignore_ascii_case("-1"));
                let accepted = if parts.len() > 1 && parts[1] != "-1" {
                    Some(parts[1] == "1")
                } else {
                    None
                };
                let won = if parts.len() > 2 { Some(parts[2] == "1") } else { None };
                Ok(PssEvent::Challenge {
                    source,
                    accepted,
                    won,
                    canceled,
                })
            }
            "brk" => {
                let time = get_string(1, "break time", 8)?;
                if !validate_time_format(&time) {
                    return parse_with_fallback(Err(AppError::ConfigError(format!(
                        "Invalid break time format: {time}"
                    ))));
                }
                let action = get_part(2).map(|s| s.to_string());
                Ok(PssEvent::Break { time, action })
            }
            "wrd" => {
                let mut round1_winner = 0;
                let mut round2_winner = 0;
                let mut round3_winner = 0;
                let mut i = 1;
                while i < parts.len() {
                    match parts[i] {
                        "rd1" => {
                            round1_winner = parse_u8(i + 1, "round1 winner", 0, 2).unwrap_or(0);
                            i += 2;
                        }
                        "rd2" => {
                            round2_winner = parse_u8(i + 1, "round2 winner", 0, 2).unwrap_or(0);
                            i += 2;
                        }
                        "rd3" => {
                            round3_winner = parse_u8(i + 1, "round3 winner", 0, 2).unwrap_or(0);
                            i += 2;
                        }
                        _ => i += 1,
                    }
                }
                Ok(PssEvent::WinnerRounds {
                    round1_winner,
                    round2_winner,
                    round3_winner,
                })
            }
            "wmh" => {
                let name = get_string(1, "winner name", 64)?;
                let classification = get_part(2).map(|c| c.to_string());
                Ok(PssEvent::Winner {
                    name,
                    classification,
                })
            }
            "at1" => {
                let athlete1_short = get_string(1, "athlete1 short name", 32)?;
                let athlete1_long = get_string(2, "athlete1 long name", 64)?;
                let athlete1_country = get_string(3, "athlete1 country", 3)?;
                let athlete2_short = get_string(5, "athlete2 short name", 32)?;
                let athlete2_long = get_string(6, "athlete2 long name", 64)?;
                let athlete2_country = get_string(7, "athlete2 country", 3)?;
                Ok(PssEvent::Athletes {
                    athlete1_short,
                    athlete1_long,
                    athlete1_country,
                    athlete2_short,
                    athlete2_long,
                    athlete2_country,
                })
            }
            "mch" => {
                let number = get_string(1, "match number", 16)?;
                let category = get_string(2, "category", 64)?;
                let weight = get_string(3, "weight", 32)?;
                let rounds = parse_u8(4, "rounds", 0, 10)?;
                let color1_bg = get_string(5, "color1 background", 7)?;
                let color1_fg = get_string(6, "color1 foreground", 7)?;
                let color2_bg = get_string(7, "color2 background", 7)?;
                let color2_fg = get_string(8, "color2 foreground", 7)?;
                if !validate_color_format(&color1_bg)
                    || !validate_color_format(&color1_fg)
                    || !validate_color_format(&color2_bg)
                    || !validate_color_format(&color2_fg)
                {
                    return parse_with_fallback(Err(AppError::ConfigError(
                        "Invalid color format in match config".into(),
                    )));
                }
                let match_id = get_string(9, "match id", 64)?;
                let division = get_string(10, "division", 32)?;
                let total_rounds = parse_u8(11, "total rounds", 1, 10)?;
                let round_duration = parse_u32(12, "round duration", 30, 600)?;
                let countdown_type = get_string(13, "countdown type", 16)?;
                let count_up = parse_u32(14, "count up", 0, 999)?;
                let format = parse_u8(15, "format", 1, 10)?;
                Ok(PssEvent::MatchConfig {
                    number,
                    category,
                    weight,
                    rounds,
                    colors: (color1_bg, color1_fg, color2_bg, color2_fg),
                    match_id,
                    division,
                    total_rounds,
                    round_duration,
                    countdown_type,
                    count_up,
                    format,
                })
            }
            code if code.starts_with('s') => {
                let mut athlete1_r1 = 0;
                let mut athlete2_r1 = 0;
                let mut athlete1_r2 = 0;
                let mut athlete2_r2 = 0;
                let mut athlete1_r3 = 0;
                let mut athlete2_r3 = 0;

                let mut i = 0;
                while i < parts.len() {
                    match parts[i] {
                        "s11" => {
                            athlete1_r1 =
                                parse_u8(i + 1, "athlete1 round1 score", 0, 50).unwrap_or(0);
                            i += 2;
                        }
                        "s21" => {
                            athlete2_r1 =
                                parse_u8(i + 1, "athlete2 round1 score", 0, 50).unwrap_or(0);
                            i += 2;
                        }
                        "s12" => {
                            athlete1_r2 =
                                parse_u8(i + 1, "athlete1 round2 score", 0, 50).unwrap_or(0);
                            i += 2;
                        }
                        "s22" => {
                            athlete2_r2 =
                                parse_u8(i + 1, "athlete2 round2 score", 0, 50).unwrap_or(0);
                            i += 2;
                        }
                        "s13" => {
                            athlete1_r3 =
                                parse_u8(i + 1, "athlete1 round3 score", 0, 50).unwrap_or(0);
                            i += 2;
                        }
                        "s23" => {
                            athlete2_r3 =
                                parse_u8(i + 1, "athlete2 round3 score", 0, 50).unwrap_or(0);
                            i += 2;
                        }
                        _ => i += 1,
                    }
                }

                Ok(PssEvent::Scores {
                    athlete1_r1,
                    athlete2_r1,
                    athlete1_r2,
                    athlete2_r2,
                    athlete1_r3,
                    athlete2_r3,
                })
            }
            "sc1" | "sc2" => {
                let mut athlete1_score = 0;
                let mut athlete2_score = 0;
                let mut i = 0;
                while i < parts.len() {
                    match parts[i] {
                        "sc1" => {
                            athlete1_score =
                                parse_u8(i + 1, "athlete1 score", 0, 50).unwrap_or(0);
                            i += 2;
                        }
                        "sc2" => {
                            athlete2_score =
                                parse_u8(i + 1, "athlete2 score", 0, 50).unwrap_or(0);
                            i += 2;
                        }
                        _ => i += 1,
                    }
                }
                Ok(PssEvent::CurrentScores {
                    athlete1_score,
                    athlete2_score,
                })
            }
            "clk" => {
                let time = get_string(1, "clock time", 8)?;
                if !validate_time_format(&time) {
                    return parse_with_fallback(Err(AppError::ConfigError(format!(
                        "Invalid clock time format: {time}"
                    ))));
                }
                let action = get_part(2).map(|s| s.to_string());
                Ok(PssEvent::Clock { time, action })
            }
            "rnd" => {
                let current_round = parse_u8(1, "current round", 1, 10)?;
                Ok(PssEvent::Round { current_round })
            }
            "pre" => Ok(PssEvent::FightLoaded),
            "rdy" => Ok(PssEvent::FightReady),
            "win" => {
                let name = get_string(1, "winner color", 16)?;
                Ok(PssEvent::Winner {
                    name,
                    classification: None,
                })
            }
            "sup" => {
                let value = parse_u8(1, "supremacy value", 0, 255)?;
                Ok(PssEvent::Supremacy { value })
            }
            "avt" => {
                let value = parse_u8(1, "video time", 0, 255)?;
                Ok(PssEvent::VideoTime { value })
            }
            _ => Ok(PssEvent::Raw(message.to_string())),
        };

        result
    }
}
