use std::path::{Path, PathBuf};
use std::{env, fs};

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bms_rs::bms::model::Bms;
use bms_rs::bms::{BmsOutput, parse_bms, prelude::KeyLayoutBeat};
use bms_rs::command::ObjId;
use encoding_rs::SHIFT_JIS;
use num_traits::ToPrimitive;

use crate::resources::BmsLib;
use crate::screens::Screen;

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

struct TimingWindow {
    pgreat: f32,
    great: f32,
    good: f32,
    bad: f32,
    poor: f32,
}

const TIMING_WINDOW: TimingWindow = TimingWindow {
    pgreat: 21. / 1000.,
    great: 60. / 1000.,
    good: 120. / 1000.,
    bad: 200. / 1000.,
    poor: 1000. / 1000.,
};

#[derive(Debug, Clone, Copy)]
pub struct Segment {
    pub t_start: f32,  // 段开始时间
    pub t_end: f32,    // 段结束时间
    pub velocity: f32, // 此段速度
}

pub fn compute_position(tnote: f32, telapse: f32, mut changes: Vec<(f32, f32)>) -> f32 {
    // 1. 按时间排序
    changes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // 2. 构造 segments
    let mut segments = Vec::new();
    for i in 0..changes.len() {
        let (t_start, v) = changes[i];
        let t_end = if i + 1 < changes.len() {
            changes[i + 1].0
        } else {
            tnote
        };
        segments.push((t_start, t_end, v));
    }

    // 3. 积分速度得到位置
    let mut pos = 0.0;
    for (t_start, t_end, v) in segments {
        if telapse >= t_end {
            // 整段完全落在 telapse 之前，全部积分
            pos += (t_end - t_start) * v;
        } else if telapse > t_start {
            // telapse 落在这个 segment 内，积分到 telapse
            pos += (telapse - t_start) * v;
            break;
        } else {
            break;
        }
    }

    pos
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (
            spawn_judgement_line,
            spawn_lane_border,
            spawn_notes.after(spawn_lanes),
            spawn_lanes,
        ),
    )
    .add_systems(Update, (notes_fall).run_if(in_state(AppState::Playing)))
    .add_systems(FixedUpdate, play_bgm.run_if(in_state(AppState::Playing)))
    .add_systems(
        FixedUpdate,
        keyboard_input.run_if(in_state(Screen::Gameplay)),
    )
    .insert_resource(Time::<Fixed>::from_hz(1000.0))
    .insert_resource(KeySound {
        lane_keysound: [ObjId::null(); 8],
    })
    .add_plugins(AudioPlugin)
    .insert_state(AppState::Loading);
}

#[derive(Resource)]
struct KeySound {
    lane_keysound: [ObjId; 8],
}

#[derive(Resource)]
struct PlayStatus {
    green_number: u32,
    speed: f32,
    bpm: f32,
    start_time: f32,
}

#[derive(Resource)]
struct BmsData {
    data: Bms,
}

#[derive(Resource)]
struct AudioAssets {
    map: HashMap<ObjId, Handle<AudioSource>>,
}

#[derive(Component)]
struct JudgementLine;

#[derive(Component)]
struct LaneBorder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Lane {
    LS,
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
}

impl Lane {
    fn all() -> &'static [Lane] {
        &[
            Lane::LS,
            Lane::L1,
            Lane::L2,
            Lane::L3,
            Lane::L4,
            Lane::L5,
            Lane::L6,
            Lane::L7,
        ]
    }
}

#[derive(Component)]
struct Note {
    time: f32,
    wav_file: ObjId,
    note_track_with_fraction: f32,
}

#[derive(Component)]
struct BGMEvent {
    time: f32,
    wav_file: ObjId,
}

#[derive(Component)]
struct BPMEvent {
    bpm: f32,
    time: f32,
}

#[derive(Resource)]
struct BPMChanges(pub Vec<(f32, f32)>);

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
    asset_server: Res<AssetServer>,
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

fn find_wav(path: &str) -> Option<PathBuf> {
    let path = Path::new(path);
    let parent = path.parent()?;
    let stem = path.file_stem()?.to_string_lossy();

    let audio_exts = ["wav", "ogg"];

    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            let file_name_os = entry.file_name();
            let file_name = file_name_os.to_string_lossy();

            let file_path = entry.path();

            if file_name.starts_with(&*stem) {
                if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                    if audio_exts.contains(&ext.to_lowercase().as_str()) {
                        return Some(file_path);
                    }
                }
            }
        }
    }

    None
}

#[derive(Component)]
struct Lanes(Lane);

fn spawn_lanes(mut commands: Commands) {
    for lane in Lane::all() {
        commands.spawn((
            Lanes(*lane),
            Transform::default(),
            GlobalTransform::default(),
        ));
    }
}

