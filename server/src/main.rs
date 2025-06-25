use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::sync::Arc;
use uuid::Uuid;

// 数据模型
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
    pub timestamp: String,
}

// 数据库模型
#[derive(Debug, FromRow)]
struct DbScore {
    id: String,
    player_name: String,
    score: i32,
    level: i32,
    difficulty: String,
    created_at: String,
}

// 查询参数
#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    limit: Option<usize>,
    offset: Option<usize>,
    difficulty: Option<String>,
}

// 应用状态
struct AppState {
    pool: SqlitePool,
}

// 数据库初始化
async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scores (
            id TEXT PRIMARY KEY,
            player_name TEXT NOT NULL,
            score INTEGER NOT NULL,
            level INTEGER NOT NULL,
            difficulty TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS idx_score ON scores(score DESC);
        CREATE INDEX IF NOT EXISTS idx_player ON scores(player_name);
        CREATE INDEX IF NOT EXISTS idx_difficulty ON scores(difficulty);
        "#,
    )
    .execute(pool)
    .await?;
    
    Ok(())
}
// API 处理函数

// 提交分数
async fn submit_score(
    data: web::Data<Arc<AppState>>,
    score_req: web::Json<CreateScoreRequest>,
) -> Result<HttpResponse> {
    // 验证输入
    if score_req.player_name.is_empty() || score_req.player_name.len() > 50 {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Invalid Input".to_string(),
            message: "Player name must be between 1 and 50 characters".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        }));
    }
    
    if !["Easy", "Medium", "Hard"].contains(&score_req.difficulty.as_str()) {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Invalid Input".to_string(),
            message: "Difficulty must be Easy, Medium, or Hard".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        }));
    }
    
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    
    let result = sqlx::query(
        r#"
        INSERT INTO scores (id, player_name, score, level, difficulty, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )
    .bind(&id)
    .bind(&score_req.player_name)
    .bind(score_req.score as i32)
    .bind(score_req.level as i32)
    .bind(&score_req.difficulty)
    .bind(&created_at)
    .execute(&data.pool)
    .await;
    
    match result {
        Ok(_) => {
            let score = Score {
                id: Some(id),
                player_name: score_req.player_name.clone(),
                score: score_req.score,
                level: score_req.level,
                difficulty: score_req.difficulty.clone(),
                created_at: Some(created_at),
                rank: None,
            };
            Ok(HttpResponse::Created().json(score))
        }
        Err(e) => {
            log::error!("Database error: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Database Error".to_string(),
                message: "Failed to save score".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            }))
        }
    }
}

