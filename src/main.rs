use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_cursor::prelude::*;
use bevy_inspector_egui::{
    inspector_options::ReflectInspectorOptions, quick::ResourceInspectorPlugin, InspectorOptions,
};
use bevy_xpbd_2d::{
    components::{LinearDamping, LinearVelocity, Mass, RigidBody},
    plugins::{collision::Collider, PhysicsPlugins},
};

#[derive(Default, Component, Reflect)]
struct Player {
    speed: Vec2,
}

#[derive(Default, Component, Reflect)]
struct Dash {
    speed: Vec2,
    duration: f32,
}

#[derive(Default, Component)]
struct Direction(Vec2);

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    acceleration: f32,
    decceleration: f32,
    max_speed: f32,
    dash_speed: f32,
    dash_decelration: f32,
    dash_duration: f32,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            acceleration: 6000.0,
            decceleration: 4000.0,
            max_speed: 800.0,
            dash_speed: 2000.0,
            dash_decelration: 200000.0,
            dash_duration: 0.1,
        }
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TrackCursorPlugin))
        .add_plugins(PhysicsPlugins::default())
        // .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(ResourceInspectorPlugin::<Configuration>::default())
        .init_resource::<Configuration>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (keyboard_input_system, look_cursor, apply_force, dash).chain(),
        )
        .register_type::<Player>()
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        Player { speed: Vec2::ZERO },
        Direction(Vec2::default()),
        RigidBody::Kinematic,
        Collider::circle(0.5),
        MaterialMesh2dBundle {
            mesh: meshes
                .add(Triangle2d::new(
                    Vec2::new(0.5, 0.0),
                    Vec2::new(-0.5, -0.5),
                    Vec2::new(-0.5, 0.5),
                ))
                .into(),
            transform: Transform::default().with_scale(Vec3::splat(64.)),
            material: materials.add(Color::PURPLE),
            ..default()
        },
    ));
    commands.spawn((
        Direction(Vec2::default()),
        RigidBody::Dynamic,
        Collider::circle(2.0),
        Mass(1000.0),
        LinearDamping(1000.0),
        MaterialMesh2dBundle {
            mesh: meshes
                .add(Triangle2d::new(
                    Vec2::new(0.5, 0.0),
                    Vec2::new(-0.5, -0.5),
                    Vec2::new(-0.5, 0.5),
                ))
                .into(),
            transform: Transform::default().with_scale(Vec3::splat(64.)),
            material: materials.add(Color::PURPLE),
            ..default()
        },
    ));
}

fn look_cursor(cursor: Res<CursorLocation>, mut query: Query<&mut Transform, With<Player>>) {
    for mut transform in &mut query {
        if let Some(cursor_pos) = cursor.world_position() {
            let dir = cursor_pos - transform.translation.xy();
            transform.rotation = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), dir.to_angle())
        }
    }
}

fn dash(
    time: Res<Time>,
    configurition: Res<Configuration>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut command: Commands,
    mut query: Query<(Entity, &Direction, Option<&mut Dash>)>,
) {
    for (id, dir, dash) in &mut query {
        if keyboard_input.just_pressed(KeyCode::Space) && dash.is_none() {
            command.entity(id).insert(Dash {
                speed: dir.0.normalize_or_zero() * configurition.dash_speed,
                duration: configurition.dash_duration,
            });
        } else if let Some(mut dash) = dash {
            if dash.duration > 0.0 {
                dash.duration -= time.delta_seconds();
            } else {
                let force = dash.speed.normalize_or_zero()
                    * configurition.dash_decelration
                    * time.delta_seconds();
                if force.length() >= dash.speed.length() {
                    dash.speed = Vec2::ZERO;
                    command.entity(id).remove::<Dash>();
                } else {
                    dash.speed -= force;
                }
            }
        }
    }
}

fn keyboard_input_system(
    time: Res<Time>,
    configurition: Res<Configuration>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Direction)>,
) {
    for (mut player, mut direction) in &mut query {
        direction.0 = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.0.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.0.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.0.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.0.y -= 1.0;
        }

        if direction.0 != Vec2::ZERO {
            player.speed +=
                direction.0.normalize_or_zero() * configurition.acceleration * time.delta_seconds();
        } else {
            let force = player.speed.normalize_or_zero()
                * configurition.decceleration
                * time.delta_seconds();
            if force.length() > player.speed.length() {
                player.speed = Vec2::ZERO;
            } else {
                player.speed -= force;
            }
        }

        if player.speed.length() > configurition.max_speed {
            player.speed = player.speed.normalize_or_zero() * configurition.max_speed;
        }
    }
}

fn apply_force(mut query: Query<(&mut Player, &mut LinearVelocity, Option<&Dash>)>) {
    for (player, mut velocity, dash) in &mut query {
        velocity.0 = player.speed;
        if let Some(dash) = dash {
            velocity.0 += dash.speed;
        }
    }
}
