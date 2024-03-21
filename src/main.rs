use bevy::{math::{vec2, vec3}, prelude::*, window::WindowResolution
};
use bevy_cursor::prelude::*;
mod planet;
use bevy_rapier2d::prelude::*;
use planet::{fusion,fusioning,growing, gravity, spawn_ball, Ball, BallType};
mod score;
mod game_over;
mod cursor;
use cursor::{cursor_cooldown, cursor_tracking, BallCursor, CursorBundle, NextCursor};
use score::{best_score_update, score_update, BestScore, BestScoreBundle, Score, ScoreBundle};
use game_over::{game_over, restart};

#[derive(Resource, Reflect)]
struct Configuration {
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
        }
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    #[default]
    Playing,
    GameOver,
}

fn main() {
    App::new()
        .init_state::<AppState>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        position: WindowPosition::Centered(MonitorSelection::Current),
                        resolution: WindowResolution::new(800.0, 900.0),
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TrackCursorPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .insert_resource(Configuration::default())
        .register_type::<Configuration>()
        // .add_plugins(PhysicsDebugPlugin::default())
        // .add_plugins(ResourceInspectorPlugin::<Configuration>::default())
        // .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default()
        .add_systems(Startup, (setup, planet_order))
        .add_systems(
            Update,
            (spawn_ball, gravity, fusion,fusioning,growing, cursor_tracking, score_update, game_over, cursor_cooldown).chain().run_if(in_state(AppState::Playing)),
        )
        .add_systems(Update, (best_score_update, cursor_tracking, gravity, restart).chain().run_if(in_state(AppState::GameOver)))
        .run();
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut asset_server: Res<AssetServer>,
    mut configuration: ResMut<RapierConfiguration>,
) {
    configuration.gravity = vec2(0., 0.);
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle{
        texture: asset_server.load("background.png"),
        transform: Transform::default().with_translation(vec3(0., 0., -10.)).with_scale(vec3(0.7, 0.7, 1.)),
        ..Default::default()
    });
    commands.spawn(SpriteBundle{
        texture: asset_server.load("GameOver.png"),
        transform: Transform::default().with_translation(vec3(-0.1, 0., -9.)).with_scale(vec3(1.54, 1.54, 1.)),
        sprite: Sprite {
            color: Color::rgba(0.0, 0.0, 0.1, 0.7),
            ..Default::default()
        },
        ..Default::default()
    });
    //next cursor
    commands.spawn((
        CursorBundle::rand(&mut meshes, &mut materials, &mut asset_server),
        NextCursor,
    ));
    //ball cursor
    commands.spawn((
        CursorBundle::rand(&mut meshes, &mut materials, &mut asset_server),
        BallCursor,
    ));

    commands.spawn(ScoreBundle::default());
    commands.spawn(BestScoreBundle::default());
}

fn planet_order(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut ball_type = Ball::default().0;
    let mut x = 0.;
    let step = 800. / 22.;
    for i in 1..12 {
        commands.spawn(SpriteBundle{
            texture: asset_server.load(ball_type.properties().1),
            transform: Transform::default().with_translation(vec3(-400. + step + x, -400., -8.)).with_scale(vec3(0.2, 0.2, 1.)),
            ..Default::default()
        });
        x += step;
        if i < 11 {
            commands.spawn(SpriteBundle{
                texture: asset_server.load("arrow.png"),
                transform: Transform::default().with_translation(vec3(-400. + step + x, -400., -8.)).with_scale(vec3(0.2, 0.2, 1.)),
                sprite: Sprite {
                    color: Color::rgba(0.2, 0.2, 0.4, 0.8),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
        x += step;
        ball_type = ball_type.next().0;
    }
}

