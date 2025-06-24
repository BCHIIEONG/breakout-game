use bevy::prelude::*;
use rand::prelude::*;

// 碰撞检测
#[derive(Debug)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

fn collide(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> Option<Collision> {
    let a_min = a_pos.xy() - a_size / 2.0;
    let a_max = a_pos.xy() + a_size / 2.0;
    let b_min = b_pos.xy() - b_size / 2.0;
    let b_max = b_pos.xy() + b_size / 2.0;

    // 检查是否有碰撞
    if a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y {
        // 计算重叠
        let left = b_max.x - a_min.x;
        let right = a_max.x - b_min.x;
        let top = b_max.y - a_min.y;
        let bottom = a_max.y - b_min.y;

        // 找出最小的重叠方向
        let min = left.min(right).min(top).min(bottom);

        if min == left {
            Some(Collision::Left)
        } else if min == right {
            Some(Collision::Right)
        } else if min == top {
            Some(Collision::Top)
        } else {
            Some(Collision::Bottom)
        }
    } else {
        None
    }
}

// 窗口设置
const WINDOW_WIDTH: f32 = 900.0;
const WINDOW_HEIGHT: f32 = 600.0;

// 挡板设置
const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
const PADDLE_SPEED: f32 = 500.0;
const PADDLE_Y: f32 = -250.0;

// 球设置
const BALL_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const BALL_SPEED: f32 = 400.0;

// 砖块设置
const BRICK_SIZE: Vec2 = Vec2::new(75.0, 30.0);
const BRICK_ROWS: usize = 6;
const BRICK_COLUMNS: usize = 10;
const GAP_SIZE: f32 = 5.0;

// 激光设置
const LASER_SIZE: Vec2 = Vec2::new(5.0, 20.0);
const LASER_SPEED: f32 = 600.0;

// 颜色定义
const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.15);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.7, 1.0);
const BALL_COLOR: Color = Color::rgb(1.0, 0.9, 0.7);
const NORMAL_BRICK_COLOR: Color = Color::rgb(0.8, 0.3, 0.3);
const HARD_BRICK_COLOR: Color = Color::rgb(0.5, 0.2, 0.2);
const UNBREAKABLE_BRICK_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);
const LASER_COLOR: Color = Color::rgb(1.0, 0.2, 0.2);

// 游戏状态
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    MainMenu,
    DifficultySelect,
    Playing,
    Paused,
    GameOver,
    Victory,
    NextLevel,
}

// 难度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

// 难度设置
#[derive(Resource)]
struct DifficultySettings {
    difficulty: Difficulty,
    lives: u32,
    ball_speed_modifier: f32,
    paddle_speed_modifier: f32,
    reset_lives_on_level: bool,
    time_limit: Option<f32>, // 困难模式的时间限制（秒）
}

impl DifficultySettings {
    fn new(difficulty: Difficulty) -> Self {
        match difficulty {
            Difficulty::Easy => Self {
                difficulty,
                lives: 5,
                ball_speed_modifier: 0.8,
                paddle_speed_modifier: 1.0,
                reset_lives_on_level: true,
                time_limit: None,
            },
            Difficulty::Medium => Self {
                difficulty,
                lives: 3,
                ball_speed_modifier: 1.0,
                paddle_speed_modifier: 1.20,  // 稍微加快挡板速度
                reset_lives_on_level: false,
                time_limit: None,
            },
            Difficulty::Hard => Self {
                difficulty,
                lives: 3,
                ball_speed_modifier: 1.3,
                paddle_speed_modifier: 1.8,   // 更快的挡板速度
                reset_lives_on_level: false,
                time_limit: Some(180.0), // 3分钟每关
            },
        }
    }
}

// 游戏初始化标记
#[derive(Resource)]
struct GameInitialized(bool);

// 组件定义
#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball {
    velocity: Vec2,
}

#[derive(Component)]
struct Brick {
    brick_type: BrickType,
    health: i32,
}

#[derive(Component, Clone, Copy)]
enum BrickType {
    Normal,
    Hard,
    Unbreakable,
}

#[derive(Component)]
struct PowerUp {
    power_type: PowerUpType,
    velocity: Vec2,
}

#[derive(Clone, Copy)]
enum PowerUpType {
    PaddleExpand,
    PaddleShrink,
    BallSpeedUp,
    BallSpeedDown,
    MultiBall,
    PenetratingBall,
    LaserGun,
}

#[derive(Component)]
struct Laser {
    velocity: Vec2,
}

