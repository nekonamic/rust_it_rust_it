use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use bevy::{
    asset::UnapprovedPathMode,
    input::keyboard::KeyboardInput,
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use bevy_kira_audio::prelude::*;
use bms_rs::{
    bms::{BmsOutput, parse_bms, prelude::KeyLayoutBeat},
    command::ObjId,
};
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
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Rust*it!!Rust*it!!".into(),
                        name: Some("bevy.app".into()),
                        resolution: WindowResolution::new(1920, 1080)
                            .with_scale_factor_override(1.),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                }),
            AudioPlugin,
        ))
        .insert_state(AppState::Loading)
        .add_systems(
            Startup,
            (
                spawn_camera,
                spawn_judgement_line,
                spawn_lane_border,
                spawn_notes,
            ),
        )
        .add_systems(
            Update,
            (controller, notes_fall.run_if(in_state(AppState::Playing))),
        )
        .insert_resource(Config {
            speed: 800.,
            bpm: 0.,
        })
        .insert_resource(GameState { start_time: 0.0 })
        .add_systems(FixedUpdate, print_key_input)
        .insert_resource(Time::<Fixed>::from_hz(1000.0))
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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Playing,
}

#[derive(Resource)]
struct GameState {
    start_time: f32,
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

fn controller(
    mut next_state: ResMut<NextState<AppState>>,
    input: Res<ButtonInput<KeyCode>>,
    mut game_status: ResMut<GameState>,
    time: Res<Time>,
) {
    if input.pressed(KeyCode::Space) {
        next_state.set(AppState::Playing);
        game_status.start_time = time.elapsed_secs();
    }
}

#[derive(Component)]
pub struct Note {
    pub lane: u16,
    pub time: f32,
    pub position_y: f32,
    pub wav_file: ObjId,
    pub is_note: bool,
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

#[derive(Resource)]
struct AudioAssets {
    map: HashMap<ObjId, Handle<AudioSource>>,
}

fn find_wav(path: &str) -> Option<PathBuf> {
    let path = Path::new(path);
    let parent = path.parent()?;
    let stem = path.file_stem()?.to_string_lossy();

    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with(&*stem) {
                return Some(entry.path());
            }
        }
    }

    None
}

fn spawn_notes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut config: ResMut<Config>,
    asset_server: Res<AssetServer>,
) {
    let bytes = std::fs::read("bms/BLOODY ROSE/BLOODY ROSE_Unfulfilled.bms").unwrap();
    let (source, _encoding_used, _had_errors) = SHIFT_JIS.decode(&bytes);
    let source = source.into_owned();
    let BmsOutput { bms, .. }: BmsOutput<KeyLayoutBeat> = parse_bms(&source);

    println!("Title: {}", bms.header.title.as_deref().unwrap(),);

    let wav_files = bms.notes.wav_files.clone();

    let mut audio_map = HashMap::new();
    for (id, pathbuf) in wav_files {
        let related_path = PathBuf::from("bms/BLOODY ROSE").join(&pathbuf);
        let abs_path = env::current_dir().unwrap().join(&related_path);

        if let Some(file) = find_wav(&abs_path.to_string_lossy()) {
            let handle: Handle<AudioSource> = asset_server.load(file);
            audio_map.insert(id, handle);
        }
    }
    commands.insert_resource(AudioAssets { map: audio_map });

    config.bpm = bms.arrangers.bpm.unwrap().to_f32().unwrap();
    let beat_interval = 60. * 4. / config.bpm;

    let white_note_color = materials.add(Color::srgb(1., 1., 1.));
    let blue_note_color = materials.add(Color::srgb(0., 0., 1.));
    let scratch_color = materials.add(Color::srgb(1., 0., 0.));

    let all_note = bms.notes.all_notes();
    for wav_obj in all_note {
        let note_track_with_fraction: f32 = wav_obj.offset.track.0 as f32
            + wav_obj.offset.numerator as f32 / wav_obj.offset.denominator as f32;
        let note_time = note_track_with_fraction * beat_interval;
        let position_y = JUDGEMENTLINE_POSITION.y + (note_time * config.speed);
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
                    wav_file: wav_obj.wav_id,
                    is_note: true,
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
                    wav_file: wav_obj.wav_id,
                    is_note: true,
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
                    wav_file: wav_obj.wav_id,
                    is_note: true,
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
                    wav_file: wav_obj.wav_id,
                    is_note: true,
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
                    wav_file: wav_obj.wav_id,
                    is_note: true,
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
                    wav_file: wav_obj.wav_id,
                    is_note: true,
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
                    wav_file: wav_obj.wav_id,
                    is_note: true,
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
                    wav_file: wav_obj.wav_id,
                    is_note: true,
                },
            ));
        } else if lane == 1 {
            commands.spawn((
                Transform::from_translation(Vec2::new(0., position_y).extend(0.)),
                Note {
                    lane,
                    time: note_time,
                    position_y,
                    wav_file: wav_obj.wav_id,
                    is_note: false,
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
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
    game_status: Res<GameState>,
) {
    let current_time = time.elapsed_secs();
    let elapsed = current_time - game_status.start_time;

    for (entity, mut transform, note) in query.iter_mut() {
        let new_pos =
            note.position_y + (JUDGEMENTLINE_POSITION.y - note.position_y) * (elapsed / note.time);
        transform.translation.y = new_pos;

        if transform.translation.y <= JUDGEMENTLINE_POSITION.y {
            commands.entity(entity).despawn();
            if let Some(handle) = audio_assets.map.get(&note.wav_file) {
                audio.play(handle.clone());
            }
        }
    }
}

fn print_key_input(
    mut keyboard_input_events: MessageReader<KeyboardInput>,
) {
    for event in keyboard_input_events.read() {
        println!(
            "Physical key: {:?}, Logical key: {:?}, Text: {:?}, State: {:?}, Repeat: {}",
            event.key_code,
            event.logical_key,
            event.text,
            event.state,
            event.repeat
        );
    }
}