#[derive(Debug, Clone)]
struct Interval {
    start: f64,
    end: f64,
    value: f64,
}

#[derive(Debug, Clone)]
struct BpmCrotchetFunction {
    intervals: Vec<Interval>,
}

impl BpmCrotchetFunction {
    fn bpm_time_function(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }

        let mut remaining = x;
        let mut acc = 0.0;
        let n = self.intervals.len();

        if n == 0 {
            return 0.0;
        }

        for (i, it) in self.intervals.iter().enumerate() {
            if remaining <= 0.0 {
                break;
            }

            if x <= it.start {
                if i == 0 {
                    return 0.0;
                } else {
                    let prev = &self.intervals[i - 1];
                    let length = x - prev.end.max(prev.start);
                    if length > 0.0 {
                        acc += 60.0 * length / prev.value;
                    }
                    return acc;
                }
            }

            let seg_start = it.start.max(0.0);
            let seg_end = it.end.min(x);
            if seg_end > seg_start {
                let len = seg_end - seg_start;
                acc += 60.0 * len / it.value;
                remaining = x - seg_end;
                if seg_end >= x {
                    return acc;
                }
            }
        }

        let last = &self.intervals[n - 1];
        if x > last.end {
            let extra_len = x - last.end;
            acc += 60.0 * extra_len / last.value;
        }

        acc
    }
}

#[derive(Debug, Clone)]
struct Crotchet {
    section_len_changes_hashmap: HashMap<u64, f64>,
}

impl Crotchet {
    pub fn get_crotchet(&self, measure_idx: u64, pos_in_measure: f64) -> f64 {
        let default_beats = 4f64;

        let mut total_beats_before = 0f64;

        for m in 0..measure_idx {
            let beats = self
                .section_len_changes_hashmap
                .get(&m)
                .copied()
                .unwrap_or(default_beats);
            total_beats_before += beats;
        }

        let current_beats = self
            .section_len_changes_hashmap
            .get(&measure_idx)
            .copied()
            .unwrap_or(default_beats);

        let beat_in_measure = pos_in_measure * current_beats as f64;

        (total_beats_before as f64) + beat_in_measure
    }
}