#[derive(Component)]
struct Particle {
    velocity: Vec2,
    lifetime: f32,
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct LevelText;

#[derive(Component)]
struct LivesText;

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct LaserText;

#[derive(Component)]
struct MainMenuUI;

#[derive(Component)]
struct DifficultyUI;

#[derive(Component)]
struct GameOverUI;

#[derive(Component)]
struct VictoryUI;

#[derive(Component)]
struct PauseUI;

#[derive(Component)]
struct GameEntity;

// 资源定义
#[derive(Resource)]
struct Score(u32);

#[derive(Resource)]
struct Level(u32);

#[derive(Resource)]
struct Lives(u32);

#[derive(Resource)]
struct LevelTimer(f32);

#[derive(Resource)]
struct PowerUpEffects {
    paddle_size_modifier: f32,
    ball_speed_modifier: f32,
    penetrating_ball: bool,
    penetrating_timer: f32,
    has_laser: bool,
    laser_timer: f32,
}

impl Default for PowerUpEffects {
    fn default() -> Self {
        Self {
            paddle_size_modifier: 1.0,
            ball_speed_modifier: 1.0,
            penetrating_ball: false,
            penetrating_timer: 0.0,
            has_laser: false,
            laser_timer: 0.0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Super Breakout".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }).set(AssetPlugin {
            // 确保资源能正确加载
            ..default()
        }))
        .init_state::<GameState>()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Score(0))
        .insert_resource(Level(1))
        .insert_resource(Lives(3))
        .insert_resource(LevelTimer(0.0))
        .insert_resource(PowerUpEffects::default())
        .insert_resource(DifficultySettings::new(Difficulty::Medium))
        .insert_resource(GameInitialized(false))
        // 菜单系统
        .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
        .add_systems(Update, main_menu_system.run_if(in_state(GameState::MainMenu)))
        .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
        // 难度选择系统
        .add_systems(OnEnter(GameState::DifficultySelect), setup_difficulty_menu)
        .add_systems(Update, difficulty_menu_system.run_if(in_state(GameState::DifficultySelect)))
        .add_systems(OnExit(GameState::DifficultySelect), cleanup_difficulty_menu)
        // 游戏系统
        .add_systems(OnEnter(GameState::Playing), setup_game_conditional)
        .add_systems(
            Update,
            (
                paddle_movement,
                ball_movement,
                ball_collision,
                powerup_movement,
                powerup_collision,
                particle_system,
                update_powerup_timers,
                update_level_timer,
                check_victory,
                update_ui,
                pause_game_input,
                laser_shooting,
                laser_movement,
                laser_collision,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // 暂停系统
        .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
        .add_systems(Update, pause_menu_system.run_if(in_state(GameState::Paused)))
        .add_systems(OnExit(GameState::Paused), cleanup_pause_menu)
        // 游戏结束系统
        .add_systems(OnEnter(GameState::GameOver), (cleanup_game, setup_game_over))
        .add_systems(Update, game_over_system.run_if(in_state(GameState::GameOver)))
        .add_systems(OnExit(GameState::GameOver), cleanup_game_over)
        // 胜利系统
        .add_systems(OnEnter(GameState::Victory), setup_victory)
        .add_systems(Update, victory_system.run_if(in_state(GameState::Victory)))
        .add_systems(OnExit(GameState::Victory), cleanup_victory)
        // 下一关系统
        .add_systems(OnEnter(GameState::NextLevel), (cleanup_game, next_level_setup))
        .run();
}

// 设置主菜单
fn setup_main_menu(mut commands: Commands, mut game_initialized: ResMut<GameInitialized>) {
    game_initialized.0 = false;
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgb(0.1, 0.1, 0.15)),
                ..default()
            },
            MainMenuUI,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "SUPER BREAKOUT",
                TextStyle {
                    font_size: 80.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            
            parent.spawn(TextBundle::from_section(
                "Press SPACE to Start",
                TextStyle {
                    font_size: 30.0,
                    color: Color::rgb(0.7, 0.7, 0.7),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(50.0)),
                ..default()
            }));

            parent.spawn(TextBundle::from_section(
                "Controls:\nArrow Keys or A/D: Move paddle\nSPACE: Shoot laser (when available)\nESC: Pause game\nCollect power-ups for special abilities",
                TextStyle {
                    font_size: 20.0,
                    color: Color::rgb(0.6, 0.6, 0.6),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(100.0)),
                ..default()
            }));
        });
}

// 主菜单系统
fn main_menu_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::DifficultySelect);
    }
}

// 清理主菜单
fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// 设置难度选择菜单
fn setup_difficulty_menu(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgb(0.1, 0.1, 0.15)),
                ..default()
            },
            DifficultyUI,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "SELECT DIFFICULTY",
                TextStyle {
                    font_size: 60.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            
            parent.spawn(TextBundle::from_section(
                "[1] EASY - 5 Lives, Slower Ball, Lives Reset Each Level",
                TextStyle {
                    font_size: 25.0,
                    color: Color::rgb(0.2, 0.8, 0.2),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(50.0)),
                ..default()
            }));

            parent.spawn(TextBundle::from_section(
                "[2] MEDIUM - 3 Lives, Normal Ball, Faster Paddle",
                TextStyle {
                    font_size: 25.0,
                    color: Color::rgb(0.8, 0.8, 0.2),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            }));

            parent.spawn(TextBundle::from_section(
                "[3] HARD - 3 Lives, Very Fast Ball & Paddle, Time Limit",
                TextStyle {
                    font_size: 25.0,
                    color: Color::rgb(0.8, 0.2, 0.2),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            }));

            parent.spawn(TextBundle::from_section(
                "Press 1, 2, or 3 to select",
                TextStyle {
                    font_size: 20.0,
                    color: Color::rgb(0.6, 0.6, 0.6),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(50.0)),
                ..default()
            }));
        });
}

