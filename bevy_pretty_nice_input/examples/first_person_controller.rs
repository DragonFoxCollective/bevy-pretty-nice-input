use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy_pretty_nice_input::{binding1d, binding2d, prelude::*};
use bevy_rapier3d::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_cursor_options: Some(CursorOptions {
                visible: false,
                grab_mode: CursorGrabMode::Locked,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(PrettyNiceInputPlugin)
        .add_systems(Startup, (setup_env, setup_player))
        .add_systems(Update, (walk, update_grounded))
        .add_observer(look)
        .add_observer(jump)
        .add_observer(exit_app)
        .run()
}

fn setup_env(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let white = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..Default::default()
    });

    commands.spawn((
        Transform::from_xyz(0.0, -1.0, 0.0),
        Collider::cuboid(50.0, 1.0, 50.0),
        Mesh3d(meshes.add(Cuboid::new(100.0, 2.0, 100.0))),
        MeshMaterial3d(white.clone()),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.0))),
        MeshMaterial3d(white.clone()),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_8,
            0.0,
        )),
    ));
}

#[derive(Component)]
pub struct Player {
    speed: f32,
    sprint_speed: f32,
    jump_force: f32,
    look_sensitivity: f32,
    camera: Entity,
    collider: Entity,
}

#[derive(Component)]
pub struct Pitch(f32);

fn setup_player(mut commands: Commands) {
    let camera = commands
        .spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: 70.0f32.to_radians(),
                ..default()
            }),
            Transform::from_xyz(0.0, 1.6, 0.0),
            Pitch(0.0),
        ))
        .id();

    let collider = commands
        .spawn((
            Collider::capsule_y(0.75, 0.25), // remember, the capsule radius is added to the height
            Transform::from_xyz(0.0, 1.0, 0.0),
        ))
        .id();

    commands
        .spawn((
            Player {
                speed: 5.0,
                sprint_speed: 10.0,
                jump_force: 5.0,
                look_sensitivity: 0.002,
                camera,
                collider,
            },
            RigidBody::Dynamic,
            Velocity::default(),
            Transform::default(),
            Visibility::default(),
            LockedAxes::ROTATION_LOCKED,
            (
                input_transition!(() <=> (>Walking), Axis2D[binding2d::wasd()]),
                input_transition!(() <=> (Sprinting), Axis1D[binding1d::left_shift()]),
                input!(Look, Axis2D[binding2d::mouse_move()]),
                input!(
                    Jump,
                    Axis1D[binding1d::space()],
                    [
                        ButtonPress::default(),
                        InputBuffer::new(0.2),
                        FilterBuffered::<Grounded>::default(),
                        Cooldown::new(0.5),
                        ResetBuffer,
                    ]
                ),
                input!(ExitApp, Axis1D[binding1d::key(KeyCode::Escape)]),
            ),
            ComponentBuffer::<Grounded>::observe(0.2),
        ))
        .add_children(&[camera, collider]);
}

#[derive(Component, TryFromActionData)]
#[action_data(Axis2D)]
pub struct Walking(Vec2);

#[derive(Component, Default)]
pub struct Sprinting;

fn walk(
    mut players: Query<(
        &mut Velocity,
        &Player,
        &GlobalTransform,
        Option<&Walking>,
        Has<Sprinting>,
    )>,
) {
    for (mut velocity, player, transform, walking, sprinting) in players.iter_mut() {
        let walk_direction = walking
            .map(|w| Vec3::new(w.0.x, 0.0, -w.0.y).clamp_length_max(1.0))
            .unwrap_or_default();
        let walk_speed = if sprinting {
            player.sprint_speed
        } else {
            player.speed
        };
        let walk_velocity = walk_direction * walk_speed;

        let vertical_velocity = velocity.linvel.project_onto(transform.up().as_vec3());
        let horizontal_velocity = transform.rotation() * walk_velocity;
        velocity.linvel = vertical_velocity + horizontal_velocity;
    }
}

#[derive(Action)]
pub struct Look;

fn look(
    look: On<Pressed<Look>>,
    mut players: Query<(&mut Transform, &Player)>,
    mut cameras: Query<(&mut Transform, &mut Pitch), Without<Player>>,
) -> Result<()> {
    let (mut player_transform, player) = players.get_mut(look.input)?;
    let (mut camera_transform, mut pitch) = cameras.get_mut(player.camera)?;

    let input = look.data.as_2d_ok()?;
    let delta_yaw = -input.x * player.look_sensitivity;
    let delta_pitch = -input.y * player.look_sensitivity;

    player_transform.rotate_local_y(delta_yaw);
    pitch.0 =
        (pitch.0 + delta_pitch).clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    camera_transform.rotation = Quat::from_axis_angle(Vec3::X, pitch.0);

    Ok(())
}

#[derive(Action)]
pub struct Jump;

fn jump(
    jump: On<JustPressed<Jump>>,
    mut players: Query<(&mut Velocity, &GlobalTransform, &Player)>,
) -> Result<()> {
    let (mut velocity, transform, player) = players.get_mut(jump.input)?;
    velocity.linvel = transform.up() * player.jump_force;
    Ok(())
}

#[derive(Component)]
pub struct Grounded;

fn update_grounded(
    mut bodies: Query<(Entity, &GlobalTransform, &Player)>,
    rapier_context: ReadRapierContext,
    mut commands: Commands,
) -> Result {
    let rapier_context = rapier_context.single()?;
    for (player, transform, body) in bodies.iter_mut() {
        let collider_entity = body.collider;
        let mut hit = false;
        rapier_context.intersect_ray(
            transform.translation() + transform.up() * 0.05,
            transform.down().into(),
            0.25,
            true,
            QueryFilter::default(),
            |collided_entity, _ray_intersection| {
                if collided_entity == collider_entity {
                    true
                } else {
                    hit = true;
                    false
                }
            },
        );
        if hit {
            commands.entity(player).insert(Grounded);
        } else {
            commands.entity(player).remove::<Grounded>();
        }
    }
    Ok(())
}

#[derive(Action)]
pub struct ExitApp;

fn exit_app(_exit: On<Pressed<ExitApp>>, mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