// 获取排行榜
async fn get_leaderboard(
    data: web::Data<Arc<AppState>>,
    query: web::Query<LeaderboardQuery>,
) -> Result<HttpResponse> {
    let limit = query.limit.unwrap_or(10).min(100);
    let offset = query.offset.unwrap_or(0);
    
    // 构建查询
    let mut sql = "SELECT * FROM scores".to_string();
    let mut conditions = Vec::new();
    
    if let Some(ref difficulty) = query.difficulty {
        if ["Easy", "Medium", "Hard"].contains(&difficulty.as_str()) {
            conditions.push(format!("difficulty = '{}'", difficulty));
        }
    }
    
    if !conditions.is_empty() {
        sql.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
    }
    
    sql.push_str(" ORDER BY score DESC");
    sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));
    
    let scores: Vec<DbScore> = sqlx::query_as(&sql)
        .fetch_all(&data.pool)
        .await
        .map_err(|e| {
            log::error!("Database error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;
    
    // 计算总数
    let count_sql = if let Some(ref difficulty) = query.difficulty {
        format!("SELECT COUNT(*) FROM scores WHERE difficulty = '{}'", difficulty)
    } else {
        "SELECT COUNT(*) FROM scores".to_string()
    };
    
    let total: (i32,) = sqlx::query_as(&count_sql)
        .fetch_one(&data.pool)
        .await
        .unwrap_or((0,));
    
    // 转换为响应格式
    let mut response_scores = Vec::new();
    for (index, db_score) in scores.iter().enumerate() {
        response_scores.push(Score {
            id: Some(db_score.id.clone()),
            player_name: db_score.player_name.clone(),
            score: db_score.score as u32,
            level: db_score.level as u32,
            difficulty: db_score.difficulty.clone(),
            created_at: Some(db_score.created_at.clone()),
            rank: Some((offset + index + 1) as u32),
        });
    }
    
    Ok(HttpResponse::Ok().json(LeaderboardResponse {
        scores: response_scores,
        total: total.0 as usize,
        limit,
        offset,
    }))
}

// 获取玩家统计
async fn get_player_stats(
    data: web::Data<Arc<AppState>>,
    player_name: web::Path<String>,
) -> Result<HttpResponse> {
    let player_name = player_name.into_inner();
    
    // 检查玩家是否存在
    let exists: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM scores WHERE player_name = ?1"
    )
    .bind(&player_name)
    .fetch_one(&data.pool)
    .await
    .unwrap_or((0,));
    
    if exists.0 == 0 {
        return Ok(HttpResponse::NotFound().json(ErrorResponse {
            error: "Not Found".to_string(),
            message: format!("Player '{}' not found", player_name),
            timestamp: Utc::now().to_rfc3339(),
        }));
    }
    
    // 获取统计数据
    let stats: (i32, i32, f64, i32) = sqlx::query_as(
        r#"
        SELECT 
            COUNT(*) as total_games,
            MAX(score) as highest_score,
            AVG(score) as average_score,
            MAX(level) as highest_level
        FROM scores 
        WHERE player_name = ?1
        "#
    )
    .bind(&player_name)
    .fetch_one(&data.pool)
    .await
    .unwrap_or((0, 0, 0.0, 0));
    
    // 按难度统计
    let easy_count: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM scores WHERE player_name = ?1 AND difficulty = 'Easy'"
    )
    .bind(&player_name)
    .fetch_one(&data.pool)
    .await
    .unwrap_or((0,));
    
    let medium_count: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM scores WHERE player_name = ?1 AND difficulty = 'Medium'"
    )
    .bind(&player_name)
    .fetch_one(&data.pool)
    .await
    .unwrap_or((0,));
    
    let hard_count: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM scores WHERE player_name = ?1 AND difficulty = 'Hard'"
    )
    .bind(&player_name)
    .fetch_one(&data.pool)
    .await
    .unwrap_or((0,));
    
    // 确定最喜欢的难度
    let mut favorite_difficulty = "Medium".to_string();
    let max_count = easy_count.0.max(medium_count.0).max(hard_count.0);
    if max_count == easy_count.0 && easy_count.0 > 0 {
        favorite_difficulty = "Easy".to_string();
    } else if max_count == hard_count.0 && hard_count.0 > 0 {
        favorite_difficulty = "Hard".to_string();
    }
    
    Ok(HttpResponse::Ok().json(PlayerStats {
        player_name,
        total_games: stats.0 as u32,
        highest_score: stats.1 as u32,
        average_score: stats.2,
        highest_level: stats.3 as u32,
        favorite_difficulty,
        scores_by_difficulty: DifficultyScores {
            easy: easy_count.0 as u32,
            medium: medium_count.0 as u32,
            hard: hard_count.0 as u32,
        },
    }))
}
// 获取全局统计
async fn get_global_stats(
    data: web::Data<Arc<AppState>>,
) -> Result<HttpResponse> {
    // 总游戏数和平均分
    let game_stats: (i32, f64) = sqlx::query_as(
        "SELECT COUNT(*), AVG(score) FROM scores"
    )
    .fetch_one(&data.pool)
    .await
    .unwrap_or((0, 0.0));
    
    // 总玩家数
    let player_count: (i32,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT player_name) FROM scores"
    )
    .fetch_one(&data.pool)
    .await
    .unwrap_or((0,));
    
    // 最高分记录
    let highest_score: Option<DbScore> = sqlx::query_as(
        "SELECT * FROM scores ORDER BY score DESC LIMIT 1"
    )
    .fetch_optional(&data.pool)
    .await
    .unwrap_or(None);
    
    // 按难度统计
    let easy_count: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM scores WHERE difficulty = 'Easy'"
    )
    .fetch_one(&data.pool)
    .await
    .unwrap_or((0,));
    
    let medium_count: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM scores WHERE difficulty = 'Medium'"
    )
    .fetch_one(&data.pool)
    .await
    .unwrap_or((0,));
    
    let hard_count: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM scores WHERE difficulty = 'Hard'"
    )
    .fetch_one(&data.pool)
    .await
    .unwrap_or((0,));
    
    // 确定最受欢迎的难度
    let mut popular_difficulty = "Medium".to_string();
    let max_count = easy_count.0.max(medium_count.0).max(hard_count.0);
    if max_count == easy_count.0 && easy_count.0 > 0 {
        popular_difficulty = "Easy".to_string();
    } else if max_count == hard_count.0 && hard_count.0 > 0 {
        popular_difficulty = "Hard".to_string();
    }
    
    Ok(HttpResponse::Ok().json(GlobalStats {
        total_games_played: game_stats.0 as u32,
        total_players: player_count.0 as u32,
        highest_score_ever: highest_score.map(|db_score| Score {
            id: Some(db_score.id),
            player_name: db_score.player_name,
            score: db_score.score as u32,
            level: db_score.level as u32,
            difficulty: db_score.difficulty,
            created_at: Some(db_score.created_at),
            rank: Some(1),
        }),
        average_score: game_stats.1,
        scores_by_difficulty: DifficultyScores {
            easy: easy_count.0 as u32,
            medium: medium_count.0 as u32,
            hard: hard_count.0 as u32,
        },
        popular_difficulty,
    }))
}