fn spawn_notes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    lib: ResMut<BmsLib>,
    lane_query: Query<(Entity, &Lanes)>,
) {
    let bytes = std::fs::read(lib.cursor_entry().unwrap().path.clone()).unwrap();
    let (source, _encoding_used, _had_errors) = SHIFT_JIS.decode(&bytes);
    let source = source.into_owned();
    let BmsOutput { bms, .. }: BmsOutput<KeyLayoutBeat> = parse_bms(&source);

    let wav_files = bms.notes.wav_files.clone();

    let mut audio_map = HashMap::new();
    for (id, pathbuf) in wav_files {
        if let Some(file) = find_wav(
            env::current_dir()
                .unwrap()
                .join(lib.cursor_dir().unwrap().join(&pathbuf).to_str().unwrap())
                .to_str()
                .unwrap(),
        ) {
            let handle: Handle<AudioSource> = asset_server.load(file);
            audio_map.insert(id, handle);
        }
    }
    commands.insert_resource(AudioAssets { map: audio_map });

    let mut bpm = bms.arrangers.bpm.clone().unwrap().to_f32().unwrap();
    let green_number = 500;

    let mut section_len_changes_hashmap: HashMap<u64, f64> = HashMap::new();
    let section_len_changes = &bms.arrangers.section_len_changes;

    for (_, section_len_change_obj) in section_len_changes {
        section_len_changes_hashmap.insert(
            section_len_change_obj.track.0,
            section_len_change_obj.length.to_f64().unwrap() * 4.,
        );
    }
    let crotchet = Crotchet {
        section_len_changes_hashmap,
    };

    let mut bpm_changes_interval: Vec<Interval> = vec![];
    let bpm_changes = &bms.arrangers.bpm_changes;
    for (_, bpm_change_obj) in bpm_changes {
        bpm_changes_interval.push(
            start: 
            (crotchet.get_crotchet(
                bpm_change_obj.time.track.0,
                bpm_change_obj.time.numerator as f64 / bpm_change_obj.time.denominator as f64,
            ), bpm_change_obj.bpm.to_f64().unwrap()),
        );
    }
    let bpm_crotchet_function = BpmCrotchetFunction { bpm_changes_arr };

    let white_note_color = materials.add(Color::srgb(1., 1., 1.));
    let blue_note_color = materials.add(Color::srgb(0., 0., 1.));
    let scratch_color = materials.add(Color::srgb(1., 0., 0.));

    let all_note = bms.notes.all_notes();
    for wav_obj in all_note {
        let note_track_with_fraction: f32 = wav_obj.offset.track.0 as f32
            + wav_obj.offset.numerator as f32 / wav_obj.offset.denominator as f32;
        let note_time = note_track_with_fraction * beat_interval;
        let speed = LANE_HEIGHT / (green_number as f32 / 10. / 60.);
        let position_y = JUDGEMENTLINE_POSITION.y + (note_time * speed);
        let lane = wav_obj.channel_id.as_u16();
        if lane == 68 {
            let lane_entity = lane_query
                .iter()
                .find(|(_, lane)| lane.0 == Lane::LS)
                .map(|(e, _)| e)
                .unwrap();
            commands.entity(lane_entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(SCRATCH_WIDTH, SCRATCH_HEIGHT))),
                    MeshMaterial2d(scratch_color.clone()),
                    Transform::from_translation(
                        Vec2::new(SCRATCH_L2R_RELATIVE_X, position_y).extend(0.),
                    ),
                    Note {
                        time: note_time,
                        wav_file: wav_obj.wav_id,
                        note_track_with_fraction: note_track_with_fraction,
                    },
                ));
            });
        } else if lane == 63 {
            let lane_entity = lane_query
                .iter()
                .find(|(_, lane)| lane.0 == Lane::L1)
                .map(|(e, _)| e)
                .unwrap();
            commands.entity(lane_entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(WHITE_NOTE_WIDTH, WHITE_NOTE_HEIGHT))),
                    MeshMaterial2d(white_note_color.clone()),
                    Transform::from_translation(
                        Vec2::new(NOTE1_L2R_RELATIVE_X, position_y).extend(0.),
                    ),
                    Note {
                        time: note_time,
                        wav_file: wav_obj.wav_id,
                        note_track_with_fraction: note_track_with_fraction,
                    },
                ));
            });
        } else if lane == 64 {
            let lane_entity = lane_query
                .iter()
                .find(|(_, lane)| lane.0 == Lane::L2)
                .map(|(e, _)| e)
                .unwrap();
            commands.entity(lane_entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(BLUE_NOTE_WIDTH, BLUE_NOTE_HEIGHT))),
                    MeshMaterial2d(blue_note_color.clone()),
                    Transform::from_translation(
                        Vec2::new(NOTE2_L2R_RELATIVE_X, position_y).extend(0.),
                    ),
                    Note {
                        time: note_time,
                        wav_file: wav_obj.wav_id,
                        note_track_with_fraction: note_track_with_fraction,
                    },
                ));
            });
        } else if lane == 65 {
            let lane_entity = lane_query
                .iter()
                .find(|(_, lane)| lane.0 == Lane::L3)
                .map(|(e, _)| e)
                .unwrap();
            commands.entity(lane_entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(WHITE_NOTE_WIDTH, WHITE_NOTE_HEIGHT))),
                    MeshMaterial2d(white_note_color.clone()),
                    Transform::from_translation(
                        Vec2::new(NOTE3_L2R_RELATIVE_X, position_y).extend(0.),
                    ),
                    Note {
                        time: note_time,
                        wav_file: wav_obj.wav_id,
                        note_track_with_fraction: note_track_with_fraction,
                    },
                ));
            });
        } else if lane == 66 {
            let lane_entity = lane_query
                .iter()
                .find(|(_, lane)| lane.0 == Lane::L4)
                .map(|(e, _)| e)
                .unwrap();
            commands.entity(lane_entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(BLUE_NOTE_WIDTH, BLUE_NOTE_HEIGHT))),
                    MeshMaterial2d(blue_note_color.clone()),
                    Transform::from_translation(
                        Vec2::new(NOTE4_L2R_RELATIVE_X, position_y).extend(0.),
                    ),
                    Note {
                        time: note_time,
                        wav_file: wav_obj.wav_id,
                        note_track_with_fraction: note_track_with_fraction,
                    },
                ));
            });
        } else if lane == 67 {
            let lane_entity = lane_query
                .iter()
                .find(|(_, lane)| lane.0 == Lane::L5)
                .map(|(e, _)| e)
                .unwrap();
            commands.entity(lane_entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(WHITE_NOTE_WIDTH, WHITE_NOTE_HEIGHT))),
                    MeshMaterial2d(white_note_color.clone()),
                    Transform::from_translation(
                        Vec2::new(NOTE5_L2R_RELATIVE_X, position_y).extend(0.),
                    ),
                    Note {
                        time: note_time,
                        wav_file: wav_obj.wav_id,
                        note_track_with_fraction: note_track_with_fraction,
                    },
                ));
            });
        } else if lane == 70 {
            let lane_entity = lane_query
                .iter()
                .find(|(_, lane)| lane.0 == Lane::L6)
                .map(|(e, _)| e)
                .unwrap();
            commands.entity(lane_entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(BLUE_NOTE_WIDTH, BLUE_NOTE_HEIGHT))),
                    MeshMaterial2d(blue_note_color.clone()),
                    Transform::from_translation(
                        Vec2::new(NOTE6_L2R_RELATIVE_X, position_y).extend(0.),
                    ),
                    Note {
                        time: note_time,
                        wav_file: wav_obj.wav_id,
                        note_track_with_fraction: note_track_with_fraction,
                    },
                ));
            });
        } else if lane == 71 {
            let lane_entity = lane_query
                .iter()
                .find(|(_, lane)| lane.0 == Lane::L7)
                .map(|(e, _)| e)
                .unwrap();
            commands.entity(lane_entity).with_children(|parent| {
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(WHITE_NOTE_WIDTH, WHITE_NOTE_HEIGHT))),
                    MeshMaterial2d(white_note_color.clone()),
                    Transform::from_translation(
                        Vec2::new(NOTE7_L2R_RELATIVE_X, position_y).extend(0.),
                    ),
                    Note {
                        time: note_time,
                        wav_file: wav_obj.wav_id,
                        note_track_with_fraction: note_track_with_fraction,
                    },
                ));
            });
        } else if lane == 1 {
            commands.spawn((
                Transform::from_translation(Vec2::new(0., position_y).extend(0.)),
                BGMEvent {
                    time: note_time,
                    wav_file: wav_obj.wav_id,
                },
            ));
        } else {
            info!("not found: {}", lane);
        }
    }
}