// 难度选择系统
fn difficulty_menu_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut difficulty_settings: ResMut<DifficultySettings>,
    mut lives: ResMut<Lives>,
) {
    if keyboard_input.just_pressed(KeyCode::Digit1) || keyboard_input.just_pressed(KeyCode::Numpad1) {
        *difficulty_settings = DifficultySettings::new(Difficulty::Easy);
        lives.0 = difficulty_settings.lives;
        next_state.set(GameState::Playing);
    } else if keyboard_input.just_pressed(KeyCode::Digit2) || keyboard_input.just_pressed(KeyCode::Numpad2) {
        *difficulty_settings = DifficultySettings::new(Difficulty::Medium);
        lives.0 = difficulty_settings.lives;
        next_state.set(GameState::Playing);
    } else if keyboard_input.just_pressed(KeyCode::Digit3) || keyboard_input.just_pressed(KeyCode::Numpad3) {
        *difficulty_settings = DifficultySettings::new(Difficulty::Hard);
        lives.0 = difficulty_settings.lives;
        next_state.set(GameState::Playing);
    }
}

// 清理难度选择菜单
fn cleanup_difficulty_menu(mut commands: Commands, query: Query<Entity, With<DifficultyUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// 条件性设置游戏
fn setup_game_conditional(
    commands: Commands,
    score: ResMut<Score>,
    lives: ResMut<Lives>,
    level_timer: ResMut<LevelTimer>,
    level: Res<Level>,
    difficulty_settings: Res<DifficultySettings>,
    mut game_initialized: ResMut<GameInitialized>,
) {
    if !game_initialized.0 {
        setup_game(commands, score, lives, level_timer, level, difficulty_settings);
        game_initialized.0 = true;
    }
}

// 设置游戏
fn setup_game(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut level_timer: ResMut<LevelTimer>,
    level: Res<Level>,
    difficulty_settings: Res<DifficultySettings>,
) {
    // 重置分数和生命（新游戏时）
    if level.0 == 1 {
        score.0 = 0;
        lives.0 = difficulty_settings.lives;
    } else if difficulty_settings.reset_lives_on_level {
        // Easy模式下每关重置生命
        lives.0 = difficulty_settings.lives;
    }

    // 重置计时器
    if let Some(time_limit) = difficulty_settings.time_limit {
        level_timer.0 = time_limit;
    }

    // 创建相机
    commands.spawn((Camera2dBundle::default(), GameEntity));

    // 创建挡板
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, PADDLE_Y, 0.0),
                scale: Vec3::new(PADDLE_SIZE.x, PADDLE_SIZE.y, 1.0),
                ..default()
            },
            ..default()
        },
        Paddle,
        GameEntity,
    ));

    // 创建球
    let mut rng = rand::thread_rng();
    let ball_direction = Vec2::new(
        if rng.gen_bool(0.5) { 1.0 } else { -1.0 },
        1.0,
    ).normalize();

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: BALL_COLOR,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, -200.0, 0.0),
                scale: Vec3::new(BALL_SIZE.x, BALL_SIZE.y, 1.0),
                ..default()
            },
            ..default()
        },
        Ball {
            velocity: ball_direction * BALL_SPEED * difficulty_settings.ball_speed_modifier,
        },
        GameEntity,
    ));

    // 创建砖块
    spawn_bricks(&mut commands, level.0);

    // UI
    setup_ui(&mut commands, &difficulty_settings);
}

// 生成砖块
fn spawn_bricks(commands: &mut Commands, level: u32) {
    let mut rng = rand::thread_rng();
    let total_width = BRICK_COLUMNS as f32 * (BRICK_SIZE.x + GAP_SIZE) - GAP_SIZE;
    let start_x = -total_width / 2.0 + BRICK_SIZE.x / 2.0;
    let start_y = 200.0;

    for row in 0..BRICK_ROWS {
        for col in 0..BRICK_COLUMNS {
            let x = start_x + col as f32 * (BRICK_SIZE.x + GAP_SIZE);
            let y = start_y - row as f32 * (BRICK_SIZE.y + GAP_SIZE);

            // 根据关卡生成不同类型的砖块
            let (brick_type, color, health) = match level {
                1 => {
                    // 第一关：大部分普通砖块
                    if rng.gen_range(0..100) < 10 {
                        (BrickType::Hard, HARD_BRICK_COLOR, 2)
                    } else {
                        (BrickType::Normal, NORMAL_BRICK_COLOR, 1)
                    }
                }
                2 => {
                    // 第二关：混合砖块
                    let rand_val = rng.gen_range(0..100);
                    if rand_val < 5 {
                        (BrickType::Unbreakable, UNBREAKABLE_BRICK_COLOR, -1)
                    } else if rand_val < 30 {
                        (BrickType::Hard, HARD_BRICK_COLOR, 2)
                    } else {
                        (BrickType::Normal, NORMAL_BRICK_COLOR, 1)
                    }
                }
                _ => {
                    // 第三关及以后：更多困难砖块
                    let rand_val = rng.gen_range(0..100);
                    if rand_val < 10 {
                        (BrickType::Unbreakable, UNBREAKABLE_BRICK_COLOR, -1)
                    } else if rand_val < 50 {
                        (BrickType::Hard, HARD_BRICK_COLOR, 3)
                    } else {
                        (BrickType::Normal, NORMAL_BRICK_COLOR, 1)
                    }
                }
            };

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(x, y, 0.0),
                        scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0),
                        ..default()
                    },
                    ..default()
                },
                Brick { brick_type, health },
                GameEntity,
            ));
        }
    }
}

