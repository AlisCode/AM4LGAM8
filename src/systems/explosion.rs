use bevy::{
    ecs::system::Despawn,
    prelude::{Commands, Component, Entity, Query, Res},
    sprite::TextureAtlasSprite,
    time::{Time, Timer},
};

#[derive(Debug, Component)]
pub struct Explosion(pub Timer);

#[derive(Debug, Component)]
pub struct ExplosionAnimation(pub Timer);

pub fn animate_explosion(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Explosion,
        &mut ExplosionAnimation,
        &mut TextureAtlasSprite,
    )>,
    time: Res<Time>,
) {
    let delta = time.delta();
    for (entity, mut explosion, mut anim, mut sprite) in query.iter_mut() {
        explosion.0.tick(delta);
        anim.0.tick(delta);

        if anim.0.finished() {
            sprite.index = (sprite.index + 1).min(5);
            anim.0.reset();
        }

        if explosion.0.finished() {
            commands.add(Despawn { entity });
        }
    }
}
