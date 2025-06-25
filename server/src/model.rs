
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub player_name: String,
    pub score: u32,
    pub level: u32,
    pub difficulty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateScoreRequest {
    pub player_name: String,
    pub score: u32,
    pub level: u32,
    pub difficulty: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub scores: Vec<Score>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerStats {
    pub player_name: String,
    pub total_games: u32,
    pub highest_score: u32,
    pub average_score: f64,
    pub highest_level: u32,
    pub favorite_difficulty: String,
    pub scores_by_difficulty: DifficultyScores,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DifficultyScores {
    #[serde(rename = "Easy")]
    pub easy: u32,
    #[serde(rename = "Medium")]
    pub medium: u32,
    #[serde(rename = "Hard")]
    pub hard: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalStats {
    pub total_games_played: u32,
    pub total_players: u32,
    pub highest_score_ever: Option<Score>,
    pub average_score: f64,
    pub scores_by_difficulty: DifficultyScores,
    pub popular_difficulty: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}