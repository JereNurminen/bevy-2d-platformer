use std::collections::HashSet;

use crate::bundles::level::TileCoords;

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

impl Rectangle {
    pub fn new(x: i64, y: i64, width: i64, height: i64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn area(&self) -> i64 {
        self.width * self.height
    }

    pub fn contains_tile(&self, pos: &TileCoords) -> bool {
        pos.x >= self.x
            && pos.x < self.x + self.width
            && pos.y >= self.y
            && pos.y < self.y + self.height
    }
}

pub struct TileMerger {
    tile_size: f32,
}

impl TileMerger {
    pub fn new(tile_size: f32) -> Self {
        Self { tile_size }
    }

    /// Main algorithm: converts a set of tile positions into optimized rectangles
    pub fn merge_tiles(&self, tiles: &HashSet<TileCoords>) -> Vec<Rectangle> {
        if tiles.is_empty() {
            return Vec::new();
        }

        let mut remaining_tiles = tiles.clone();
        let mut rectangles = Vec::new();

        while !remaining_tiles.is_empty() {
            // Find the best rectangle starting from any remaining tile
            let best_rect = self.find_best_rectangle(&remaining_tiles);

            // Remove all tiles covered by this rectangle
            self.remove_covered_tiles(&mut remaining_tiles, &best_rect);

            rectangles.push(best_rect);
        }

        rectangles
    }

    /// Finds the best (largest area) rectangle that can be formed from remaining tiles
    fn find_best_rectangle(&self, tiles: &HashSet<TileCoords>) -> Rectangle {
        let mut best_rect = Rectangle::new(0, 0, 1, 1);
        let mut best_area = 0;

        // Try starting from each tile position
        for &start_pos in tiles {
            let rect = self.find_largest_rect_from_position(tiles, start_pos);
            if rect.area() > best_area {
                best_area = rect.area();
                best_rect = rect;
            }
        }

        best_rect
    }

    /// Find the largest rectangle that can be formed starting from a specific position
    fn find_largest_rect_from_position(
        &self,
        tiles: &HashSet<TileCoords>,
        start: TileCoords,
    ) -> Rectangle {
        let mut best_rect = Rectangle::new(start.x, start.y, 1, 1);
        let mut best_area = 1;

        // Find the maximum width we can extend to the right
        let max_width = self.find_max_width(tiles, start);

        // For each possible width, find the maximum height
        for width in 1..=max_width {
            let height = self.find_max_height_for_width(tiles, start, width);
            let area = width * height;

            if area > best_area {
                best_area = area;
                best_rect = Rectangle::new(start.x, start.y, width, height);
            }
        }

        best_rect
    }

    /// Find maximum width we can extend from start position
    fn find_max_width(&self, tiles: &HashSet<TileCoords>, start: TileCoords) -> i64 {
        let mut width = 0;

        loop {
            let test_pos = TileCoords {
                x: start.x + width,
                y: start.y,
            };
            if tiles.contains(&test_pos) {
                width += 1;
            } else {
                break;
            }
        }

        width
    }

    /// Find maximum height for a given width starting from position
    fn find_max_height_for_width(
        &self,
        tiles: &HashSet<TileCoords>,
        start: TileCoords,
        width: i64,
    ) -> i64 {
        let mut height = 0;

        'height_loop: loop {
            // Check if we can add another row
            for x_offset in 0..width {
                let test_pos = TileCoords {
                    x: start.x + x_offset,
                    y: start.y + height,
                };
                if !tiles.contains(&test_pos) {
                    break 'height_loop;
                }
            }
            height += 1;
        }

        height
    }

    /// Remove all tiles that are covered by the given rectangle
    fn remove_covered_tiles(&self, tiles: &mut HashSet<TileCoords>, rect: &Rectangle) {
        let mut to_remove = Vec::new();

        for &pos in tiles.iter() {
            if rect.contains_tile(&pos) {
                to_remove.push(pos);
            }
        }

        for pos in to_remove {
            tiles.remove(&pos);
        }
    }

    /// Convert rectangles to world coordinates for Bevy/Avian
    pub fn rectangles_to_world_coords(
        &self,
        rectangles: &[Rectangle],
    ) -> Vec<(f32, f32, f32, f32)> {
        rectangles
            .iter()
            .map(|rect| {
                let world_x =
                    rect.x as f32 * self.tile_size + (rect.width as f32 * self.tile_size) / 2.0;
                let world_y =
                    rect.y as f32 * self.tile_size + (rect.height as f32 * self.tile_size) / 2.0;
                let world_width = rect.width as f32 * self.tile_size;
                let world_height = rect.height as f32 * self.tile_size;

                (world_x, world_y, world_width, world_height)
            })
            .collect()
    }
}

// Example usage and integration with Bevy/Avian
#[cfg(feature = "bevy")]
mod bevy_integration {
    use super::*;
    use avian2d::prelude::*;
    use bevy::prelude::*;

    pub fn create_physics_bodies_from_tiles(
        commands: &mut Commands,
        tiles: &HashSet<TileCoords>,
        tile_size: f32,
    ) {
        let merger = TileMerger::new(tile_size);
        let rectangles = merger.merge_tiles(tiles);
        let world_coords = merger.rectangles_to_world_coords(&rectangles);

        println!(
            "Optimized {} tiles into {} physics bodies",
            tiles.len(),
            rectangles.len()
        );

        for (world_x, world_y, world_width, world_height) in world_coords {
            commands.spawn((
                RigidBody::Static,
                Collider::rectangle(world_width, world_height),
                Transform::from_translation(Vec3::new(world_x, world_y, 0.0)),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rectangle() {
        let merger = TileMerger::new(32.0);
        let mut tiles = HashSet::new();

        // Create a 3x2 rectangle
        for x in 0..3 {
            for y in 0..2 {
                tiles.insert(TileCoords { x, y });
            }
        }

        let rectangles = merger.merge_tiles(&tiles);
        assert_eq!(rectangles.len(), 1);
        assert_eq!(rectangles[0].width, 3);
        assert_eq!(rectangles[0].height, 2);
    }

    #[test]
    fn test_l_shape() {
        let merger = TileMerger::new(32.0);
        let mut tiles = HashSet::new();

        // Create an L-shape
        tiles.insert(TileCoords { x: 0, y: 0 });
        tiles.insert(TileCoords { x: 1, y: 0 });
        tiles.insert(TileCoords { x: 2, y: 0 });
        tiles.insert(TileCoords { x: 0, y: 1 });
        tiles.insert(TileCoords { x: 0, y: 2 });

        let rectangles = merger.merge_tiles(&tiles);

        // Should create 2 rectangles (one 3x1 horizontal, one 2x1 vertical)
        assert_eq!(rectangles.len(), 2);

        let total_area: i64 = rectangles.iter().map(|r| r.area()).sum();
        assert_eq!(total_area, 5); // Should cover all 5 tiles
    }
}
