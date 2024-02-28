use bevy::{math::{quat, vec3}, prelude::*, sprite::MaterialMesh2dBundle};
use bevy_xpbd_2d::{components::{AngularVelocity, Friction, LinearDamping, LinearVelocity, RigidBody}, plugins::{collision::Collider, PhysicsPlugins}};
use bevy_cursor::prelude::*;

#[derive(Default, Component)]
struct Player {
    speed: Vec2,
    dash_speed: Vec2,
}

#[derive(Default, Component)]
struct Direction(Vec2);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TrackCursorPlugin))
        .add_plugins(PhysicsPlugins::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (keyboard_input_system, look_cursor,applyforce, dash).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        Player {
            speed: Vec2::ZERO,
            dash_speed: Vec2::ZERO,
        },
        Direction(Vec2::default()),
        RigidBody::Kinematic,
        MaterialMesh2dBundle {
            mesh: meshes.add(Triangle2d::new(
                Vec2::new(0.5, 0.0),
                Vec2::new(-0.5, -0.5),
                Vec2::new(-0.5, 0.5),
            )).into(),
            transform: Transform::default().with_scale(Vec3::splat(64.)),
            material: materials.add(Color::PURPLE),
            ..default()
        },
    ));
}

const ACCELERATION: f32 = 6000.0;
const DECCELERATION: f32 = 4000.0;
const DASH_FORCE: f32 = 1000.0;
const DASH_DECELRATION: f32 = 3000.0;
const MAX_SPEED: f32 = 800.0;

fn look_cursor(
    time: Res<Time>,
    cursor: Res<CursorLocation>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query:  Query<&mut Transform, With<Player>>,
) {
    for mut transform in &mut query {
        if let Some(cursor_pos) = cursor.world_position() {
            let dir = cursor_pos - transform.translation.xy();
            transform.rotation = Quat::from_axis_angle(vec3(0.0, 0.0, 1.0), dir.to_angle())
        }
    }
}

fn dash(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query:  Query<(&mut Player, &Direction)>,
) {
    for (mut player, dir) in &mut query {
        if keyboard_input.just_pressed(KeyCode::Space) {
            player.dash_speed = dir.0.normalize_or_zero() * DASH_FORCE;
        } else {
            let force = player.dash_speed.normalize_or_zero() * DASH_DECELRATION * time.delta_seconds();
            if force.length() > player.dash_speed.length() {
                player.dash_speed = Vec2::ZERO;
            } else {
                player.dash_speed -= force;
            }
        }

    }
}

fn keyboard_input_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query:  Query<(&mut Player, &mut Direction)>,
) {
    for (mut player, mut direction) in &mut query {
        direction.0 = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.0.x -= 1.0;
        } if keyboard_input.pressed(KeyCode::KeyD) {
            direction.0.x += 1.0;
        } if keyboard_input.pressed(KeyCode::KeyW) {
            direction.0.y += 1.0;
        } if keyboard_input.pressed(KeyCode::KeyS) {
            direction.0.y -= 1.0;
        }

        if direction.0 != Vec2::ZERO {
            player.speed += direction.0.normalize_or_zero() * ACCELERATION * time.delta_seconds();
        } else {
            let force = player.speed.normalize_or_zero() * DECCELERATION * time.delta_seconds();
            if force.length() > player.speed.length() {
                player.speed = Vec2::ZERO;
            } else {
                player.speed -= force;
            }
        }

        if player.speed.length() > MAX_SPEED {
            player.speed = player.speed.normalize_or_zero() * MAX_SPEED;
        }
        
    }
}

fn applyforce(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query:  Query<(&mut Player, &mut LinearVelocity)>) 
{
    for (player, mut velocity) in &mut query {
        velocity.0 = player.speed + player.dash_speed; 
    }
}