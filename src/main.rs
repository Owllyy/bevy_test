use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_cursor::prelude::*;
use bevy_inspector_egui::{
    inspector_options::ReflectInspectorOptions, quick::ResourceInspectorPlugin, InspectorOptions,
};
use bevy_xpbd_2d::{
    components::{AngularDamping, CoefficientCombine, LinearDamping, LinearVelocity, Mass, RigidBody}, constraints::PenetrationConstraint, plugins::{
        collision::{
            contact_reporting::{Collision, CollisionStarted},
            Collider,
        }, solver::PenetrationConstraints, PhysicsDebugPlugin, PhysicsPlugins, SolverPlugin
    }, resources::Gravity
};

// pub struct OwnSolverPlugin;

// impl Plugin for OwnSolverPlugin {
//     fn build(&self, app: &mut App) {
//         app.init_resource::<PenetrationConstraints>();

//         let substeps = app
//             .get_schedule_mut(SubstepSchedule)
//             .expect("add SubstepSchedule first");

//         substeps.add_systems(
//             (
//                 penetration_constraints,
//                 solve_constraint::<FixedJoint, 2>,
//                 solve_constraint::<RevoluteJoint, 2>,
//                 solve_constraint::<SphericalJoint, 2>,
//                 solve_constraint::<PrismaticJoint, 2>,
//                 solve_constraint::<DistanceJoint, 2>,
//             )
//                 .chain()
//                 .in_set(SubstepSet::SolveConstraints),
//         );

//         substeps.add_systems((update_lin_vel, update_ang_vel).in_set(SubstepSet::UpdateVelocities));

//         substeps.add_systems(
//             (
//                 solve_vel,
//                 joint_damping::<FixedJoint>,
//                 joint_damping::<RevoluteJoint>,
//                 joint_damping::<SphericalJoint>,
//                 joint_damping::<PrismaticJoint>,
//                 joint_damping::<DistanceJoint>,
//             )
//                 .chain()
//                 .in_set(SubstepSet::SolveVelocities),
//         );

//         substeps.add_systems(store_contact_impulses.in_set(SubstepSet::StoreImpulses));

//         substeps.add_systems(apply_translation.in_set(SubstepSet::ApplyTranslation));
//     }
// }

#[derive(Default, Component, Reflect)]
struct Player {
    speed: Vec2,
}

#[derive(PartialEq, Default)]
enum BallType {
    #[default] GRAPE,
    APPLE,
    LEMON,
    KAKI,
    ORANGE,
}

#[derive(Default, Component, PartialEq)]
struct Ball(BallType);

impl Ball {
    fn properties(&self) -> (f32, Color) {
        match self.0 {
            BallType::GRAPE => (50., Color::BLUE),
            BallType::APPLE => (75., Color::RED),
            BallType::LEMON => (100., Color::LIME_GREEN),
            BallType::KAKI => (150., Color::BISQUE),
            BallType::ORANGE => (200., Color::ORANGE),
        }
    }

    fn next(&self) -> Ball {
        match self.0 {
            BallType::GRAPE => Ball(BallType::APPLE),
            BallType::APPLE => Ball(BallType::LEMON),
            BallType::LEMON => Ball(BallType::KAKI),
            BallType::KAKI => Ball(BallType::ORANGE),
            BallType::ORANGE => Ball(BallType::GRAPE),
        }
    }
}

#[derive(Default, Component, Reflect)]
struct Dash {
    speed: Vec2,
    duration: f32,
}

#[derive(Default, Component)]
struct Direction(Vec2);

#[derive(Default, Bundle)]
struct WallBundle {
    rigid_body: RigidBody,
    collider: Collider,
    material: MaterialMesh2dBundle<ColorMaterial>,
}