// 设置UI
fn setup_ui(commands: &mut Commands, difficulty_settings: &DifficultySettings) {
    // 分数文本
    commands.spawn((
        TextBundle::from_section(
            "Score: 0",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        }),
        ScoreText,
        GameEntity,
    ));

    // 关卡文本
    commands.spawn((
        TextBundle::from_section(
            "Level: 1",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(WINDOW_WIDTH / 2.0 - 50.0),
            top: Val::Px(10.0),
            ..default()
        }),
        LevelText,
        GameEntity,
    ));

    // 生命文本
    commands.spawn((
        TextBundle::from_section(
            "Lives: 3",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        }),
        LivesText,
        GameEntity,
    ));

    // 如果是困难模式，添加计时器文本
    if difficulty_settings.difficulty == Difficulty::Hard {
        commands.spawn((
            TextBundle::from_section(
                "Time: 180",
                TextStyle {
                    font_size: 30.0,
                    color: Color::rgb(0.8, 0.2, 0.2),
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                left: Val::Px(WINDOW_WIDTH / 2.0 - 50.0),
                top: Val::Px(50.0),
                ..default()
            }),
            TimerText,
            GameEntity,
        ));
    }

    // 激光状态文本
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 25.0,
                color: Color::rgb(0.2, 0.8, 0.8),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            bottom: Val::Px(10.0),
            ..default()
        }),
        LaserText,
        GameEntity,
    ));
}

// 更新UI
fn update_ui(
    score: Res<Score>,
    level: Res<Level>,
    lives: Res<Lives>,
    level_timer: Res<LevelTimer>,
    power_effects: Res<PowerUpEffects>,
    difficulty_settings: Res<DifficultySettings>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<LevelText>, Without<LivesText>, Without<TimerText>, Without<LaserText>)>,
    mut level_query: Query<&mut Text, (With<LevelText>, Without<ScoreText>, Without<LivesText>, Without<TimerText>, Without<LaserText>)>,
    mut lives_query: Query<&mut Text, (With<LivesText>, Without<ScoreText>, Without<LevelText>, Without<TimerText>, Without<LaserText>)>,
    mut timer_query: Query<&mut Text, (With<TimerText>, Without<ScoreText>, Without<LevelText>, Without<LivesText>, Without<LaserText>)>,
    mut laser_query: Query<&mut Text, (With<LaserText>, Without<ScoreText>, Without<LevelText>, Without<LivesText>, Without<TimerText>)>,
) {
    if let Ok(mut text) = score_query.get_single_mut() {
        text.sections[0].value = format!("Score: {}", score.0);
    }
    if let Ok(mut text) = level_query.get_single_mut() {
        text.sections[0].value = format!("Level: {}", level.0);
    }
    if let Ok(mut text) = lives_query.get_single_mut() {
        text.sections[0].value = format!("Lives: {}", lives.0);
    }
    
    // 更新计时器文本（仅限困难模式）
    if difficulty_settings.difficulty == Difficulty::Hard {
        if let Ok(mut text) = timer_query.get_single_mut() {
            text.sections[0].value = format!("Time: {}", level_timer.0.ceil() as i32);
        }
    }

    // 更新激光状态文本
    if let Ok(mut text) = laser_query.get_single_mut() {
        if power_effects.has_laser {
            text.sections[0].value = format!("LASER: {:.1}s", power_effects.laser_timer);
        } else {
            text.sections[0].value = String::new();
        }
    }
}
// 更新关卡计时器
fn update_level_timer(
    time: Res<Time>,
    mut level_timer: ResMut<LevelTimer>,
    difficulty_settings: Res<DifficultySettings>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if difficulty_settings.difficulty == Difficulty::Hard {
        if level_timer.0 > 0.0 {
            level_timer.0 -= time.delta_seconds();
            if level_timer.0 <= 0.0 {
                level_timer.0 = 0.0;
                next_state.set(GameState::GameOver);
            }
        }
    }
}

// 挡板移动
fn paddle_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paddle_query: Query<&mut Transform, With<Paddle>>,
    time: Res<Time>,
    power_effects: Res<PowerUpEffects>,
    difficulty_settings: Res<DifficultySettings>,
) {
    if let Ok(mut transform) = paddle_query.get_single_mut() {
        let mut direction = 0.0;

        if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
            direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
            direction += 1.0;
        }

        let paddle_width = PADDLE_SIZE.x * power_effects.paddle_size_modifier;
        let half_paddle = paddle_width / 2.0;
        let boundary = WINDOW_WIDTH / 2.0 - half_paddle;

        transform.translation.x += direction * PADDLE_SPEED * difficulty_settings.paddle_speed_modifier * time.delta_seconds();
        transform.translation.x = transform.translation.x.clamp(-boundary, boundary);
        transform.scale.x = paddle_width;
    }
}

// 激光射击系统
fn laser_shooting(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    power_effects: Res<PowerUpEffects>,
    paddle_query: Query<&Transform, With<Paddle>>,
) {
    if power_effects.has_laser && keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(paddle_transform) = paddle_query.get_single() {
            let paddle_width = PADDLE_SIZE.x * power_effects.paddle_size_modifier;
            
            // 从挡板两端发射激光
            for offset in [-paddle_width / 3.0, paddle_width / 3.0] {
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: LASER_COLOR,
                            ..default()
                        },
                        transform: Transform {
                            translation: Vec3::new(
                                paddle_transform.translation.x + offset,
                                paddle_transform.translation.y + PADDLE_SIZE.y,
                                0.0,
                            ),
                            scale: Vec3::new(LASER_SIZE.x, LASER_SIZE.y, 1.0),
                            ..default()
                        },
                        ..default()
                    },
                    Laser {
                        velocity: Vec2::new(0.0, LASER_SPEED),
                    },
                    GameEntity,
                ));
            }
        }
    }
}

