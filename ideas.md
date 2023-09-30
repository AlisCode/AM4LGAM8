# 2048-style Puzzle Game

Grid size:
    * 5x5 -> faster games, more stressing
    * 7x7 -> middleground
    * 9x9 -> longer games, less stressing, more prone to randomness = frustration
Incoming tiles
    * Random coords ?
    * Through a specific RX point ?

Push tiles around Sokoban-style
Tile collide with another one if the tile gets moved into a tile that cannot be moved on that direction (blocked by a wall, or by other tiles themselves blocked by a wall)
Colliding tiles of the same type will trigger their effect

Tiles types:
* Crate/Coin, explodes = +1 score = most common
* Little bomb, explodes adjacent
* Big bomb, explodes in a circle around the collision point
* Block, solidifies into a wall (unmovable object)

End of the run :
* No move available
* Grid is filled

# Endless Runner or Puzzle Levels?

* Endless runner fits the theme better (limited space doesn't really apply to levels), but is tricky to get right
* Puzzle levels fits the theme less, is easier to level-design, more interesting, but probably prone to scope-creeping = I don't have time for that
