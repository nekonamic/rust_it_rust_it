use std::path::PathBuf;

use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    window::WindowResolution,
};
use bms_rs::bms::{BmsOutput, parse_bms, prelude::KeyLayoutBeat};
use encoding_rs::SHIFT_JIS;
use num_traits::ToPrimitive;

const LANE_HEIGHT: f32 = 722.;
const LANE_WIDTH: f32 = 432.;
const WHITE_NOTE_HEIGHT: f32 = 12.;
const WHITE_NOTE_WIDTH: f32 = 52.;
const BLUE_NOTE_HEIGHT: f32 = 12.;
const BLUE_NOTE_WIDTH: f32 = 40.;
const SCRATCH_HEIGHT: f32 = 12.;
const SCRATCH_WIDTH: f32 = 90.;

const BORDER_THICKNESS: f32 = 2.;
const JUDGEMENTLINE_THICKNESS: f32 = 4.;

const JUDGEMENTLINE_POSITION: Vec2 = Vec2::new(
    0.,
    (1080. / 2.) - (LANE_HEIGHT - JUDGEMENTLINE_THICKNESS / 2.),
);
const LEFT_BORDER_POSITION: Vec2 = Vec2::new(
    -(LANE_WIDTH / 2. + BORDER_THICKNESS / 2.),
    1080. / 2. - LANE_HEIGHT / 2.,
);
const RIGHT_BORDER_POSITION: Vec2 = Vec2::new(
    LANE_WIDTH / 2. + BORDER_THICKNESS / 2.,
    1080. / 2. - LANE_HEIGHT / 2.,
);
const BOTTOM_BORDER_POSITION: Vec2 =
    Vec2::new(0., (1080. / 2.) - (LANE_HEIGHT + BORDER_THICKNESS / 2.));

const NOTE_GAP: f32 = 2.;
const SCRATCH_L2R_RELATIVE_X: f32 = -(LANE_WIDTH / 2.) + SCRATCH_WIDTH / 2.;
const NOTE1_L2R_RELATIVE_X: f32 =
    -(LANE_WIDTH / 2.) + SCRATCH_WIDTH + NOTE_GAP + WHITE_NOTE_WIDTH / 2.;
const NOTE2_L2R_RELATIVE_X: f32 =
    -(LANE_WIDTH / 2.) + SCRATCH_WIDTH + NOTE_GAP * 2. + WHITE_NOTE_WIDTH + BLUE_NOTE_WIDTH / 2.;
const NOTE3_L2R_RELATIVE_X: f32 = -(LANE_WIDTH / 2.)
    + SCRATCH_WIDTH
    + NOTE_GAP * 3.
    + WHITE_NOTE_WIDTH
    + BLUE_NOTE_WIDTH
    + WHITE_NOTE_WIDTH / 2.;
const NOTE4_L2R_RELATIVE_X: f32 = -(LANE_WIDTH / 2.)
    + SCRATCH_WIDTH
    + NOTE_GAP * 4.
    + WHITE_NOTE_WIDTH * 2.
    + BLUE_NOTE_WIDTH
    + BLUE_NOTE_WIDTH / 2.;
const NOTE5_L2R_RELATIVE_X: f32 = -(LANE_WIDTH / 2.)
    + SCRATCH_WIDTH
    + NOTE_GAP * 5.
    + WHITE_NOTE_WIDTH * 2.
    + BLUE_NOTE_WIDTH * 2.
    + WHITE_NOTE_WIDTH / 2.;
const NOTE6_L2R_RELATIVE_X: f32 = -(LANE_WIDTH / 2.)
    + SCRATCH_WIDTH
    + NOTE_GAP * 6.
    + WHITE_NOTE_WIDTH * 3.
    + BLUE_NOTE_WIDTH * 2.
    + BLUE_NOTE_WIDTH / 2.;
const NOTE7_L2R_RELATIVE_X: f32 = -(LANE_WIDTH / 2.)
    + SCRATCH_WIDTH
    + NOTE_GAP * 7.
    + WHITE_NOTE_WIDTH * 3.
    + BLUE_NOTE_WIDTH * 3.
    + WHITE_NOTE_WIDTH / 2.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust*it!!Rust*it!!".into(),
                name: Some("bevy.app".into()),
                resolution: WindowResolution::new(1920, 1080).with_scale_factor_override(1.),
                ..default()
            }),
            ..default()
        }))
        .add_systems(
            Startup,
            (
                spawn_camera,
                spawn_judgement_line,
                spawn_lane_border,
                spawn_notes,
            ),
        )
        .add_systems(Update, (keyboard_events, notes_fall))
        .insert_resource(Config {
            speed: 800.,
            bpm: 0.,
        })
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Resource)]
struct Config {
    pub speed: f32,
    pub bpm: f32,
}

