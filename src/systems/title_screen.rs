use bevy::{
    prelude::{
        BuildChildren, ButtonBundle, Camera, Commands, Component, DespawnRecursive, Entity,
        NextState, NodeBundle, Query, Rect, Res, ResMut, Resource, TextBundle, Transform, Vec3,
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
pub struct PlayButton;

#[derive(Resource)]
pub struct TitleScreenEntities {
    pub title_screen_sprite: Entity,
    pub ui: Entity,
}

impl Default for TitleScreenEntities {
    fn default() -> Self {
        TitleScreenEntities {
            title_screen_sprite: Entity::PLACEHOLDER,
            ui: Entity::PLACEHOLDER,
        }
    }
}

pub fn setup(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut entities: ResMut<TitleScreenEntities>,
) {
    let title_screen_sprite = commands
        .spawn(SpriteBundle {
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
        })
        .id();

    let ui = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        })
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
                            ..Default::default()
                        },
                    ));
                });
        })
        .id();

    entities.title_screen_sprite = title_screen_sprite;
    entities.ui = ui;
}

pub fn update_ui(
    mut commands: Commands,
    query: Query<(&Interaction, &PlayButton)>,
    camera_entity: Query<(Entity, &Camera)>,
    mut state: ResMut<NextState<GameState>>,
    title_screen_entities: Res<TitleScreenEntities>,
) {
    for (interaction, _play_btn) in query.iter() {
        match interaction {
            Interaction::Pressed => {
                // Despawn everything
                let (entity, _) = camera_entity.single();
                commands.add(DespawnRecursive { entity });

                commands.add(DespawnRecursive {
                    entity: title_screen_entities.ui,
                });
                commands.add(DespawnRecursive {
                    entity: title_screen_entities.title_screen_sprite,
                });

                // Swith to play state
                state.set(GameState::Playing);
            }
            _ => (),
        }
    }
}
