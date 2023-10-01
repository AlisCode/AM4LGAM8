use bevy::{
    prelude::{
        Commands, Component, Query, Rect, Res, ResMut, Resource, TextBundle, Transform, Vec3,
    },
    sprite::{Sprite, SpriteBundle},
    text::{Text, TextStyle},
    ui::{Style, Val},
};

use crate::{
    assets::GameAssets,
    constants::{foreground_color, GAME_LOGIC_HEIGHT, GAME_LOGIC_WIDTH, TILE_SIZE},
};

// Components

#[derive(Component)]
pub struct ScoreLabel;

// Resource

#[derive(Debug, Default, Resource)]
pub struct GameScore(i32);

impl GameScore {
    pub fn add(&mut self, points: i32) {
        self.0 += points;
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }

    pub fn get(&self) -> i32 {
        self.0
    }
}

// Systems

pub fn reset_score(mut game_score: ResMut<GameScore>) {
    game_score.reset();
}

pub fn spawn_ui(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            rect: Some(Rect::new(0., 0., GAME_LOGIC_WIDTH, GAME_LOGIC_HEIGHT)),
            ..Default::default()
        },
        texture: assets.ui.clone(),
        transform: Transform {
            translation: Vec3::new(TILE_SIZE * 1.5, TILE_SIZE * 1.5 - 3., -1.),
            ..Default::default()
        },
        ..Default::default()
    });

    commands.spawn((
        TextBundle::from_section(
            "0",
            TextStyle {
                font_size: 45.,
                color: foreground_color(),
                ..Default::default()
            },
        )
        .with_style(Style {
            position_type: bevy::ui::PositionType::Absolute,
            bottom: Val::Px(-5.),
            left: Val::Px(120.),
            ..Default::default()
        }),
        ScoreLabel,
    ));
}

pub fn update_ui(game_score: Res<GameScore>, mut query: Query<(&mut Text, &ScoreLabel)>) {
    for (mut text, _label) in query.iter_mut() {
        text.sections[0].value = game_score.get().to_string();
    }
}