#[derive(Component)]
struct JudgementLine;

#[derive(Component)]
struct LaneBorder;

fn spawn_judgement_line(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Rectangle::new(LANE_WIDTH + 4., 4.));
    let material = materials.add(Color::srgb(1., 0., 0.));

    commands.spawn((
        JudgementLine,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Transform::from_translation(JUDGEMENTLINE_POSITION.extend(0.)),
    ));
}

fn spawn_lane_border(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let color = materials.add(Color::srgb(1., 1., 1.));

    commands
        .spawn((
            LaneBorder,
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .with_children(|parent| {
            // 下
            parent.spawn((
                Mesh2d(meshes.add(Rectangle::new(
                    LANE_WIDTH + BORDER_THICKNESS * 2.,
                    BORDER_THICKNESS,
                ))),
                MeshMaterial2d(color.clone()),
                Transform::from_translation(BOTTOM_BORDER_POSITION.extend(0.)),
            ));

            // 左
            parent.spawn((
                Mesh2d(meshes.add(Rectangle::new(BORDER_THICKNESS, LANE_HEIGHT))),
                MeshMaterial2d(color.clone()),
                Transform::from_translation(LEFT_BORDER_POSITION.extend(0.)),
            ));

            // 右
            parent.spawn((
                Mesh2d(meshes.add(Rectangle::new(BORDER_THICKNESS, LANE_HEIGHT))),
                MeshMaterial2d(color.clone()),
                Transform::from_translation(RIGHT_BORDER_POSITION.extend(0.)),
            ));
        });
}

fn keyboard_events(mut evr_kbd: MessageReader<KeyboardInput>) {
    for ev in evr_kbd.read() {
        match ev.state {
            ButtonState::Pressed => {
                println!("Key press: {:?} ({:?})", ev.key_code, ev.logical_key);
            }
            ButtonState::Released => {
                println!("Key release: {:?} ({:?})", ev.key_code, ev.logical_key);
            }
        }
    }
}

#[derive(Component)]
pub struct Note {
    pub lane: u16,
    pub time: f32,
    pub position_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgeResult {
    PGreat,
    Great,
    Good,
    Bad,
    Poor,
}

#[derive(Resource)]
pub struct HitWindow {
    pub pgreat: f32,
    pub great: f32,
    pub good: f32,
    pub bad: f32,
    pub poor: f32,
}

fn spawn_notes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut config: ResMut<Config>,
) {
    let bytes = std::fs::read("bms/BLOODY ROSE/BLOODY ROSE_Unfulfilled.bms").unwrap();
    let (source, _encoding_used, _had_errors) = SHIFT_JIS.decode(&bytes);
    let source = source.into_owned();
    let BmsOutput { bms, .. }: BmsOutput<KeyLayoutBeat> = parse_bms(&source);

    println!("Title: {}", bms.header.title.as_deref().unwrap(),);

    let wav_files = bms.notes.wav_files.clone();
    for (.., mut pathbuf) in wav_files {
        pathbuf = PathBuf::from("bms/BLOODY ROSE").join(pathbuf);
    }

    config.bpm = bms.arrangers.bpm.unwrap().to_f32().unwrap();
    let beat_interval = 60. * 4. / config.bpm;

    let judgement_y = (1080. / 2.) - (LANE_HEIGHT - JUDGEMENTLINE_THICKNESS / 2.);
    let white_note_color = materials.add(Color::srgb(1., 1., 1.));
    let blue_note_color = materials.add(Color::srgb(0., 0., 1.));
    let scratch_color = materials.add(Color::srgb(1., 0., 0.));

    let all_note = bms.notes.all_notes();
    for wav_obj in all_note {
        let note_track_with_fraction: f32 = wav_obj.offset.track.0 as f32
            + wav_obj.offset.numerator as f32 / wav_obj.offset.denominator as f32;
        let note_time = note_track_with_fraction * beat_interval;
        let position_y = judgement_y + (note_time * config.speed);
        let lane = wav_obj.channel_id.as_u16();
        if lane == 68 {
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(SCRATCH_WIDTH, SCRATCH_HEIGHT))),
                MeshMaterial2d(scratch_color.clone()),
                Transform::from_translation(
                    Vec2::new(SCRATCH_L2R_RELATIVE_X, position_y).extend(0.),
                ),
                Note {
                    lane,
                    time: note_time,
                    position_y,
                },
            ));
        } else if lane == 63 {
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(WHITE_NOTE_WIDTH, WHITE_NOTE_HEIGHT))),
                MeshMaterial2d(white_note_color.clone()),
                Transform::from_translation(Vec2::new(NOTE1_L2R_RELATIVE_X, position_y).extend(0.)),
                Note {
                    lane,
                    time: note_time,
                    position_y,
                },
            ));
        } else if lane == 64 {
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(BLUE_NOTE_WIDTH, BLUE_NOTE_HEIGHT))),
                MeshMaterial2d(blue_note_color.clone()),
                Transform::from_translation(Vec2::new(NOTE2_L2R_RELATIVE_X, position_y).extend(0.)),
                Note {
                    lane,
                    time: note_time,
                    position_y,
                },
            ));
        } else if lane == 65 {
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(BLUE_NOTE_WIDTH, BLUE_NOTE_HEIGHT))),
                MeshMaterial2d(white_note_color.clone()),
                Transform::from_translation(Vec2::new(NOTE3_L2R_RELATIVE_X, position_y).extend(0.)),
                Note {
                    lane,
                    time: note_time,
                    position_y,
                },
            ));
        } else if lane == 66 {
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(BLUE_NOTE_WIDTH, BLUE_NOTE_HEIGHT))),
                MeshMaterial2d(blue_note_color.clone()),
                Transform::from_translation(Vec2::new(NOTE4_L2R_RELATIVE_X, position_y).extend(0.)),
                Note {
                    lane,
                    time: note_time,
                    position_y,
                },
            ));
        } else if lane == 67 {
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(BLUE_NOTE_WIDTH, BLUE_NOTE_HEIGHT))),
                MeshMaterial2d(white_note_color.clone()),
                Transform::from_translation(Vec2::new(NOTE5_L2R_RELATIVE_X, position_y).extend(0.)),
                Note {
                    lane,
                    time: note_time,
                    position_y,
                },
            ));
        } else if lane == 70 {
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(BLUE_NOTE_WIDTH, BLUE_NOTE_HEIGHT))),
                MeshMaterial2d(blue_note_color.clone()),
                Transform::from_translation(Vec2::new(NOTE6_L2R_RELATIVE_X, position_y).extend(0.)),
                Note {
                    lane,
                    time: note_time,
                    position_y,
                },
            ));
        } else if lane == 71 {
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(BLUE_NOTE_WIDTH, BLUE_NOTE_HEIGHT))),
                MeshMaterial2d(white_note_color.clone()),
                Transform::from_translation(Vec2::new(NOTE7_L2R_RELATIVE_X, position_y).extend(0.)),
                Note {
                    lane,
                    time: note_time,
                    position_y,
                },
            ));
        } else {
        }
    }
}

fn notes_fall(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Note)>,
    config: Res<Config>,
) {
    let delta = time.delta_secs();
    let judgement_y = (1080. / 2.) - (LANE_HEIGHT - JUDGEMENTLINE_THICKNESS / 2.);

    for (entity, mut transform, mut note) in query.iter_mut() {
        transform.translation.y -= config.speed * delta;
        note.position_y = transform.translation.y;

        if transform.translation.y <= judgement_y {
            // 在这里销毁该 note 实体
            commands.entity(entity).despawn();
        }
    }
}

fn auto_play(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Note)>,
    config: ResMut<Config>,
) {
    let delta = time.delta_secs();
    let judgement_y = (1080. / 2.) - (LANE_HEIGHT - JUDGEMENTLINE_THICKNESS / 2.);

    for (mut transform, mut note) in query.iter_mut() {
        transform.translation.y -= config.speed * delta;
        note.position_y = transform.translation.y;

        if transform.translation.y <= judgement_y {
            transform.translation.y = judgement_y;
        }
    }
}
