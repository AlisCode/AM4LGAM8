use bevy::{
    ecs::system::Despawn,
    prelude::{Commands, Component, Entity, Query, Res},
    time::{Time, Timer},
};

#[derive(Component)]
pub struct MarkedForDeletion(pub Timer);

pub fn tick_marked_for_deletion(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut MarkedForDeletion)>,
) {
    let delta = time.delta();
    for (entity, mut marker) in query.iter_mut() {
        marker.0.tick(delta);
        if marker.0.finished() {
            commands.add(Despawn { entity });
        }
    }
}