// 激光移动系统
fn laser_movement(
    mut commands: Commands,
    mut lasers: Query<(Entity, &mut Transform, &Laser)>,
    time: Res<Time>,
) {
    for (entity, mut transform, laser) in lasers.iter_mut() {
        transform.translation += laser.velocity.extend(0.0) * time.delta_seconds();
        
        // 如果激光超出屏幕顶部，删除它
        if transform.translation.y > WINDOW_HEIGHT / 2.0 + 50.0 {
            commands.entity(entity).despawn();
        }
    }
}

// 激光碰撞系统
fn laser_collision(
    mut commands: Commands,
    lasers: Query<(Entity, &Transform), With<Laser>>,
    mut bricks: Query<(Entity, &Transform, &mut Brick, &mut Sprite), Without<Laser>>,
    mut score: ResMut<Score>,
) {
    for (laser_entity, laser_transform) in lasers.iter() {
        for (brick_entity, brick_transform, mut brick, mut sprite) in bricks.iter_mut() {
            if let Some(_) = collide(
                laser_transform.translation,
                LASER_SIZE,
                brick_transform.translation,
                BRICK_SIZE,
            ) {
                // 激光不能破坏不可破坏的砖块
                if matches!(brick.brick_type, BrickType::Unbreakable) {
                    commands.entity(laser_entity).despawn();
                    break;
                }

                // 激光造成额外伤害
                brick.health -= 2;

                if brick.health <= 0 {
                    // 销毁砖块
                    commands.entity(brick_entity).despawn();
                    
                    // 增加分数
                    match brick.brick_type {
                        BrickType::Normal => score.0 += 15, // 激光破坏获得更多分数
                        BrickType::Hard => score.0 += 30,
                        _ => {}
                    }

                    // 生成粒子效果
                    spawn_particles(&mut commands, brick_transform.translation, brick_transform.scale);
                } else {
                    // 更新砖块颜色表示受损
                    sprite.color = Color::rgb(
                        sprite.color.r() * 0.6,
                        sprite.color.g() * 0.6,
                        sprite.color.b() * 0.6,
                    );
                }

                // 激光击中后消失
                commands.entity(laser_entity).despawn();
                break;
            }
        }
    }
}

// 球移动
fn ball_movement(
    mut ball_query: Query<(&mut Transform, &Ball)>,
    time: Res<Time>,
    power_effects: Res<PowerUpEffects>,
    difficulty_settings: Res<DifficultySettings>,
) {
    for (mut transform, ball) in ball_query.iter_mut() {
        let velocity = ball.velocity * power_effects.ball_speed_modifier * difficulty_settings.ball_speed_modifier;
        transform.translation += velocity.extend(0.0) * time.delta_seconds();
    }
}

