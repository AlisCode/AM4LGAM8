use bevy::prelude::{Component, Entity, Event, EventWriter, Query};
use bevy_mod_picking::prelude::{DragEnd, Listener, On, Pointer};

use super::grid::GridCoordinates;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Event)]
pub struct RequestMoveEvent {
    /// Entity that triggered the event
    pub source: Entity,
    pub move_direction: MoveDirection,
    pub source_coords: GridCoordinates,
}

pub fn on_pointer_drag_end_handler() -> impl Component {
    let on_pointer_drag_end = On::<Pointer<DragEnd>>::run(handle_pointer_drag_end);
    on_pointer_drag_end
}

fn handle_pointer_drag_end(
    pointer_event: Listener<Pointer<DragEnd>>,
    query: Query<(Entity, &GridCoordinates)>,
    mut move_event_tx: EventWriter<RequestMoveEvent>,
) {
    let move_direction = {
        let dx = pointer_event.distance.x;
        let dy = pointer_event.distance.y;
        if dx.abs() > dy.abs() {
            // Move is horizontal
            if dx > 0. {
                MoveDirection::Right
            } else {
                MoveDirection::Left
            }
        } else {
            // Move is vertical
            if dy > 0. {
                MoveDirection::Down
            } else {
                MoveDirection::Up
            }
        }
    };

    let source_coords = query
        .iter()
        .find_map(|(entity, grid_coordinates)| {
            (entity == pointer_event.target).then(|| grid_coordinates.clone())
        })
        .expect("Failed to find event source");

    move_event_tx.send(RequestMoveEvent {
        source: pointer_event.target,
        move_direction,
        source_coords,
    });
}
