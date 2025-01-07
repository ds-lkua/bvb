use std::thread::spawn;

use bevy::ecs::schedule::SystemSet;
use bevy::input::common_conditions::{input_just_released, input_pressed};
use bevy::prelude::*;
use bevy::render::camera;
use bevy::render::view::window;
use bevy::state::commands;
use bevy::ui::update;
#[cfg(target_os = "macos")]
use bevy::window::CompositeAlphaMode;
use bevy::window::{CursorOptions, PrimaryWindow, WindowLevel, WindowMode};
use ops::cos;

fn main() {
    let window = Window {
        // Enable transparent support for the window
        transparent: true,
        // composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
        decorations: false,
        window_level: WindowLevel::AlwaysOnTop,
        #[cfg(target_os = "macos")]
        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        // cursor_options: CursorOptions {
        // hit_test: false,
        // ..Default::default()
        // },
        ..default()
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..Default::default()
        }))
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins(HelloPlugin)
        .add_event::<BulletEvent>()
        .insert_resource(MyTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .insert_resource(UpdateTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .add_systems(Startup, setup_gif)
        .add_systems(Startup, setup_cam)
        .add_systems(
            Update,
            (
                do_chhh,
                pos_bullet.run_if(input_just_released(MouseButton::Left)),
                pre_bullet.run_if(input_pressed(MouseButton::Left)),
                move_bullet,
                remove_bullet,
                play_sp,
            ),
        )
        .add_systems(PostUpdate, update_bullet_count)
        .add_systems(PreUpdate, show_bullet_count)
        .run();
}

#[derive(Component)]
struct AmIdx {
    first: usize,
    last: usize,
}

#[derive(Component)]
struct AnTm(Timer);

fn setup_gif(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tal: ResMut<Assets<TextureAtlasLayout>>,
) {
    let tx = asset_server.load("s1.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 1, None, None);
    let tall = tal.add(layout);
    let idx = AmIdx { first: 1, last: 5 };

    commands.spawn((
        Sprite::from_atlas_image(
            tx,
            TextureAtlas {
                layout: tall,
                index: idx.first,
            },
        ),
        Transform::from_scale(Vec3::splat(6.0)),
        idx,
        AnTm(Timer::from_seconds(0.5, TimerMode::Repeating)),
    ));
}

fn play_sp(time: Res<Time>, mut query: Query<(&mut AnTm, &mut AmIdx, &mut Sprite)>) {
    for (mut tm, idx, mut sp) in &mut query {
        tm.0.tick(time.delta());
        if tm.0.just_finished() {
            if let Some(ats) = &mut sp.texture_atlas {
                ats.index = if ats.index == idx.last {
                    idx.first
                } else {
                    ats.index + 1
                };
            }
        }
    }
}

#[derive(Resource)]
struct MyTimer(Timer);

#[derive(Resource)]
struct UpdateTimer(Timer);

#[derive(Component)]
struct MyCircleHandle {
    shape: Handle<Mesh>,
    color: Handle<ColorMaterial>,
}

#[derive(Component)]
struct BulletCount {
    num: i32,
}

#[derive(Component)]
struct MyCircle {
    size: f32,
}

#[derive(Component)]
struct Target;

#[derive(Component)]
struct Bullet {
    speed: f32,
    x_scale: f32,
    y_scale: f32,
}

#[derive(Component)]
struct PreBullet {
    size: f32,
    grow: f32,
}

fn setup_cam(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let color_handle = materials.add(Color::rgb(0.8, 0.7, 0.5));
    let shape = meshes.add(Circle::new(50.0));
    let handles = MyCircleHandle {
        shape: shape.clone(),
        color: color_handle.clone(),
    };
    let qq = (
        Mesh2d(shape),
        MeshMaterial2d(color_handle),
        Target,
        MyCircle { size: 50.0 },
    );
    commands.spawn(Camera2d);
    commands.spawn(qq);
    commands.spawn(handles);
    commands.spawn((
        Text::new("bullets: 0"),
        TextLayout::new_with_justify(JustifyText::Left),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        BulletCount { num: 0 },
    ));
}

fn show_bullet_count(mut text: Single<(&mut Text, &BulletCount)>) {
    text.0 .0 = format!("bullets: {}", text.1.num);
}

fn get_scale(xd: f32, yd: f32) -> f32 {
    if yd != 0.0 {
        xd / yd
    } else {
        0.0
    }
}

fn calc_xy_move(xd: f32, yd: f32, moved: f32) -> (f32, f32) {
    let xx = (xd.powi(2) + yd.powi(2)).sqrt();
    if xx == 0.0 {
        return (0.0, 0.0);
    }
    let mut x_move = xd / xx * moved;
    if x_move.abs() > xd.abs() {
        x_move = xd;
    }
    let mut y_move = yd / xx * moved;
    if y_move.abs() > yd.abs() {
        y_move = yd
    }
    (x_move, y_move)
}

fn pre_bullet(
    mut commands: Commands,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    mut shapes: Query<(Entity, &mut Transform, &mut Mesh2d, &mut PreBullet), With<PreBullet>>,
    time: Res<Time>,
    mut timer: ResMut<UpdateTimer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for mut shape in shapes.iter_mut() {
        if !timer.0.tick(time.delta()).just_finished() {
            return;
        }
        shape.3.grow += 1.0;
        let grow = 1.0 + shape.3.grow;
        shape.1.scale = Vec3::new(grow, grow, 0.0);
        return;
    }

    let cursor_pos = match window_query.cursor_position() {
        Some(pos) => pos,
        None => return,
    };
    let world_pos = camera_query.0.viewport_to_world(camera_query.1, cursor_pos);
    let pp = world_pos.unwrap().origin.trunc();
    let shape = meshes.add(Circle::new(1.0));
    let color_handle = materials.add(Color::rgb(0.4, 0.2, 0.7));
    let qq = (
        Mesh2d(shape),
        MeshMaterial2d(color_handle),
        PreBullet {
            size: 1.0,
            grow: 1.0,
        },
        Transform::from_xyz(pp.x, pp.y, pp.z),
    );
    commands.spawn(qq);
}

#[derive(Event)]
enum BulletEvent {
    Add,
    Delete,
}

fn pos_bullet(
    mut commands: Commands,
    mut ev: EventWriter<BulletEvent>,
    target_shape: Single<(&Transform, &Mesh2d), With<Target>>,
    shapes: Query<(Entity, &mut Transform, &mut Mesh2d), (With<PreBullet>, Without<Target>)>,
) {
    for shape in shapes.iter() {
        let t = target_shape.0.translation;
        let x_dist = t.x - shape.1.translation.x;
        let y_dist = t.y - shape.1.translation.y;
        let dist = (x_dist.powi(2) + y_dist.powi(2)).sqrt();
        let x_scale = if dist == 0.0 { 1.0 } else { x_dist / dist };
        let y_scale = if dist == 0.0 { 1.0 } else { y_dist / dist };
        commands
            .entity(shape.0)
            .remove::<PreBullet>()
            .insert(Bullet {
                // speed: 5.0 + 0.1 * shape.3.grow,
                speed: 5.0,
                y_scale,
                x_scale,
            });
        ev.send(BulletEvent::Add);
    }
}

fn move_bullet(mut shapes: Query<(&mut Transform, &Bullet), With<Bullet>>) {
    for mut shape in shapes.iter_mut() {
        shape.0.translation.x += shape.1.speed * shape.1.x_scale;
        shape.0.translation.y += shape.1.speed * shape.1.y_scale;
    }
}

fn remove_bullet(
    mut commands: Commands,
    mut ev: EventWriter<BulletEvent>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    shapes: Query<(Entity, &Transform, &Bullet), With<Bullet>>,
) {
    let max_x = window_query.width();
    let max_y = window_query.height();
    for shape in shapes.iter() {
        let (x, y, z) = (
            shape.1.translation.x,
            shape.1.translation.y,
            shape.1.translation.z,
        );
        let pos = camera_query
            .0
            .world_to_viewport(camera_query.1, Vec3::new(x, y, z));
        if let Ok(post) = pos {
            if post.x >= 0.0 && post.y >= 0.0 && post.x <= max_x && post.y <= max_y {
                continue;
            }
            commands.entity(shape.0).despawn();
            ev.send(BulletEvent::Delete);
        }
    }
}

fn update_bullet_count(mut evs: EventReader<BulletEvent>, mut bc: Single<&mut BulletCount>) {
    for ev in evs.read() {
        match ev {
            BulletEvent::Add => bc.num += 1,
            BulletEvent::Delete => bc.num -= 1,
        }
    }
}

fn do_chhh(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    mut shape: Single<&mut Transform, With<Target>>,
    time: Res<Time>,
    mut timer: ResMut<MyTimer>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        // return;
    }

    let cursor_pos = match window_query.cursor_position() {
        Some(pos) => pos,
        None => return,
    };
    let world_pos = camera_query.0.viewport_to_world(camera_query.1, cursor_pos);
    let pp = world_pos.unwrap().origin.trunc();
    let step = 10.0;
    let old_pos = shape.translation;
    let mut x_dist = pp.x - shape.translation.x;
    let mut y_dist = pp.y - shape.translation.y;
    let mut scale = get_scale(x_dist, y_dist).abs();
    scale = if scale >= 1.0 { scale } else { 1.0 };
    let (x_move, y_move) = calc_xy_move(x_dist, y_dist, step);
    shape.translation.x += x_move;
    shape.translation.y += y_move;
    // if x_dist.abs() < x_move {
    //     shape.translation.x = pp.x;
    // } else {
    //     shape.translation.x += x_move;
    // }
    // if y_dist.abs() < y_move {
    //     shape.translation.y = pp.y;
    // } else {
    //     shape.translation.y += y_move;
    // }

    return;
    if shape.translation.x > pp.x {
        shape.translation.x -= step;
        if shape.translation.x < pp.x {
            shape.translation.x = pp.x
        }
    } else if shape.translation.x < pp.x {
        shape.translation.x += step;
        if shape.translation.x > pp.x {
            shape.translation.x = pp.x
        }
    };
    if shape.translation.y > pp.y {
        shape.translation.y -= step;
        if shape.translation.y < pp.y {
            shape.translation.y = pp.y
        }
    } else if shape.translation.y < pp.y {
        shape.translation.y += step;
        if shape.translation.y > pp.y {
            shape.translation.y = pp.y
        }
    }
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    // update our timer with the time elapsed since the last update
    // if that caused the timer to finish, we say hello to everyone
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0);
        }
    }
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Elaina Proctor" {
            name.0 = "Elaina Hume".to_string();
            break; // We don't need to change any other names.
        }
    }
}

#[derive(Resource)]
struct GreetTimer(Timer);

pub struct HelloPlugin;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
struct AA;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        ((update_people, greet_people).chain()).in_set(AA);
        // add things to your app here
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Startup, add_people);
    }
}
