use std::path::PathBuf;

use bevy::{prelude::*, sprite::Anchor};
use bms_rs::bms::{BmsOutput, model::Header, parse_bms, prelude::KeyLayoutBeat};
use encoding_rs::SHIFT_JIS;
use num_traits::ToPrimitive;
use walkdir::WalkDir;

use crate::{
    resources::{BmsEntry, BmsLib},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Select), spawn_select)
        .add_systems(Update, keyboard_input.run_if(in_state(Screen::Select)))
        .add_systems(OnExit(Screen::Select), cleanup_select_screen)
        .insert_resource(BmsLib {
            cursor: 0,
            bms_arr: vec![],
        });
}

#[derive(Component)]
struct SelectItem;

#[derive(Component)]
struct OnSelectScreen;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Genre;

#[derive(Component)]
struct Title;

#[derive(Component)]
struct Artist;

#[derive(Component)]
struct BPM;

#[derive(Component)]
struct PlayLevel;

#[derive(Component)]
struct Rank;

const LINE_HEIGHT: f32 = 50.;
const LINE_WIDTH: f32 = 800.;
const RIGHT_OFFSET: f32 = 1080. - LINE_WIDTH / 2.;
const BORDER_THICKNESS: f32 = 2.;

const BMS_PATH: &str = "./bms";

