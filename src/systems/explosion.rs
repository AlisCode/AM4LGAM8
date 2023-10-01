use bevy::{
    prelude::{Component, Query, Res},
    sprite::TextureAtlasSprite,
    time::{Time, Timer},
};

#[derive(Debug, Component)]
pub struct Explosion(pub Timer);

#[derive(Debug, Component)]
pub struct ExplosionAnimation(pub Timer);

pub fn animate_explosion(
    mut query: Query<(&mut ExplosionAnimation, &mut TextureAtlasSprite)>,
    time: Res<Time>,
) {
    let delta = time.delta();
    for (mut anim, mut sprite) in query.iter_mut() {
        anim.0.tick(delta);

        if anim.0.finished() {
            sprite.index = (sprite.index + 1).min(5);
            anim.0.reset();
        }
    }
}