// 球碰撞检测
fn ball_collision(
    mut commands: Commands,
    mut ball_query: Query<(Entity, &mut Transform, &mut Ball)>,
    paddle_query: Query<&Transform, (With<Paddle>, Without<Ball>)>,
    mut brick_query: Query<(Entity, &Transform, &mut Brick, &mut Sprite), Without<Ball>>,
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut next_state: ResMut<NextState<GameState>>,
    power_effects: Res<PowerUpEffects>,
    difficulty_settings: Res<DifficultySettings>,
) {
    // 安全获取挡板
    let paddle_result = paddle_query.get_single();
    if paddle_result.is_err() {
        return; // 如果没有挡板，直接返回
    }
    let paddle_transform = paddle_result.unwrap();
    let paddle_width = PADDLE_SIZE.x * power_effects.paddle_size_modifier;
    
    let total_balls = ball_query.iter().count();
    let mut balls_to_remove = Vec::new();

    for (ball_entity, mut ball_transform, mut ball) in ball_query.iter_mut() {
        // 墙壁碰撞
        let half_width = WINDOW_WIDTH / 2.0;
        let half_height = WINDOW_HEIGHT / 2.0;

        if ball_transform.translation.x < -half_width + BALL_SIZE.x / 2.0 {
            ball_transform.translation.x = -half_width + BALL_SIZE.x / 2.0;
            ball.velocity.x = ball.velocity.x.abs();
        } else if ball_transform.translation.x > half_width - BALL_SIZE.x / 2.0 {
            ball_transform.translation.x = half_width - BALL_SIZE.x / 2.0;
            ball.velocity.x = -ball.velocity.x.abs();
        }

        if ball_transform.translation.y > half_height - BALL_SIZE.y / 2.0 {
            ball_transform.translation.y = half_height - BALL_SIZE.y / 2.0;
            ball.velocity.y = -ball.velocity.y.abs();
        }

        // 底部边界
        if ball_transform.translation.y < -half_height {
            if total_balls > 1 {
                // 如果还有其他球，只删除这个球
                balls_to_remove.push(ball_entity);
            } else {
                // 这是最后一个球
                if lives.0 == 1 {
                    // 最后一条命，直接游戏结束
                    next_state.set(GameState::GameOver);
                } else {
                    // 还有生命，扣除一条并重置
                    lives.0 = lives.0.saturating_sub(1);
                    // 重置球位置
                    ball_transform.translation = Vec3::new(0.0, -200.0, 0.0);
                    ball.velocity = Vec2::new(
                        if rand::random() { 1.0 } else { -1.0 },
                        1.0,
                    ).normalize() * BALL_SPEED * difficulty_settings.ball_speed_modifier;
                }
            }
        }

        // 挡板碰撞
        if let Some(collision) = collide(
            ball_transform.translation,
            BALL_SIZE,
            paddle_transform.translation,
            Vec2::new(paddle_width, PADDLE_SIZE.y),
        ) {
            match collision {
                Collision::Left | Collision::Right => {
                    ball.velocity.x = -ball.velocity.x;
                }
                Collision::Top | Collision::Bottom => {
                    ball.velocity.y = ball.velocity.y.abs();
                    
                    // 根据击中位置调整球的横向速度
                    let hit_position = (ball_transform.translation.x - paddle_transform.translation.x) 
                        / (paddle_width / 2.0);
                    ball.velocity.x = hit_position * BALL_SPEED * 0.75;
                }
            }
        }

        // 砖块碰撞
        for (brick_entity, brick_transform, mut brick, mut sprite) in brick_query.iter_mut() {
            if let Some(collision) = collide(
                ball_transform.translation,
                BALL_SIZE,
                brick_transform.translation,
                BRICK_SIZE,
            ) {
                // 不可破坏砖块
                if matches!(brick.brick_type, BrickType::Unbreakable) {
                    match collision {
                        Collision::Left | Collision::Right => {
                            ball.velocity.x = -ball.velocity.x;
                        }
                        Collision::Top | Collision::Bottom => {
                            ball.velocity.y = -ball.velocity.y;
                        }
                    }
                    continue;
                }

                // 穿透球效果
                if !power_effects.penetrating_ball {
                    match collision {
                        Collision::Left | Collision::Right => {
                            ball.velocity.x = -ball.velocity.x;
                        }
                        Collision::Top | Collision::Bottom => {
                            ball.velocity.y = -ball.velocity.y;
                        }
                    }
                }

                // 减少砖块生命值
                brick.health -= 1;

                if brick.health <= 0 {
                    // 销毁砖块
                    commands.entity(brick_entity).despawn();
                    
                    // 增加分数
                    match brick.brick_type {
                        BrickType::Normal => score.0 += 10,
                        BrickType::Hard => score.0 += 20,
                        _ => {}
                    }

                    // 生成粒子效果
                    spawn_particles(&mut commands, brick_transform.translation, brick_transform.scale);

                    // 概率生成道具
                    if rand::thread_rng().gen_bool(0.2) {
                        spawn_powerup(&mut commands, brick_transform.translation);
                    }
                } else {
                    // 更新砖块颜色表示受损
                    sprite.color = Color::rgb(
                        sprite.color.r() * 0.8,
                        sprite.color.g() * 0.8,
                        sprite.color.b() * 0.8,
                    );
                }

                break;
            }
        }
    }
    
    // 删除需要移除的球
    for entity in balls_to_remove {
        commands.entity(entity).despawn();
    }
}

// 生成粒子效果
fn spawn_particles(commands: &mut Commands, position: Vec3, scale: Vec3) {
    let mut rng = rand::thread_rng();
    
    for _ in 0..10 {
        let velocity = Vec2::new(
            rng.gen_range(-200.0..200.0),
            rng.gen_range(-200.0..200.0),
        );
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(
                        rng.gen_range(0.5..1.0),
                        rng.gen_range(0.5..1.0),
                        rng.gen_range(0.5..1.0),
                    ),
                    ..default()
                },
                transform: Transform {
                    translation: position,
                    scale: scale * 0.2,
                    ..default()
                },
                ..default()
            },
            Particle {
                velocity,
                lifetime: 1.0,
            },
            GameEntity,
        ));
    }
}

// 粒子系统更新
fn particle_system(
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Transform, &mut Particle, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle, mut sprite) in particles.iter_mut() {
        particle.lifetime -= time.delta_seconds();
        
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            transform.translation += particle.velocity.extend(0.0) * time.delta_seconds();
            transform.scale *= 0.95;
            sprite.color.set_a(particle.lifetime);
        }
    }
}

// 生成道具
fn spawn_powerup(commands: &mut Commands, position: Vec3) {
    let mut rng = rand::thread_rng();
    
    let power_type = match rng.gen_range(0..7) {
        0 => PowerUpType::PaddleExpand,
        1 => PowerUpType::PaddleShrink,
        2 => PowerUpType::BallSpeedUp,
        3 => PowerUpType::BallSpeedDown,
        4 => PowerUpType::MultiBall,
        5 => PowerUpType::PenetratingBall,
        _ => PowerUpType::LaserGun,
    };

    let color = match power_type {
        PowerUpType::PaddleExpand => Color::rgb(0.2, 0.8, 0.2),
        PowerUpType::PaddleShrink => Color::rgb(0.8, 0.2, 0.2),
        PowerUpType::BallSpeedUp => Color::rgb(0.8, 0.8, 0.2),
        PowerUpType::BallSpeedDown => Color::rgb(0.2, 0.2, 0.8),
        PowerUpType::MultiBall => Color::rgb(0.8, 0.2, 0.8),
        PowerUpType::PenetratingBall => Color::rgb(0.8, 0.5, 0.2),
        PowerUpType::LaserGun => Color::rgb(0.2, 0.8, 0.8),
    };

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                ..default()
            },
            transform: Transform {
                translation: position,
                scale: Vec3::new(30.0, 15.0, 1.0),
                ..default()
            },
            ..default()
        },
        PowerUp {
            power_type,
            velocity: Vec2::new(0.0, -150.0),
        },
        GameEntity,
    ));
}

