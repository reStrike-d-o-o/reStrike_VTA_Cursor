use crate::plugins::obs_obws::recording_events::RecordingSession;
use chrono::Local;

/// Format a filename based on the template and session metadata
pub fn format_filename(template: &str, session: &RecordingSession) -> String {
    let mut formatting = template.to_string();

    // Replace placeholders
    if let Some(ref p1) = session.player1_name {
        formatting = formatting.replace("{player1}", p1);
    } else {
        formatting = formatting.replace("{player1}", "Player1");
    }

    if let Some(ref p1f) = session.player1_flag {
        formatting = formatting.replace("{country1}", p1f);
    } else {
        formatting = formatting.replace("{country1}", "UNK");
    }

    if let Some(ref p2) = session.player2_name {
        formatting = formatting.replace("{player2}", p2);
    } else {
        formatting = formatting.replace("{player2}", "Player2");
    }

    if let Some(ref p2f) = session.player2_flag {
        formatting = formatting.replace("{country2}", p2f);
    } else {
        formatting = formatting.replace("{country2}", "UNK");
    }

    if let Some(ref mnum) = session.match_number {
        formatting = formatting.replace("{matchNumber}", mnum);
    } else {
        formatting = formatting.replace("{matchNumber}", "000");
    }

    // Date/Time placeholders
    let now = Local::now();
    formatting = formatting.replace("{date}", &now.format("%Y-%m-%d").to_string());
    formatting = formatting.replace("{time}", &now.format("%H-%M-%S").to_string());

    // Sanitize filename
    sanitize_filename(formatting)
}

/// Sanitize a filename by replacing invalid characters
pub fn sanitize_filename(filename: String) -> String {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut sanitized = filename;
    for c in invalid_chars {
        sanitized = sanitized.replace(c, "_");
    }
    // Also trim whitespace
    sanitized.trim().to_string()
}