// 删除分数（管理员功能）
async fn delete_score(
    data: web::Data<Arc<AppState>>,
    score_id: web::Path<String>,
) -> Result<HttpResponse> {
    let result = sqlx::query("DELETE FROM scores WHERE id = ?1")
        .bind(score_id.as_str())
        .execute(&data.pool)
        .await;
    
    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(HttpResponse::NoContent().finish())
            } else {
                Ok(HttpResponse::NotFound().json(ErrorResponse {
                    error: "Not Found".to_string(),
                    message: "Score not found".to_string(),
                    timestamp: Utc::now().to_rfc3339(),
                }))
            }
        }
        Err(e) => {
            log::error!("Database error: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Database Error".to_string(),
                message: "Failed to delete score".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            }))
        }
    }
}

// 健康检查
async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now().to_rfc3339()
    })))
}

// 配置路由
fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health_check))
            .route("/scores", web::post().to(submit_score))
            .route("/scores", web::get().to(get_leaderboard))
            .route("/scores/{id}", web::delete().to(delete_score))
            .route("/players/{player_name}/stats", web::get().to(get_player_stats))
            .route("/stats/global", web::get().to(get_global_stats))
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    log::info!("Starting Breakout Game API Server...");
    
    // 创建数据库文件（如果不存在）
    let db_path = "breakout_scores.db";
    if !std::path::Path::new(db_path).exists() {
        std::fs::File::create(db_path).expect("Failed to create database file");
    }
    
    // 使用完整的数据库 URL
    let database_url = format!("sqlite://{}", db_path);
    
    // 创建数据库连接池
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to create pool");
    
    // 初始化数据库
    init_db(&pool)
        .await
        .expect("Failed to initialize database");
    
    log::info!("Database initialized");
    
    let app_state = Arc::new(AppState { pool });
    
    log::info!("Starting HTTP server at http://localhost:8080");
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(config_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}