// 道具移动
fn powerup_movement(
    mut commands: Commands,
    mut powerups: Query<(Entity, &mut Transform, &PowerUp)>,
    time: Res<Time>,
) {
    for (entity, mut transform, powerup) in powerups.iter_mut() {
        transform.translation += powerup.velocity.extend(0.0) * time.delta_seconds();
        
        // 移出屏幕后删除
        if transform.translation.y < -WINDOW_HEIGHT / 2.0 - 50.0 {
            commands.entity(entity).despawn();
        }
    }
}

// 道具碰撞
fn powerup_collision(
    mut commands: Commands,
    powerups: Query<(Entity, &Transform, &PowerUp)>,
    paddle_query: Query<&Transform, With<Paddle>>,
    mut power_effects: ResMut<PowerUpEffects>,
    ball_query: Query<(&Transform, &Ball)>,
) {
    // 安全获取挡板
    let paddle_result = paddle_query.get_single();
    if paddle_result.is_err() {
        return; // 如果没有挡板，直接返回
    }
    let paddle_transform = paddle_result.unwrap();
    let paddle_width = PADDLE_SIZE.x * power_effects.paddle_size_modifier;

    for (powerup_entity, powerup_transform, powerup) in powerups.iter() {
        if collide(
            powerup_transform.translation,
            Vec2::new(30.0, 15.0),
            paddle_transform.translation,
            Vec2::new(paddle_width, PADDLE_SIZE.y),
        ).is_some() {
            // 应用道具效果
            match powerup.power_type {
                PowerUpType::PaddleExpand => {
                    power_effects.paddle_size_modifier = (power_effects.paddle_size_modifier * 1.5).min(2.5);
                }
                PowerUpType::PaddleShrink => {
                    power_effects.paddle_size_modifier = (power_effects.paddle_size_modifier * 0.7).max(0.5);
                }
                PowerUpType::BallSpeedUp => {
                    power_effects.ball_speed_modifier = (power_effects.ball_speed_modifier * 1.3).min(2.0);
                }
                PowerUpType::BallSpeedDown => {
                    power_effects.ball_speed_modifier = (power_effects.ball_speed_modifier * 0.7).max(0.5);
                }
                PowerUpType::MultiBall => {
                    // 生成额外的球
                    if let Ok((ball_transform, ball)) = ball_query.get_single() {
                        for i in 0..2 {
                            let angle = (i as f32 - 0.5) * 0.5;
                            let new_velocity = Vec2::new(
                                ball.velocity.x * angle.cos() - ball.velocity.y * angle.sin(),
                                ball.velocity.x * angle.sin() + ball.velocity.y * angle.cos(),
                            );
                            
                            commands.spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: BALL_COLOR,
                                        ..default()
                                    },
                                    transform: Transform {
                                        translation: ball_transform.translation,
                                        scale: Vec3::new(BALL_SIZE.x, BALL_SIZE.y, 1.0),
                                        ..default()
                                    },
                                    ..default()
                                },
                                Ball { velocity: new_velocity },
                                GameEntity,
                            ));
                        }
                    }
                }
                PowerUpType::PenetratingBall => {
                    power_effects.penetrating_ball = true;
                    power_effects.penetrating_timer = 10.0;
                }
                PowerUpType::LaserGun => {
                    power_effects.has_laser = true;
                    power_effects.laser_timer = 15.0;
                }
            }

            commands.entity(powerup_entity).despawn();
        }
    }
}

// 更新道具计时器
fn update_powerup_timers(
    mut power_effects: ResMut<PowerUpEffects>,
    time: Res<Time>,
) {
    if power_effects.penetrating_ball {
        power_effects.penetrating_timer -= time.delta_seconds();
        if power_effects.penetrating_timer <= 0.0 {
            power_effects.penetrating_ball = false;
        }
    }

    if power_effects.has_laser {
        power_effects.laser_timer -= time.delta_seconds();
        if power_effects.laser_timer <= 0.0 {
            power_effects.has_laser = false;
        }
    }
}

// 检查胜利条件
fn check_victory(
    bricks: Query<&Brick>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let has_breakable_bricks = bricks.iter().any(|brick| 
        !matches!(brick.brick_type, BrickType::Unbreakable)
    );

    if !has_breakable_bricks {
        next_state.set(GameState::Victory);
    }
}

// 清理游戏
fn cleanup_game(
    mut commands: Commands,
    entities: Query<Entity, With<GameEntity>>,
    mut game_initialized: ResMut<GameInitialized>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
    game_initialized.0 = false;
}

