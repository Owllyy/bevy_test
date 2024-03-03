use bevy::{math::{vec2, vec3}, prelude::*, sprite::MaterialMesh2dBundle, window::WindowResolution};
use bevy_cursor::prelude::*;
use bevy_xpbd_2d::{
    components::{
        AngularDamping, CoefficientCombine, LinearDamping, LinearVelocity, Mass, RigidBody,
    },
    plugins::{
        collision::{contact_reporting::CollisionStarted, Collider},
        PhysicsDebugPlugin, PhysicsPlugins,
    },
};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(PartialEq, Default, Copy, Clone, Component)]
enum BallType {
    #[default]
    ONE,
    TWO,
    THREE,
    FOUR,
    FIVE,
    SIX,
    SEVEN,
    EIGHT,
    NINE
}

#[derive(Default, Component, PartialEq)]
struct Ball(BallType);
impl BallType {
    fn properties(&self) -> (f32, Color) {
        match self {
            BallType::ONE => (25., Color::BLUE),
            BallType::TWO => (35., Color::RED),
            BallType::THREE => (55., Color::LIME_GREEN),
            BallType::FOUR => (75., Color::BISQUE),
            BallType::FIVE => (100., Color::ORANGE),
            BallType::SIX => (125., Color::VIOLET),
            BallType::SEVEN => (150., Color::PINK),
            BallType::EIGHT => (175., Color::GREEN),
            BallType::NINE => (200., Color::GRAY),
        }
    }

    fn next(&self) -> Ball {
        match self {
            BallType::ONE => Ball(BallType::TWO),
            BallType::TWO => Ball(BallType::THREE),
            BallType::THREE => Ball(BallType::FOUR),
            BallType::FOUR => Ball(BallType::FIVE),
            BallType::FIVE => Ball(BallType::SIX),
            BallType::SIX => Ball(BallType::SEVEN),
            BallType::SEVEN => Ball(BallType::EIGHT),
            BallType::EIGHT => Ball(BallType::NINE),
            BallType::NINE => Ball(BallType::ONE),
        }
    }
}



#[derive(Default, Component, PartialEq)]
struct BallCursor;

const NEXT_CURSOR_LOCATION: Vec2 = vec2(350., 350.);
#[derive(Default, Component, PartialEq)]
struct NextCursor;

#[derive(Default, Bundle)]
struct CursorBundle {
    balltype: BallType,
    rigidbody: RigidBody,
    material: MaterialMesh2dBundle<ColorMaterial>,
}

impl CursorBundle {
    fn new(balltype: BallType,
        meshes: &mut Assets<Mesh>,
        materials: &mut ResMut<Assets<ColorMaterial>>,) -> Self {
        Self {
            balltype,
            rigidbody: RigidBody::Kinematic,
            material: MaterialMesh2dBundle {
                mesh: meshes.add(Circle::default()).into(),
                transform: Transform::from_translation(NEXT_CURSOR_LOCATION.extend(0.))
                    .with_scale(Vec3::splat(balltype.properties().0)),
                material: materials.add(balltype.properties().1),
                ..default()
            },
        }
    }

    fn rand(
        meshes: &mut Assets<Mesh>,
        materials: &mut ResMut<Assets<ColorMaterial>>
    ) -> Self {
        let mut rng = rand::thread_rng();
        let balltype = match rng.gen_range(0..=4) { // rand 0.8
            0 => BallType::ONE,
            1 => BallType::TWO,
            2 => BallType::THREE,
            3 => BallType::FOUR,
            _ => BallType::FIVE,
        };
        CursorBundle::new(balltype, meshes, materials)
    }
}

#[derive(Default, Bundle)]
struct BallBundle {
    ball: Ball,
    rigid_body: RigidBody,
    collider: Collider,
    mass: Mass,
    material: MaterialMesh2dBundle<ColorMaterial>,
    linear_damping: LinearDamping,
    angular_damping: AngularDamping,
}

impl BallBundle {
    fn new(
        position: Vec3,
        meshes: &mut Assets<Mesh>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        ball: Ball,
    ) -> Self {
        let properites = ball.0.properties();
        Self {
            ball,
            rigid_body: RigidBody::Dynamic,
            collider: Collider::circle(0.5),
            mass: Mass(properites.0),
            angular_damping: AngularDamping(20.),
            linear_damping: LinearDamping(20.),
            material: MaterialMesh2dBundle {
                mesh: meshes.add(Circle::default()).into(),
                transform: Transform::from_translation(position)
                    .with_scale(Vec3::splat(properites.0)),
                material: materials.add(properites.1),
                ..default()
            },
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                position: WindowPosition::Centered(MonitorSelection::Current),
                resolution: WindowResolution::new(800.0, 800.0),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(TrackCursorPlugin)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin::default())
        // .add_plugins(ResourceInspectorPlugin::<Configuration>::default())
        // .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default()
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_ball, fusion, gravity, cursor_tracking).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    //next cursor
    commands.spawn((CursorBundle::rand(&mut meshes, &mut materials), NextCursor));
    //ball cursor
    commands.spawn((CursorBundle::rand(&mut meshes, &mut materials), BallCursor));
}

const CURSOR_ORBIT: f32 = 350.0;
const TRACKING_SPEED: f32 = 8.0;

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

const GRAVITY_FORCE: f32 = 100.0;

fn gravity(mut query: Query<(&mut LinearVelocity, &Transform), With<Ball>>) {
    for (mut velocity, trasform) in &mut query {
        let mut dir = Vec3::default() - trasform.translation;
        if dir.length() > 20. {
            dir = dir.normalize_or_zero() * GRAVITY_FORCE;
            velocity.0 += dir.xy();
        }
    }
}

fn fusion(
    mut commands: Commands,
    mut collision_event_reader: EventReader<CollisionStarted>,
    query: Query<(&mut Ball, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for &CollisionStarted(e1, e2) in collision_event_reader.read() {
        if let (Ok((ball1, trasnform1)), Ok((ball2, transform2))) = (query.get(e1), query.get(e2)) {
            if ball1 == ball2 {
                commands.entity(e1).despawn();
                commands.entity(e2).despawn();
                commands.spawn(BallBundle::new(
                    (trasnform1.translation + transform2.translation) / 2.,
                    &mut meshes,
                    &mut materials,
                    ball1.0.next(),
                ));
            }
        }
    }
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Query<(Entity, &mut Transform, &BallType), With<BallCursor>>,
    next_cursor: Query<Entity, With<NextCursor>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Ok((current, cursor, balltype)) = cursor.get_single() {
            commands.spawn(BallBundle::new(
                cursor.translation,
                &mut meshes,
                &mut materials,
                Ball(*balltype),
            ));

            commands.entity(current).despawn();

            if let Ok((entity)) = next_cursor.get_single() {
                commands.entity(entity).insert(BallCursor);
                commands.entity(entity).remove::<NextCursor>();

                commands.spawn((CursorBundle::rand(&mut meshes, &mut materials), NextCursor));
            }   
        }
    }
}
