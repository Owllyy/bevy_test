use bevy::{math::vec2, prelude::*, sprite::MaterialMesh2dBundle};
use bevy_xpbd_2d::prelude::*;

use crate::score::Score;


#[derive(Bundle)]
pub struct BallBundle {
    pub ball: Ball,
    pub rigid_body: RigidBody,
    pub restitution: Restitution,
    pub collider: Collider,
    pub mass: Mass,
    pub texture: SpriteBundle,
    pub linear_damping: LinearDamping,
    pub angular_damping: AngularDamping,
}

impl BallBundle {
    pub fn new(
        position: Vec3,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        asset_server: &mut Res<AssetServer>,
        ball: Ball,
    ) -> Self {
        let properites = ball.0;
        Self {
            ball,
            rigid_body: RigidBody::Dynamic,
            restitution: Restitution::new(0.0),
            collider: Collider::circle(0.53),
            mass: Mass(properites.properties().0),
            angular_damping: AngularDamping(20000.),
            linear_damping: LinearDamping(20.),
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
            BallType::ONE => (25., "moon.png".to_string(), 1),
            BallType::TWO => (35., "earth.png".to_string(), 2),
            BallType::THREE => (55., "mars.png".to_string(), 4),
            BallType::FOUR => (75., "snow.png".to_string(), 8),
            BallType::FIVE => (100., "toxic.png".to_string(), 16),
            BallType::SIX => (125., "lava.png".to_string(), 32),
            BallType::SEVEN => (150., "milk.png".to_string(), 64),
            BallType::EIGHT => (175., "green.png".to_string(), 128),
            BallType::NINE => (200., "emma.png".to_string(), 258),
            BallType::TEN => (225., "sand.png".to_string(), 512),
            BallType::ELEVEN => (250., "sun.png".to_string(), 1024),
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
    mut collision_event_reader: EventReader<CollisionStarted>,
    query: Query<(&mut Ball, &mut Transform)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut score: Query<&mut Score>,
    mut asset_server: Res<AssetServer>,
) {
    for &CollisionStarted(e1, e2) in collision_event_reader.read() {
        if let (Ok((ball1, trasnform1)), Ok((ball2, transform2))) = (query.get(e1), query.get(e2)) {
            if ball1 == ball2 {
                commands.entity(e1).despawn();
                commands.entity(e2).despawn();
                let next_ball = BallBundle::new(
                    (trasnform1.translation + transform2.translation) / 2.,
                    &mut materials,
                    &mut asset_server,
                    ball1.0.next(),
                );

                // Add score todo remove from here
                if let Ok(mut score) = score.get_single_mut() {
                    score.0 += next_ball.ball.0.properties().2;
                }

                commands.spawn(next_ball);
            }
        }
    }
}