// 游戏结束界面
fn setup_game_over(mut commands: Commands, score: Res<Score>, difficulty_settings: Res<DifficultySettings>) {
    let difficulty_text = match difficulty_settings.difficulty {
        Difficulty::Easy => "EASY",
        Difficulty::Medium => "MEDIUM",
        Difficulty::Hard => "HARD",
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.8)),
                ..default()
            },
            GameOverUI,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "GAME OVER",
                TextStyle {
                    font_size: 60.0,
                    color: Color::rgb(0.8, 0.2, 0.2),
                    ..default()
                },
            ));
            
            parent.spawn(TextBundle::from_section(
                format!("Final Score: {} ({})", score.0, difficulty_text),
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(30.0)),
                ..default()
            }));

            parent.spawn(TextBundle::from_section(
                "Press SPACE to return to menu",
                TextStyle {
                    font_size: 25.0,
                    color: Color::rgb(0.7, 0.7, 0.7),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(50.0)),
                ..default()
            }));
        });
}

// 游戏结束系统
fn game_over_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut level: ResMut<Level>,
    mut power_effects: ResMut<PowerUpEffects>,
    mut lives: ResMut<Lives>,
    difficulty_settings: Res<DifficultySettings>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        level.0 = 1;
        // 重置道具效果，避免影响下一局游戏
        *power_effects = PowerUpEffects::default();
        // 重置生命值
        lives.0 = difficulty_settings.lives;
        next_state.set(GameState::MainMenu);
    }
}

// 清理游戏结束界面
fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// 胜利界面
fn setup_victory(mut commands: Commands, score: Res<Score>, level: Res<Level>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.8)),
                ..default()
            },
            VictoryUI,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "LEVEL COMPLETE!",
                TextStyle {
                    font_size: 60.0,
                    color: Color::rgb(0.2, 0.8, 0.2),
                    ..default()
                },
            ));
            
            parent.spawn(TextBundle::from_section(
                format!("Current Score: {}", score.0),
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(30.0)),
                ..default()
            }));

            parent.spawn(TextBundle::from_section(
                format!("Level {} Completed", level.0),
                TextStyle {
                    font_size: 30.0,
                    color: Color::rgb(0.8, 0.8, 0.2),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            }));

            parent.spawn(TextBundle::from_section(
                "Press SPACE for next level",
                TextStyle {
                    font_size: 25.0,
                    color: Color::rgb(0.7, 0.7, 0.7),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(50.0)),
                ..default()
            }));
        });
}

// 胜利系统
fn victory_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::NextLevel);
    }
}

// 清理胜利界面
fn cleanup_victory(mut commands: Commands, query: Query<Entity, With<VictoryUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// 下一关设置
fn next_level_setup(
    mut level: ResMut<Level>,
    mut next_state: ResMut<NextState<GameState>>,
    mut power_effects: ResMut<PowerUpEffects>,
    mut game_initialized: ResMut<GameInitialized>,
) {
    level.0 += 1;
    *power_effects = PowerUpEffects::default();
    game_initialized.0 = false;  // 重置初始化状态
    next_state.set(GameState::Playing);
}

// 暂停游戏输入检测
fn pause_game_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}

// 设置暂停菜单
fn setup_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.7)),
                z_index: ZIndex::Global(100),
                ..default()
            },
            PauseUI,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "PAUSED",
                TextStyle {
                    font_size: 80.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            parent.spawn(TextBundle::from_section(
                "[R] Resume Game",
                TextStyle {
                    font_size: 30.0,
                    color: Color::rgb(0.2, 0.8, 0.2),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(50.0)),
                ..default()
            }));

            parent.spawn(TextBundle::from_section(
                "[N] New Game",
                TextStyle {
                    font_size: 30.0,
                    color: Color::rgb(0.8, 0.8, 0.2),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            }));

            parent.spawn(TextBundle::from_section(
                "[M] Main Menu",
                TextStyle {
                    font_size: 30.0,
                    color: Color::rgb(0.8, 0.2, 0.2),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            }));

            parent.spawn(TextBundle::from_section(
                "Press ESC to resume",
                TextStyle {
                    font_size: 20.0,
                    color: Color::rgb(0.6, 0.6, 0.6),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(50.0)),
                ..default()
            }));
        });
}

// 暂停菜单系统
fn pause_menu_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut level: ResMut<Level>,
    mut score: ResMut<Score>,
    mut lives: ResMut<Lives>,
    mut power_effects: ResMut<PowerUpEffects>,
    difficulty_settings: Res<DifficultySettings>,
    mut commands: Commands,
    game_entities: Query<Entity, With<GameEntity>>,
    mut game_initialized: ResMut<GameInitialized>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) || keyboard_input.just_pressed(KeyCode::KeyR) {
        // 继续游戏
        next_state.set(GameState::Playing);
    } else if keyboard_input.just_pressed(KeyCode::KeyN) {
        // 重新开始游戏 - 先清理现有游戏实体
        for entity in game_entities.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        level.0 = 1;
        score.0 = 0;
        lives.0 = difficulty_settings.lives;
        *power_effects = PowerUpEffects::default();
        game_initialized.0 = false;  // 重置初始化状态
        next_state.set(GameState::Playing);
    } else if keyboard_input.just_pressed(KeyCode::KeyM) {
        // 返回主菜单 - 先清理现有游戏实体
        for entity in game_entities.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        level.0 = 1;
        score.0 = 0;
        lives.0 = difficulty_settings.lives;
        *power_effects = PowerUpEffects::default();
        game_initialized.0 = false;  // 重置初始化状态
        next_state.set(GameState::MainMenu);
    }
}

// 清理暂停菜单
fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}