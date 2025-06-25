use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub player_name: String,
    pub score: u32,
    pub level: u32,
    pub difficulty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
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

pub struct ApiClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:8080/api".to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }
    
    // 提交分数（非阻塞）
    pub fn submit_score_async(&self, player_name: String, score: u32, level: u32, difficulty: String) {
        let client = self.client.clone();
        let url = format!("{}/scores", self.base_url);
        
        // 在新线程中发送请求，避免阻塞游戏
        std::thread::spawn(move || {
            let request = CreateScoreRequest {
                player_name,
                score,
                level,
                difficulty,
            };
            
            match client.post(&url)
                .json(&request)
                .send() {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Score submitted successfully!");
                    } else {
                        eprintln!("Failed to submit score: {}", response.status());
                    }
                }
                Err(e) => {
                    eprintln!("Error submitting score: {}", e);
                }
            }
        });
    }
    
    // 获取排行榜（阻塞）
    pub fn get_leaderboard(&self, limit: Option<usize>, difficulty: Option<&str>) -> Result<LeaderboardResponse, Box<dyn Error>> {
        let mut url = format!("{}/scores", self.base_url);
        let mut params = Vec::new();
        
        if let Some(limit) = limit {
            params.push(format!("limit={}", limit));
        }
        
        if let Some(difficulty) = difficulty {
            params.push(format!("difficulty={}", difficulty));
        }
        
        if !params.is_empty() {
            url.push_str("?");
            url.push_str(&params.join("&"));
        }
        
        let response = self.client.get(&url).send()?;
        
        if response.status().is_success() {
            let leaderboard: LeaderboardResponse = response.json()?;
            Ok(leaderboard)
        } else {
            Err(format!("Failed to get leaderboard: {}", response.status()).into())
        }
    }
    
    // 测试连接
    pub fn test_connection(&self) -> bool {
        match self.client.get(&format!("{}/health", self.base_url)).send() {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}