fn notes_fall(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Note)>,
    mut status: ResMut<PlayStatus>,
) {
    let current_time = time.elapsed_secs();
    let elapsed = current_time - status.start_time;

    for (mut transform, note) in query.iter_mut() {
        let new_pos = JUDGEMENTLINE_POSITION.y + (note.time - elapsed) * status.speed;
        transform.translation.y = new_pos;
    }
}

fn play_bgm(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BGMEvent)>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
    status: ResMut<PlayStatus>,
) {
    let current_time = time.elapsed_secs();
    let elapsed = current_time - status.start_time;

    for (entity, bgm_event) in query.iter_mut() {
        if bgm_event.time <= elapsed {
            commands.entity(entity).despawn();
            if let Some(handle) = audio_assets.map.get(&bgm_event.wav_file) {
                audio.play(handle.clone());
            }
        }
    }
}

fn handel_bpm(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BPMEvent)>,
    mut status: ResMut<PlayStatus>,
) {
    let current_time = time.elapsed_secs();
    let elapsed = current_time - status.start_time;

    for (entity, bpm_event) in query.iter_mut() {
        if bpm_event.time <= elapsed {
            commands.entity(entity).despawn();
        } else {
            break;
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Playing,
}

const KEY_LANE_MAP: &[(KeyCode, Lane)] = &[
    (KeyCode::KeyA, Lane::LS),
    (KeyCode::KeyS, Lane::L1),
    (KeyCode::KeyD, Lane::L2),
    (KeyCode::KeyF, Lane::L3),
    (KeyCode::Space, Lane::L4),
    (KeyCode::KeyJ, Lane::L5),
    (KeyCode::KeyK, Lane::L6),
    (KeyCode::KeyL, Lane::L7),
];

fn keyboard_input(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<AppState>>,
    mut status: ResMut<PlayStatus>,
    lanes: Query<(&Lanes, &Children)>,
    notes: Query<&Note>,
) {
    // Start
    if keys.just_pressed(KeyCode::KeyY) {
        next_state.set(AppState::Playing);
        status.start_time = time.elapsed_secs();
        return;
    }

    let elapsed = time.elapsed_secs() - status.start_time;

    // 遍历按键映射
    for (key, target_lane) in KEY_LANE_MAP {
        if keys.just_pressed(*key) {
            let (_, children) = lanes
                .iter()
                .find(|(lane, _)| lane.0 == *target_lane)
                .unwrap();

            let closest = children
                .iter()
                .filter_map(|child| notes.get(child).ok().map(|note| (child, note)))
                .min_by(|(_, a_note), (_, b_note)| {
                    let da = (a_note.time - elapsed).abs();
                    let db = (b_note.time - elapsed).abs();
                    da.total_cmp(&db)
                });

            if let Some((entity, note)) = closest {
                if note.time - TIMING_WINDOW.good <= elapsed
                    && elapsed <= note.time + TIMING_WINDOW.good
                {
                    if let Some(handle) = audio_assets.map.get(&note.wav_file) {
                        audio.play(handle.clone());
                    }
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

// fn update_keysound(
//     mut key_sound: ResMut<KeySound>,
//     time: Res<Time>,
//     mut query: Query<(Entity, &mut BGMEvent)>,
// ) {
//     let current_time = time.elapsed_secs();
//     let elapsed = current_time - status.start_time;
// }
