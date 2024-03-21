use bevy::{math::vec2, prelude::*};
use bevy_cursor::CursorLocation;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::planet::BallType;

#[derive(Default, Bundle)]
pub struct CursorBundle {
    balltype: BallType,
    rigidbody: RigidBody,
    texture: SpriteBundle,
}

impl CursorBundle {
    pub fn new(
        balltype: BallType,
        meshes: &mut Assets<Mesh>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        asset_server: &mut Res<AssetServer>,
    ) -> Self {
        let properties = balltype.properties();
        Self {
            balltype,
            rigidbody: RigidBody::KinematicVelocityBased,
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

    pub fn rand(
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


#[derive(Component)]
pub struct CursorCooldown(f32);

impl Default for CursorCooldown {
    fn default() -> Self {
        Self(0.5)
    }
}

#[derive(Default, Component, PartialEq)]
pub struct BallCursor;

const NEXT_CURSOR_LOCATION: Vec2 = vec2(350., 350.);
#[derive(Default, Component, PartialEq)]
pub struct NextCursor;

const CURSOR_ORBIT: f32 = 350.0;
const TRACKING_SPEED: f32 = 8.0;

pub fn cursor_cooldown(
    time: Res<Time>,
    mut commands: Commands,
    mut cooldown: Query<(Entity, &mut CursorCooldown)>,
) {
    for (entity, mut cooldown) in cooldown.iter_mut() {
        if cooldown.0 > 0. {
            cooldown.0 -= time.delta_seconds();
        } else {
            commands.entity(entity).despawn();
        }
    }
}

pub fn cursor_tracking(
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
