use bevy::{
    prelude::{
        BuildChildren, ButtonBundle, Commands, Component, DespawnRecursive, Entity, NextState,
        NodeBundle, Query, Res, ResMut, TextBundle,
    },
    text::TextStyle,
    ui::{AlignItems, FlexDirection, Interaction, JustifyContent, PositionType, Style, Val},
};

use crate::{
    assets::GameAssets,
    constants::{background_color, foreground_color},
    core::GameState,
};

use super::ui::GameScore;

#[derive(Component)]
pub struct OnGameOverScreen;

#[derive(Component)]
pub struct ToTitleScreenButton;

pub fn setup(mut commands: Commands, score: Res<GameScore>, assets: Res<GameAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                ..Default::default()
            },
            OnGameOverScreen,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Game over!",
                TextStyle {
                    font_size: 40.0,
                    color: foreground_color(),
                    font: assets.joystix.clone(),
                    ..Default::default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                format!("Your score: {}", score.get()),
                TextStyle {
                    font_size: 40.0,
                    color: foreground_color(),
                    font: assets.joystix.clone(),
                    ..Default::default()
                },
            ));
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
                    ToTitleScreenButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Exit",
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
    query: Query<(&Interaction, &ToTitleScreenButton)>,
    entities_on_title_screen: Query<(Entity, &OnGameOverScreen)>,
    mut state: ResMut<NextState<GameState>>,
) {
    for (interaction, _play_btn) in query.iter() {
        match interaction {
            Interaction::Pressed => {
                for (entity, _) in entities_on_title_screen.iter() {
                    commands.add(DespawnRecursive { entity })
                }

                // Swith to title screen
                state.set(GameState::TitleScreen);
            }
            _ => (),
        }
    }
}
