use std::path::{Path, PathBuf};
use chrono::Local;
use crate::types::AppResult;

/// Path generation configuration
#[derive(Debug, Clone)]
pub struct PathGeneratorConfig {
    pub videos_root: PathBuf,
    pub default_format: String,
    pub include_minutes_seconds: bool,
    pub folder_pattern: Option<String>,
}

impl Default for PathGeneratorConfig {
    fn default() -> Self {
        Self {
            videos_root: Self::detect_windows_videos_folder(),
            default_format: "mp4".to_string(),
            include_minutes_seconds: true,
            folder_pattern: Some("{tournament}/{tournamentDay}".to_string()),
        }
    }
}

impl PathGeneratorConfig {
    /// Detect Windows Videos folder
    pub fn detect_windows_videos_folder() -> PathBuf {
        // Try to get the Videos folder from environment
        if let Ok(videos_path) = std::env::var("USERPROFILE") {
            let videos_folder = PathBuf::from(videos_path).join("Videos");
            if videos_folder.exists() {
                return videos_folder;
            }
        }
        
        // Fallback to default Windows path
        PathBuf::from("C:/Users/Damjan/Videos")
    }
}

/// Path generation result
#[derive(Debug, Clone)]
pub struct GeneratedPath {
    pub full_path: PathBuf,
    pub directory: PathBuf,
    pub filename: String,
    pub tournament_name: Option<String>,
    pub tournament_day: Option<String>,
    pub match_number: Option<String>,
}

/// OBS Recording Path Generator
pub struct ObsPathGenerator {
    config: PathGeneratorConfig,
}

impl ObsPathGenerator {
    /// Create a new path generator
    pub fn new(config: Option<PathGeneratorConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }
    
    /// Generate recording path for a match
    pub fn generate_recording_path(
        &self,
        match_id: &str,
        tournament_name: Option<String>,
        tournament_day: Option<String>,
        match_number: Option<String>,
        player1_name: Option<String>,
        player1_flag: Option<String>,
        player2_name: Option<String>,
        player2_flag: Option<String>,
    ) -> AppResult<GeneratedPath> {
        // Create match info
        let match_info = MatchInfo {
            match_id: match_id.to_string(),
            match_number,
            player1_name,
            player1_flag,
            player2_name,
            player2_flag,
        };
        
        // Generate directory path
        let directory = self.generate_directory_path(&tournament_name, &tournament_day, &match_info.match_number);
        
        // Generate filename
        let filename = self.generate_filename(&match_info, &tournament_name, &tournament_day);
        
        // Create full path
        let full_path = directory.join(&filename);
        
        Ok(GeneratedPath {
            full_path,
            directory,
            filename,
            tournament_name,
            tournament_day,
            match_number: match_info.match_number,
        })
    }
    
    /// Generate directory path
    pub fn generate_directory_path(
        &self,
        tournament_name: &Option<String>,
        tournament_day: &Option<String>,
        _match_number: &Option<String>,
    ) -> PathBuf {
        let mut path = self.config.videos_root.clone();
        let pattern = self
            .config
            .folder_pattern
            .clone()
            .unwrap_or_else(|| "{tournament}/{tournamentDay}".to_string());
        let t_name = tournament_name.clone().unwrap_or_else(|| {
            let now = Local::now();
            format!("Tournament_{}", now.format("%Y-%m-%d"))
        });
        let t_day = tournament_day.clone().unwrap_or_else(|| {
            let now = Local::now();
            format!("Day_{}", now.format("%Y-%m-%d"))
        });
        for seg in pattern.split('/') {
            let seg_value = match seg {
                "{tournament}" => &t_name,
                "{tournamentDay}" => &t_day,
                other => other,
            };
            if seg_value.is_empty() { continue; }
            path.push(self.sanitize_filename(seg_value));
        }
        path
    }
    
    /// Generate filename
    pub fn generate_filename(
        &self,
        match_info: &MatchInfo,
        _tournament_name: &Option<String>,
        _tournament_day: &Option<String>,
    ) -> String {
        let now = Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let time_str = if self.config.include_minutes_seconds {
            now.format("%H-%M-%S").to_string()
        } else {
            now.format("%H-%M").to_string()
        };
        
        // Build filename components
        let mut components = Vec::new();
        
        // Match number
        if let Some(match_num) = &match_info.match_number {
            components.push(match_num.clone());
        }
        
        // Player 1
        if let Some(player1) = &match_info.player1_name {
            let player1_str = if let Some(flag1) = &match_info.player1_flag {
                format!("{}_{}", player1, flag1)
            } else {
                player1.clone()
            };
            components.push(player1_str);
        }
        
        // vs
        components.push("vs".to_string());
        
        // Player 2
        if let Some(player2) = &match_info.player2_name {
            let player2_str = if let Some(flag2) = &match_info.player2_flag {
                format!("{}_{}", player2, flag2)
            } else {
                player2.clone()
            };
            components.push(player2_str);
        }
        
        // Date and time
        components.push(date_str);
        components.push(time_str);
        
        // Join components and add extension
        let filename = components.join("_");
        format!("{}.{}", filename, self.config.default_format)
    }
    

    
    /// Sanitize filename for Windows compatibility
    fn sanitize_filename(&self, filename: &str) -> String {
        filename
            .chars()
            .map(|c| match c {
                '<' | '>' | ':' | '"' | '|' | '?' | '*' | '\\' | '/' => '_',
                _ => c,
            })
            .collect::<String>()
            .trim()
            .to_string()
    }
    
    /// Ensure directory exists
    pub fn ensure_directory_exists(&self, path: &Path) -> AppResult<()> {
        if !path.exists() {
            std::fs::create_dir_all(path)
                .map_err(|e| crate::types::AppError::IoError(e))?;
        }
        Ok(())
    }
}

/// Match information for path generation
#[derive(Debug, Clone)]
pub struct MatchInfo {
    pub match_id: String,
    pub match_number: Option<String>,
    pub player1_name: Option<String>,
    pub player1_flag: Option<String>,
    pub player2_name: Option<String>,
    pub player2_flag: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_sanitize_filename() {
        let generator = ObsPathGenerator::new(None);
        
        assert_eq!(generator.sanitize_filename("Test:File"), "Test_File");
        assert_eq!(generator.sanitize_filename("Test/File"), "Test_File");
        assert_eq!(generator.sanitize_filename("Test*File"), "Test_File");
        assert_eq!(generator.sanitize_filename("Test File"), "Test File");
    }
    
    #[test]
    fn test_generate_filename() {
        let config = PathGeneratorConfig {
            videos_root: PathBuf::from("C:/Videos"),
            default_format: "mp4".to_string(),
            include_minutes_seconds: true,
        };
        
        let generator = ObsPathGenerator::new(Some(config));
        
        let match_info = MatchInfo {
            match_id: "101".to_string(),
            match_number: Some("101".to_string()),
            player1_name: Some("N. DESMOND".to_string()),
            player1_flag: Some("MRN".to_string()),
            player2_name: Some("M. THIBAULT".to_string()),
            player2_flag: Some("SUI".to_string()),
        };
        
        let filename = generator.generate_filename(&match_info, &Some("Test Tournament".to_string()), &Some("Day 1".to_string()));
        
        // Should contain match number, players, and date/time
        assert!(filename.contains("101"));
        assert!(filename.contains("N._DESMOND_MRN"));
        assert!(filename.contains("vs"));
        assert!(filename.contains("M._THIBAULT_SUI"));
        assert!(filename.ends_with(".mp4"));
    }
}
