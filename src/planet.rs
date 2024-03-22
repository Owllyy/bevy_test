use std::borrow::BorrowMut;

use bevy::{math::{vec2, vec3}, prelude::*, scene::ron::de::Position, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::prelude::*;

use crate::{cursor::CursorCooldown, score::Score, BallCursor, CursorBundle, NextCursor};

const ANIMATION_SPEED: f32 = 0.15;

#[derive(Component)]
pub struct Growing(f32);
impl Default for Growing {
    fn default() -> Self {
        Self(ANIMATION_SPEED)
    }
}

pub fn growing(
    time: Res<Time>,
    mut commands: Commands,
    mut balls: Query<(Entity, &mut Transform, &mut RigidBody, &mut Growing, &Ball), Without<Fusion>>,) {
    for (ball, mut transform, mut body, mut timer, size) in balls.iter_mut() {
        if timer.0 > 0. {
            timer.0 -= time.delta_seconds();
            let size = 1. * (ANIMATION_SPEED - timer.0) / ANIMATION_SPEED * size.0.properties().0;
            transform.scale = vec3(size, size, 1.)
        } else {
            let size = size.0.properties().0;
            *body = RigidBody::Dynamic;
            transform.scale = vec3(size, size, 1.);
            commands.entity(ball).remove::<Growing>();
        }
    }
}

#[derive(Component)]
pub struct Fusion(f32);
impl Default for Fusion {
    fn default() -> Self {
        Self(ANIMATION_SPEED)
    }
}

pub fn fusioning(
time: Res<Time>,
mut commands: Commands,
mut balls: Query<(Entity, &mut Fusion)>,) {
    for (ball, mut timer) in balls.iter_mut() {
        if timer.0 > 0. {
            timer.0 -= time.delta_seconds();
        } else {
            commands.entity(ball).despawn();
        }
    }
}

#[derive(Bundle)]
pub struct BallBundle {
    pub ball: Ball,
    pub rigid_body: RigidBody,
    pub restitution: Restitution,
    pub collider: Collider,
    pub mass: AdditionalMassProperties,
    pub texture: SpriteBundle,
    pub velocity: Velocity,
    pub damping: Damping,
    pub gravity: ExternalForce,
    pub collision: ActiveEvents,
    pub friction: Friction,
}

impl BallBundle {
    pub fn new(
        position: Vec3,
        asset_server: &mut Res<AssetServer>,
        ball: Ball,
    ) -> Self {
        let properites = ball.0;
        Self {
            ball,
            rigid_body: RigidBody::Dynamic,
            gravity: ExternalForce::default(),
            friction: Friction::new(0.),
            restitution: Restitution::new(0.1),
            collider: Collider::ball(0.54),
            collision: ActiveEvents::COLLISION_EVENTS,
            velocity: Velocity::default(),
            mass: AdditionalMassProperties::Mass(properites.properties().2 as f32 * 100.0),
            damping: Damping {
                linear_damping: 50.,
                angular_damping: 50000.,
            },
            texture: SpriteBundle {
                texture: asset_server.load(properites.properties().1.clone()),
                transform: Transform::from_translation(position)
                        .with_scale(Vec3::splat(properites.properties().0)),
                sprite: Sprite {
                    custom_size: Some(vec2(1., 1.)),
                    ..Default::default()
                },
                ..default()
            },
        }
    }
    pub fn growing(
        position: Vec3,
        asset_server: &mut Res<AssetServer>,
        ball: Ball,
    ) -> Self {
        let properites = ball.0;
        Self {
            ball,
            rigid_body: RigidBody::KinematicVelocityBased,
            gravity: ExternalForce::default(),
            friction: Friction::new(0.),
            restitution: Restitution::new(0.1),
            collider: Collider::ball(0.54),
            collision: ActiveEvents::COLLISION_EVENTS,
            velocity: Velocity::default(),
            mass: AdditionalMassProperties::Mass(properites.properties().2 as f32 * 100.0),
            damping: Damping {
                linear_damping: 50.,
                angular_damping: 50000.,
            },
            texture: SpriteBundle {
                texture: asset_server.load(properites.properties().1.clone()),
                transform: Transform::from_translation(position)
                        .with_scale(Vec3::splat(0.)),
                sprite: Sprite {
                    custom_size: Some(vec2(1., 1.)),
                    ..Default::default()
                },
                ..default()
            },
        }
    }
}

#[derive(PartialEq, Default, Copy, Clone, Component)]
pub enum BallType {
    #[default]
    ONE,
    TWO,
    THREE,
    FOUR,
    FIVE,
    SIX,
    SEVEN,
    EIGHT,
    NINE,
    TEN,
    ELEVEN,
}

#[derive(Default, Component, PartialEq, Copy, Clone)]
pub struct Ball(pub BallType);
impl BallType {
    pub fn properties(&self) -> (f32, String, u64) {
        match self {
            BallType::ONE => (30., "moon.png".to_string(), 2),
            BallType::TWO => (37., "earth.png".to_string(), 3),
            BallType::THREE => (47. , "mars.png".to_string(), 4),
            BallType::FOUR => (60., "snow.png".to_string(), 8),
            BallType::FIVE => (75., "toxic.png".to_string(), 16),
            BallType::SIX => (95., "lava.png".to_string(), 32),
            BallType::SEVEN => (120., "milk.png".to_string(), 64),
            BallType::EIGHT => (151., "green.png".to_string(), 128),
            BallType::NINE => (190., "emma.png".to_string(), 258),
            BallType::TEN => (240., "sand.png".to_string(), 512),
            BallType::ELEVEN => (302., "sun.png".to_string(), 1024),
        }
    }

    pub fn next(&self) -> Ball {
        match self {
            BallType::ONE => Ball(BallType::TWO),
            BallType::TWO => Ball(BallType::THREE),
            BallType::THREE => Ball(BallType::FOUR),
            BallType::FOUR => Ball(BallType::FIVE),
            BallType::FIVE => Ball(BallType::SIX),
            BallType::SIX => Ball(BallType::SEVEN),
            BallType::SEVEN => Ball(BallType::EIGHT),
            BallType::EIGHT => Ball(BallType::NINE),
            BallType::NINE => Ball(BallType::TEN),
            BallType::TEN => Ball(BallType::ELEVEN),
            BallType::ELEVEN => Ball(BallType::ONE),
        }
    }
}

pub fn fusion(
    mut commands: Commands,
    mut collision_event_reader: EventReader<CollisionEvent>,
    mut balls: Query<(&mut Ball, &mut Transform, &mut Velocity), Without<Fusion>>,
    mut score: Query<&mut Score>,
    mut asset_server: Res<AssetServer>,
) {
    let mut already_fusionned: Vec<Entity> = vec![];

    for event in collision_event_reader.read() {
        if let CollisionEvent::Started(e1, e2, flag) = event {
            if let Ok([(ball1, transform1, mut velocity1), (ball2, transform2, mut velocity2)]) = balls.get_many_mut([*e1, *e2]) {
                if ball1.0 == ball2.0 && !already_fusionned.contains(e1) && !already_fusionned.contains(e2) {
                    commands.entity(*e1).insert(Fusion::default());
                    commands.entity(*e2).insert(Fusion::default());
                    commands.entity(*e1).insert(RigidBody::KinematicVelocityBased);
                    commands.entity(*e2).insert(RigidBody::KinematicVelocityBased);
                    already_fusionned.push(*e1);
                    already_fusionned.push(*e2);
                    let mut position = (transform1.translation + transform2.translation) / 2.;
                    position.z = transform1.translation.z + transform2.translation.z + 1.;
                    velocity2.linvel = (transform1.translation.xy() - transform2.translation.xy());
                    velocity1.linvel = (transform2.translation.xy() - transform1.translation.xy());
                    let next_ball = (BallBundle::growing(
                        position,
                        &mut asset_server,
                        ball1.0.next(),
                    ), Growing::default());

                    // Add score todo remove from here
                    if let Ok(mut score) = score.get_single_mut() {
                        score.score += next_ball.0.ball.0.properties().2;
                    }

                    commands.spawn(next_ball);
                }
            }
        } else {
            continue;
        }

    }
}


// pub fn repulsion(
//     mut collision_event_reader: EventReader<CollisionEvent>,
//     mut balls: Query<(& Transform, &mut ExternalForce), With<Ball>>,
// ) {
//     for event in collision_event_reader.read() {
//         if let CollisionEvent::Started(e1, e2, _) = event {
//             if let Ok([(transform1, mut force1), (transform2, mut force2)]) = balls.get_many_mut([*e1, *e2]) {
//                 let dir = transform1.translation - transform2.translation;
//                 force1.force = dir.xy() * 10000.;
//                 force2.force = dir.xy() * -1. * 10000.;
//             }
//         }
//     }
// }

const GRAVITY_FORCE: f32 = 100.0;

pub fn gravity(
    mut query: Query<(&mut ExternalForce, &Transform, &AdditionalMassProperties), With<Ball>>,
) {
    for (mut gravity, trasform, mass) in &mut query {
        if let &AdditionalMassProperties::Mass(mass) = mass {
            let mut dir = Vec3::default() - trasform.translation;
            gravity.force = dir.xy() * GRAVITY_FORCE * mass;
        }
    }
}

pub fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Query<(Entity, &mut Transform, &BallType), With<BallCursor>>,
    next_cursor: Query<Entity, With<NextCursor>>,
    mut score: Query<&mut Score>,
    cooldown: Query<(), With<CursorCooldown>>,
    mut asset_server: Res<AssetServer>,
) {
    if cooldown.get_single().is_ok() {
        return ;
    }
    if buttons.just_pressed(MouseButton::Left) {
        if let Ok((current, cursor, balltype)) = cursor.get_single() {
            // Set cursor cooldown
            commands.spawn(CursorCooldown::default());

            // Spawn Ball from BallCursor
            commands.spawn(BallBundle::new(
                cursor.translation,
                &mut asset_server,
                Ball(*balltype),
            ));

            // Add score todo remove from here
            if let Ok(mut score) = score.get_single_mut() {
                score.score += balltype.properties().2;
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