impl WallBundle {
    fn new(
        rectangle: Rectangle,
        position: Vec3,
        color: Handle<ColorMaterial>,
        meshes: &mut Assets<Mesh>,
    ) -> WallBundle {
        WallBundle {
            rigid_body: RigidBody::Static,
            collider: Collider::rectangle(1., 1.),
            material: MaterialMesh2dBundle {
                mesh: meshes.add(Rectangle::default()).into(),
                transform: Transform::from_translation(position)
                    .with_scale(rectangle.size().extend(1.)),
                material: color,
                ..default()
            },
        }
    }
}

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
        .add_plugins(PhysicsDebugPlugin::default())
        // .add_plugins(ResourceInspectorPlugin::<Configuration>::default())
        // .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
        .init_resource::<Configuration>()
        // .insert_resource(Gravity(Vec2::NEG_Y * 800.0))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                keyboard_input_system,
                look_cursor,
                apply_force,
                dash,
                spawn_ball,
                fusion,
                gravity,
            )
                .chain(),
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
    // commands.spawn((
    //     Player { speed: Vec2::ZERO },
    //     Direction(Vec2::default()),
    //     AngularDamping(100.),
    //     RigidBody::Dynamic,
    //     Collider::triangle(
    //         Vec2::new(-0.5, -0.5),
    //         Vec2::new(0.5, 0.0),
    //         Vec2::new(-0.5, 0.5),
    //     ),
    //     MaterialMesh2dBundle {
    //         mesh: meshes
    //             .add(Triangle2d::new(
    //                 Vec2::new(0.5, 0.0),
    //                 Vec2::new(-0.5, -0.5),
    //                 Vec2::new(-0.5, 0.5),
    //             ))
    //             .into(),
    //         transform: Transform::default().with_scale(Vec3::splat(64.)),
    //         material: materials.add(Color::PURPLE),
    //         ..default()
    //     },
    // ));
}

const GRAVITY_FORCE: f32 = 10000.0;

fn gravity(
mut query: Query<(&mut LinearVelocity, &Transform), With<Ball>>) {
    for  (mut velocity, trasform) in &mut query {
        let mut dir = Vec3::default() - trasform.translation;
        if dir.length() > 5. {
            dir = dir.normalize_or_zero() * GRAVITY_FORCE / dir.length();
            velocity.0 += dir.xy();
        }
    }
}

fn fusion(
    mut commands: Commands,
    mut collision_event_reader: EventReader<CollisionStarted>,
    mut query: Query<(&mut Ball, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for &CollisionStarted(e1, e2) in collision_event_reader.read() {
        if let (Ok((ball1, trasnform1)), Ok((ball2, trasnform2))) = (query.get(e1), query.get(e2)) {
            if ball1 == ball2 {
                commands.entity(e1).despawn();
                commands.entity(e2).despawn();
                let properties = ball1.next().properties();
                commands.spawn((
                    ball1.next(),
                    RigidBody::Dynamic,
                    CoefficientCombine::Min,
                    Collider::circle(0.5),
                    AngularDamping(20.),
                    LinearDamping(20.),
                    MaterialMesh2dBundle {
                        mesh: meshes.add(Circle::default()).into(),
                        transform: Transform::from_translation(trasnform1.translation)
                            .with_scale(Vec3::splat(properties.0)),
                        material: materials.add(properties.1),
                        ..default()
                    },
                ));
            }
        }
    }
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cursor: Res<CursorLocation>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = cursor.world_position() {
            let ball = Ball::default();
            commands.spawn((
                ball,
                RigidBody::Dynamic,
                Collider::circle(0.5),
                AngularDamping(20.),
                LinearDamping(20.),
                MaterialMesh2dBundle {
                    mesh: meshes.add(Circle::default()).into(),
                    transform: Transform::from_translation(Vec3::new(cursor_pos.x, 300., 0.))
                        .with_scale(Vec3::splat(Ball::default().properties().0)),
                    material: materials.add(Ball::default().properties().1),
                    ..default()
                },
            ));
        }
    }
}

fn wall_e(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((WallBundle::new(
        Rectangle::new(20., 850.),
        Vec3 {
            x: -600.,
            y: 0.,
            z: 0.0,
        },
        materials.add(Color::PURPLE),
        &mut meshes,
    ),));
    commands.spawn((WallBundle::new(
        Rectangle::new(20., 850.),
        Vec3 {
            x: 600.,
            y: 0.,
            z: 0.0,
        },
        materials.add(Color::PURPLE),
        &mut meshes,
    ),));
    commands.spawn((WallBundle::new(
        Rectangle::new(1200., 20.),
        Vec3 {
            x: 0.,
            y: -350.,
            z: 0.0,
        },
        materials.add(Color::PURPLE),
        &mut meshes,
    ),));
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