fn spawn_select(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut data: ResMut<BmsLib>,
) {
    for entry in WalkDir::new(BMS_PATH) {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                continue;
            }
        };
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Some(ext) = path.extension().and_then(|x| x.to_str()) {
            if ext.eq_ignore_ascii_case("bms") || ext.eq_ignore_ascii_case("bme") {
                let bytes = std::fs::read(path).unwrap();

                let (bms_text, _encoding_used, _had_errors) = SHIFT_JIS.decode(&bytes);
                let bms_text = bms_text.into_owned();

                let BmsOutput { bms, .. }: BmsOutput<KeyLayoutBeat> = parse_bms(&bms_text);
                data.bms_arr.push(BmsEntry {
                    header: bms.header,
                    path: path.to_path_buf(),
                });
            }
        }
    }

    data.bms_arr
        .sort_by(|a, b| a.header.title.cmp(&b.header.title));

    let border_color = materials.add(Color::srgb(1., 1., 1.));
    let selected_border_color = materials.add(Color::srgb(1., 0., 0.));

    let text_font = TextFont {
        font_size: 50.0,
        ..default()
    };

    commands.spawn((
        Text2d::new(data.bms_arr[0].header.genre.clone().unwrap()),
        text_font.clone(),
        TextLayout::new_with_justify(Justify::Right),
        Transform::from_translation(Vec2::new(0., 100.).extend(0.)),
        Anchor::CENTER_RIGHT,
        OnSelectScreen,
        Genre,
    ));

    commands.spawn((
        Text2d::new(data.bms_arr[0].header.title.clone().unwrap()),
        text_font.clone(),
        TextLayout::new_with_justify(Justify::Right),
        Transform::from_translation(Vec2::new(0., 0.).extend(0.)),
        Anchor::CENTER_RIGHT,
        OnSelectScreen,
        Title,
    ));

    commands.spawn((
        Text2d::new(data.bms_arr[0].header.artist.clone().unwrap()),
        text_font.clone(),
        TextLayout::new_with_justify(Justify::Right),
        Transform::from_translation(Vec2::new(0., -100.).extend(0.)),
        Anchor::CENTER_RIGHT,
        OnSelectScreen,
        Artist,
    ));

    commands
        .spawn((
            OnSelectScreen,
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .with_children(|parent| {
            // 上
            parent.spawn((
                Mesh2d(meshes.add(Rectangle::new(LINE_WIDTH, BORDER_THICKNESS))),
                MeshMaterial2d(selected_border_color.clone()),
                Transform::from_translation(Vec2::new(RIGHT_OFFSET, LINE_HEIGHT / 2.).extend(1.)),
            ));

            // 下
            parent.spawn((
                Mesh2d(meshes.add(Rectangle::new(LINE_WIDTH, BORDER_THICKNESS))),
                MeshMaterial2d(selected_border_color.clone()),
                Transform::from_translation(Vec2::new(RIGHT_OFFSET, -LINE_HEIGHT / 2.).extend(1.)),
            ));

            // 左
            parent.spawn((
                Mesh2d(meshes.add(Rectangle::new(BORDER_THICKNESS, LINE_HEIGHT))),
                MeshMaterial2d(selected_border_color.clone()),
                Transform::from_translation(
                    Vec2::new(RIGHT_OFFSET - LINE_WIDTH / 2., 0.).extend(1.),
                ),
            ));
        });

    commands
        .spawn((
            OnSelectScreen,
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .with_children(|parent| {
            for (i, header) in data.bms_arr.iter().enumerate() {
                let title = header.header.title.clone().unwrap();
                let stack_y = -(i as f32) * (LINE_HEIGHT + BORDER_THICKNESS * 3.);

                parent
                    .spawn((
                        Text2d::new(title),
                        text_font.clone(),
                        TextLayout::new_with_justify(Justify::Left),
                        Transform::from_translation(
                            Vec2::new(RIGHT_OFFSET - LINE_WIDTH / 2., stack_y).extend(0.),
                        ),
                        Anchor::CENTER_LEFT,
                        SelectItem,
                    ))
                    .with_children(|parent| {
                        // 上
                        parent.spawn((
                            Mesh2d(meshes.add(Rectangle::new(LINE_WIDTH, BORDER_THICKNESS))),
                            MeshMaterial2d(border_color.clone()),
                            Transform::from_translation(
                                Vec2::new(LINE_WIDTH / 2. - BORDER_THICKNESS, LINE_HEIGHT / 2.)
                                    .extend(0.),
                            ),
                        ));

                        // 下
                        parent.spawn((
                            Mesh2d(meshes.add(Rectangle::new(LINE_WIDTH, BORDER_THICKNESS))),
                            MeshMaterial2d(border_color.clone()),
                            Transform::from_translation(
                                Vec2::new(LINE_WIDTH / 2. - BORDER_THICKNESS, -LINE_HEIGHT / 2.)
                                    .extend(0.),
                            ),
                        ));

                        // 左
                        parent.spawn((
                            Mesh2d(meshes.add(Rectangle::new(BORDER_THICKNESS, LINE_HEIGHT))),
                            MeshMaterial2d(border_color.clone()),
                            Transform::from_translation(
                                Vec2::new(-BORDER_THICKNESS, 0.).extend(0.),
                            ),
                        ));
                    });
            }
        });
}

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<SelectItem>>,
    mut data: ResMut<BmsLib>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut query_text: Query<
        (&mut Text2d, Option<&Genre>, Option<&Title>, Option<&Artist>),
        With<OnSelectScreen>,
    >,
) {
    if keys.just_pressed(KeyCode::ArrowDown) {
        let offset = LINE_HEIGHT + BORDER_THICKNESS * 3.;

        for mut tf in &mut query {
            tf.translation.y += offset;
        }

        if data.cursor == data.bms_arr.len().to_u32().unwrap() - 1 {
            data.cursor = 0;
        } else {
            data.cursor += 1;
        }

        for (mut text2d, genre, title, artist) in query_text.iter_mut() {
            if genre.is_some() {
                text2d.0 = data.bms_arr[data.cursor as usize]
                    .header
                    .genre
                    .clone()
                    .unwrap();
            }
            if title.is_some() {
                text2d.0 = data.bms_arr[data.cursor as usize]
                    .header
                    .title
                    .clone()
                    .unwrap();
            }
            if artist.is_some() {
                text2d.0 = data.bms_arr[data.cursor as usize]
                    .header
                    .artist
                    .clone()
                    .unwrap();
            }
        }
    }

    if keys.just_pressed(KeyCode::ArrowUp) {
        let offset = -(50. + 2. * 3.);

        for mut tf in &mut query {
            tf.translation.y += offset;
        }

        if data.cursor == 0 {
            data.cursor = data.bms_arr.len().to_u32().unwrap() - 1;
        } else {
            data.cursor -= 1;
        }

        for (mut text2d, genre, title, artist) in query_text.iter_mut() {
            if genre.is_some() {
                text2d.0 = data.bms_arr[data.cursor as usize]
                    .header
                    .genre
                    .clone()
                    .unwrap();
            }
            if title.is_some() {
                text2d.0 = data.bms_arr[data.cursor as usize]
                    .header
                    .title
                    .clone()
                    .unwrap();
            }
            if artist.is_some() {
                text2d.0 = data.bms_arr[data.cursor as usize]
                    .header
                    .artist
                    .clone()
                    .unwrap();
            }
        }
    }

    if keys.just_pressed(KeyCode::Enter) {
        next_screen.set(Screen::Gameplay)
    }
}

fn cleanup_select_screen(mut commands: Commands, query: Query<Entity, With<OnSelectScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
