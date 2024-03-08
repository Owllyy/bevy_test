use bevy::{
    math::{vec2, vec3},
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::WindowResolution,
};
use bevy_cursor::prelude::*;
mod planet;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use planet::{fusion, Ball, BallBundle, BallType};
mod score;
use bevy_xpbd_2d::{
    components::{
        AngularDamping, CoefficientCombine, GravityScale, LinearDamping, LinearVelocity, Mass,
        RigidBody,
    },
    plugins::{
        collision::{contact_reporting::CollisionStarted, Collider},
        PhysicsDebugPlugin, PhysicsPlugins,
    },
    resources::Gravity,
    PhysicsSchedule,
};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use score::{score_update, Score, ScoreBundle};

#[derive(Resource, Reflect)]
struct Configuration {
    gravity_scale: f32,
    gravity_distance: f32,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            gravity_scale: 1.,
            gravity_distance: 1000.,
        }
    }
}

#[derive(Component)]
struct Playing;

#[derive(Component)]
struct CursorCooldown(f32);

impl Default for CursorCooldown {
    fn default() -> Self {
        Self(0.5)
    }
}

#[derive(Component)]
struct GameOver;


#[derive(Default, Component, PartialEq)]
struct BallCursor;

const NEXT_CURSOR_LOCATION: Vec2 = vec2(350., 350.);
#[derive(Default, Component, PartialEq)]
struct NextCursor;

#[derive(Default, Bundle)]
struct CursorBundle {
    balltype: BallType,
    rigidbody: RigidBody,
    texture: SpriteBundle,
}

impl CursorBundle {
    fn new(
        balltype: BallType,
        meshes: &mut Assets<Mesh>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        asset_server: &mut Res<AssetServer>,
    ) -> Self {
        let properties = balltype.properties();
        Self {
            balltype,
            rigidbody: RigidBody::Kinematic,
            texture: SpriteBundle {
                texture: asset_server.load(properties.1),
                transform: Transform::from_translation(NEXT_CURSOR_LOCATION.extend(0.))
                    .with_scale(Vec3::splat(properties.0)),
                sprite: Sprite {
                    custom_size: Some(vec2(1., 1.)),
                    ..Default::default()
                },
                ..default()
            },
        }
    }

    fn rand(
        meshes: &mut Assets<Mesh>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        asset_server: &mut Res<AssetServer>,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let balltype = match rng.gen_range(0..=4) {
            // rand 0.8
            0 => BallType::ONE,
            1 => BallType::TWO,
            2 => BallType::THREE,
            3 => BallType::FOUR,
            _ => BallType::FIVE,
        };
        CursorBundle::new(balltype, meshes, materials, asset_server)
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        position: WindowPosition::Centered(MonitorSelection::Current),
                        resolution: WindowResolution::new(800.0, 800.0),
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TrackCursorPlugin)
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec2::default()))
        .insert_resource(Configuration::default())
        .register_type::<Configuration>()
        // .add_plugins(PhysicsDebugPlugin::default())
        .add_plugins(ResourceInspectorPlugin::<Configuration>::default())
        // .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (spawn_ball, fusion, gravity, cursor_tracking, score_update, game_over, cursor_cooldown).chain(),
        )
        .run();
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle{
        texture: asset_server.load("background.png"),
        transform: Transform::default().with_translation(vec3(0., 0., -10.)).with_scale(vec3(0.7, 0.7, 1.)),
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
    commands.spawn(Playing);
}

const CURSOR_ORBIT: f32 = 350.0;
const TRACKING_SPEED: f32 = 8.0;

fn cursor_cooldown(
    time: Res<Time>,
    mut commands: Commands,
    mut cooldown: Query<(Entity, &mut CursorCooldown)>,
) {
    if let Ok((entity, mut cooldown)) = cooldown.get_single_mut() {
        if cooldown.0 > 0. {
            cooldown.0 -= time.delta_seconds();
        } else {
            commands.entity(entity).despawn();
        }
    }
}

fn game_over(
    mut commands: Commands,
    mut status: Query<Entity, With<Playing>>,
    query: Query<&mut Transform, With<Ball>>,
) {
    if let Ok(playing) = status.get_single_mut() {
        for ball in query.iter() {
            if ball.translation.length() > 400. {
                commands.entity(playing).despawn();
                commands.spawn(GameOver);
            }
        }
    }
}

fn cursor_tracking(
    time: Res<Time>,
    cursor: Res<CursorLocation>,
    mut query: Query<&mut Transform, With<BallCursor>>,
) {
    if let Some(cursor) = cursor.world_position() {
        for mut transform in &mut query {
            transform.translation = (transform.translation
                + (cursor.extend(0.) * time.delta_seconds() * TRACKING_SPEED))
                .normalize_or_zero()
                * ((CURSOR_ORBIT + transform.translation.length() * 4.) / 5.);
        }
    }
}

const GRAVITY_FORCE: f32 = 200.0;

fn gravity(
    time: Res<Time>,
    mut query: Query<(&mut LinearVelocity, &Transform), With<Ball>>,
    configuration: Res<Configuration>,
) {
    for (mut velocity, trasform) in &mut query {
        let mut dir = Vec3::default() - trasform.translation;
            dir = dir * GRAVITY_FORCE * time.delta_seconds();
            velocity.0 = dir.xy();
        // if dir.length() > 20. {
        // velocity.0 += dir.normalize_or_zero().xy()
        //     * configuration.gravity_distance
        //     * time.delta_seconds();
        // }
    }
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Query<(Entity, &mut Transform, &BallType), With<BallCursor>>,
    next_cursor: Query<Entity, With<NextCursor>>,
    mut score: Query<&mut Score>,
    mut asset_server: Res<AssetServer>,
    mut status: Query<(), With<Playing>>,
    mut cooldown: Query<(), With<CursorCooldown>>,
) {
    if status.get_single().is_err() || cooldown.get_single().is_ok() {
        return;
    }
    if buttons.just_pressed(MouseButton::Left) {
        if let Ok((current, cursor, balltype)) = cursor.get_single() {
            // Spawn Ball from BallCursor
            commands.spawn(BallBundle::new(
                cursor.translation,
                &mut materials,
                &mut asset_server,
                Ball(*balltype),
            ));

            // Set cursor cooldown
            commands.spawn(CursorCooldown::default());

            // Add score todo remove from here
            if let Ok(mut score) = score.get_single_mut() {
                score.0 += balltype.properties().2;
            }

            // Despawn curent cursor
            commands.entity(current).despawn();

            // Switch NextCursor to BallCursor & spawn a new rand NextCursor
            if let Ok((entity)) = next_cursor.get_single() {
                commands.entity(entity).insert(BallCursor);
                commands.entity(entity).remove::<NextCursor>();

                commands.spawn((
                    CursorBundle::rand(&mut meshes, &mut materials, &mut asset_server),
                    NextCursor,
                ));
            }
        }
    }
}
