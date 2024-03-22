use bevy::{diagnostic::DiagnosticsStore, prelude::*};

use crate::{cursor::CursorCooldown, planet::Ball, score::{best_score_update, BestScore, Score}, AppState};


#[derive(Component)]
pub struct GameOver;

#[derive(Component)]
pub struct GameOverCooldown(f32);

#[derive(Bundle)]
pub struct GameOverText {
    pub game_over: GameOver,
    pub bundle: TextBundle,
}

impl Default for GameOverText {
    fn default() -> Self {
        Self {
            game_over: GameOver,
            bundle: TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "\tGame Over press R to restart",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font_size: 40.0,
                    ..default()
                },
            ).with_style(Style{
                top: Val::Px(430.0),
                left: Val::Px(120.0),
                ..Default::default()}),
        }
    }
}

pub fn game_over(
    mut commands: Commands,
    mut cooldown: Query<(), With<GameOverCooldown>>,
    query: Query<(Entity, &mut Transform), With<Ball>>,
    mut state: ResMut<NextState<AppState>>,
) {
    for ball in query.iter() {
        if ball.translation.length() > 350. {
            commands
            println!("Fail {}", ball.translation.length());
            commands.spawn(GameOverText::default());
            state.set(AppState::GameOver);
        }
    }
}

pub fn restart(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    balls: Query<Entity, With<Ball>>,
    mut text: Query<Entity, With<GameOver>>,
    mut cooldown: Query<Entity, With<CursorCooldown>>,
    mut score: Query<&mut Score>,
    mut best_score: Query<(&mut BestScore, &mut Text)>,
    mut state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for ball in balls.iter() {
            commands.entity(ball).despawn();
        }
        if let Ok(cooldown) = cooldown.get_single_mut() {
            commands.entity(cooldown).despawn();
        }
        if let Ok(text) = text.get_single_mut() {
            commands.entity(text).despawn();
        }
        if let Ok(mut score) = score.get_single_mut() {
            if let Ok((mut best_score_value, _)) = best_score.get_single_mut() {
                if score.score > best_score_value.score {
                    best_score_value.score = score.score;
                    best_score_update(best_score);
                }
            }
            score.score = 0;
        }
        state.set(AppState::Playing);
    }
}