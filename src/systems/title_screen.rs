use bevy::{
    prelude::{
        BuildChildren, ButtonBundle, Commands, Component, DespawnRecursive, Entity, NextState,
        NodeBundle, Query, Rect, Res, ResMut, TextBundle, Transform, Vec3,
    },
    sprite::{Sprite, SpriteBundle},
    text::TextStyle,
    ui::{AlignItems, Interaction, JustifyContent, PositionType, Style, Val},
};

use crate::{
    assets::GameAssets,
    constants::{
        background_color, foreground_color, GAME_LOGIC_HEIGHT, GAME_LOGIC_WIDTH, GRID_SIZE,
        TILE_SIZE,
    },
    core::GameState,
};

#[derive(Component)]
pub struct OnTitleScreen;

#[derive(Component)]
pub struct PlayButton;

pub fn setup(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                rect: Some(Rect::new(0., 0., GAME_LOGIC_WIDTH, GAME_LOGIC_HEIGHT)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(
                    TILE_SIZE * (GRID_SIZE as f32 / 2. - 0.5),
                    TILE_SIZE * (GRID_SIZE as f32 / 2. - 0.5) - 3.,
                    1.,
                ),
                ..Default::default()
            },
            texture: assets.title_screen.clone(),
            ..Default::default()
        },
        OnTitleScreen,
    ));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
            OnTitleScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            position_type: PositionType::Relative,
                            top: Val::Px(90.),
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        background_color: bevy::ui::BackgroundColor(foreground_color()),
                        ..Default::default()
                    },
                    PlayButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font_size: 40.0,
                            color: background_color(),
                            font: assets.joystix.clone(),
                            ..Default::default()
                        },
                    ));
                });
        });
}

pub fn update_ui(
    mut commands: Commands,
    query: Query<(&Interaction, &PlayButton)>,
    entities_on_title_screen: Query<(Entity, &OnTitleScreen)>,
    mut state: ResMut<NextState<GameState>>,
) {
    for (interaction, _play_btn) in query.iter() {
        match interaction {
            Interaction::Pressed => {
                for (entity, _) in entities_on_title_screen.iter() {
                    commands.add(DespawnRecursive { entity })
                }

                // Swith to play state
                state.set(GameState::Playing);
            }
            _ => (),
        }
    }
}
