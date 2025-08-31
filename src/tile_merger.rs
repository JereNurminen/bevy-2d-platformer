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

    /// Get all tile coordinates that this rectangle covers
    pub fn get_covered_tiles(&self) -> Vec<TileCoords> {
        let mut tiles = Vec::new();
        for x in self.x..(self.x + self.width) {
            for y in self.y..(self.y + self.height) {
                tiles.push(TileCoords { x, y });
            }
        }
        tiles
    }
}

pub struct TileMerger {
    tile_size: f32,
}

impl TileMerger {
    pub fn new(tile_size: f32) -> Self {
        Self { tile_size }
    }

    /// Main algorithm: converts a set of tile positions into optimized rectangles using greedy approach
    pub fn merge_tiles(&self, tiles: &HashSet<TileCoords>) -> Vec<Rectangle> {
        if tiles.is_empty() {
            return Vec::new();
        }

        let mut remaining_tiles = tiles.clone();
        let mut rectangles = Vec::new();

        while !remaining_tiles.is_empty() {
            // Find the best rectangle that can be formed from remaining tiles
            let best_rect = self.find_best_rectangle(&remaining_tiles);

            // Remove all tiles covered by this rectangle
            for tile in best_rect.get_covered_tiles() {
                remaining_tiles.remove(&tile);
            }

            rectangles.push(best_rect);
        }

        rectangles
    }

    /// Finds the rectangle with the largest area that can be formed from available tiles
    fn find_best_rectangle(&self, tiles: &HashSet<TileCoords>) -> Rectangle {
        let mut best_rect = None;
        let mut best_area = 0;

        // Try every tile as a potential top-left corner
        for &tile in tiles {
            let rect = self.find_largest_rect_from_position(tiles, tile);
            if rect.area() > best_area {
                best_area = rect.area();
                best_rect = Some(rect);
            }
        }

        best_rect.unwrap_or_else(|| {
            // Fallback: create a 1x1 rectangle from any remaining tile
            let &first_tile = tiles.iter().next().unwrap();
            Rectangle::new(first_tile.x, first_tile.y, 1, 1)
        })
    }

    /// Find the largest rectangle that can be formed starting from a specific position
    fn find_largest_rect_from_position(
        &self,
        tiles: &HashSet<TileCoords>,
        start: TileCoords,
    ) -> Rectangle {
        let mut best_rect = Rectangle::new(start.x, start.y, 1, 1);
        let mut best_area = 1;

        // Find maximum possible width from this starting position
        let max_width = self.find_max_width_from_position(tiles, start);

        // For each width from 1 to max_width, find the maximum height
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

    /// Find the maximum width we can extend horizontally from the starting position
    fn find_max_width_from_position(&self, tiles: &HashSet<TileCoords>, start: TileCoords) -> i64 {
        let mut width = 0;

        // Keep extending right as long as we find tiles
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

    /// For a given width, find the maximum height we can achieve
    fn find_max_height_for_width(
        &self,
        tiles: &HashSet<TileCoords>,
        start: TileCoords,
        width: i64,
    ) -> i64 {
        let mut height = 0;

        // Keep extending downward
        loop {
            let mut row_complete = true;

            // Check if the entire row at this height exists
            for x_offset in 0..width {
                let test_pos = TileCoords {
                    x: start.x + x_offset,
                    y: start.y + height,
                };

                if !tiles.contains(&test_pos) {
                    row_complete = false;
                    break;
                }
            }

            if row_complete {
                height += 1;
            } else {
                break;
            }
        }

        height
    }

    /// Convert rectangles to world coordinates for Bevy/Avian physics
    /// Returns (center_x, center_y, width, height) in world coordinates
    pub fn rectangles_to_world_coords(
        &self,
        rectangles: &[Rectangle],
    ) -> Vec<(f32, f32, f32, f32)> {
        rectangles
            .iter()
            .map(|rect| {
                // Calculate the center position of the rectangle
                let center_x = (rect.x as f32 + rect.width as f32 / 2.0) * self.tile_size;
                let center_y = (rect.y as f32 + rect.height as f32 / 2.0) * self.tile_size;

                // Calculate the total size
                let total_width = rect.width as f32 * self.tile_size;
                let total_height = rect.height as f32 * self.tile_size;

                (center_x, center_y, total_width, total_height)
            })
            .collect()
    }

    /// Helper method to create physics colliders from tile set
    pub fn create_collider_data(&self, tiles: &HashSet<TileCoords>) -> Vec<(f32, f32, f32, f32)> {
        let rectangles = self.merge_tiles(tiles);
        self.rectangles_to_world_coords(&rectangles)
    }
}

// Bevy integration helper
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
        let collider_data = merger.create_collider_data(tiles);

        println!(
            "Optimized {} tiles into {} physics bodies",
            tiles.len(),
            collider_data.len()
        );

        for (center_x, center_y, width, height) in collider_data {
            commands.spawn((
                RigidBody::Static,
                Collider::rectangle(width, height),
                Transform::from_translation(Vec3::new(center_x, center_y, 0.0)),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_tile() {
        let merger = TileMerger::new(32.0);
        let mut tiles = HashSet::new();
        tiles.insert(TileCoords { x: 0, y: 0 });

        let rectangles = merger.merge_tiles(&tiles);
        assert_eq!(rectangles.len(), 1);
        assert_eq!(rectangles[0].width, 1);
        assert_eq!(rectangles[0].height, 1);
    }

    #[test]
    fn test_horizontal_line() {
        let merger = TileMerger::new(32.0);
        let mut tiles = HashSet::new();

        // Create a horizontal line of 4 tiles
        for x in 0..4 {
            tiles.insert(TileCoords { x, y: 0 });
        }

        let rectangles = merger.merge_tiles(&tiles);
        assert_eq!(rectangles.len(), 1);
        assert_eq!(rectangles[0].width, 4);
        assert_eq!(rectangles[0].height, 1);
    }

    #[test]
    fn test_vertical_line() {
        let merger = TileMerger::new(32.0);
        let mut tiles = HashSet::new();

        // Create a vertical line of 3 tiles
        for y in 0..3 {
            tiles.insert(TileCoords { x: 0, y });
        }

        let rectangles = merger.merge_tiles(&tiles);
        assert_eq!(rectangles.len(), 1);
        assert_eq!(rectangles[0].width, 1);
        assert_eq!(rectangles[0].height, 3);
    }

    #[test]
    fn test_rectangle_formation() {
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
        assert_eq!(rectangles[0].area(), 6);
    }

    #[test]
    fn test_l_shape() {
        let merger = TileMerger::new(32.0);
        let mut tiles = HashSet::new();

        // Create an L-shape:
        // XXX
        // X
        // X
        tiles.insert(TileCoords { x: 0, y: 0 });
        tiles.insert(TileCoords { x: 1, y: 0 });
        tiles.insert(TileCoords { x: 2, y: 0 });
        tiles.insert(TileCoords { x: 0, y: 1 });
        tiles.insert(TileCoords { x: 0, y: 2 });

        let rectangles = merger.merge_tiles(&tiles);

        // Should create multiple rectangles
        assert!(rectangles.len() >= 2);

        // Total area should equal number of original tiles
        let total_area: i64 = rectangles.iter().map(|r| r.area()).sum();
        assert_eq!(total_area, 5);
    }

    #[test]
    fn test_scattered_tiles() {
        let merger = TileMerger::new(32.0);
        let mut tiles = HashSet::new();

        // Create scattered individual tiles
        tiles.insert(TileCoords { x: 0, y: 0 });
        tiles.insert(TileCoords { x: 2, y: 2 });
        tiles.insert(TileCoords { x: 5, y: 5 });

        let rectangles = merger.merge_tiles(&tiles);

        // Should create one rectangle per tile
        assert_eq!(rectangles.len(), 3);

        // Each should be 1x1
        for rect in rectangles {
            assert_eq!(rect.area(), 1);
        }
    }

    #[test]
    fn test_world_coordinates() {
        let merger = TileMerger::new(32.0);
        let mut tiles = HashSet::new();

        // Single tile at origin
        tiles.insert(TileCoords { x: 0, y: 0 });

        let rectangles = merger.merge_tiles(&tiles);
        let world_coords = merger.rectangles_to_world_coords(&rectangles);

        assert_eq!(world_coords.len(), 1);
        let (center_x, center_y, width, height) = world_coords[0];

        // Center should be at half tile size
        assert_eq!(center_x, 16.0);
        assert_eq!(center_y, 16.0);
        assert_eq!(width, 32.0);
        assert_eq!(height, 32.0);
    }

    #[test]
    fn test_efficiency_improvement() {
        let merger = TileMerger::new(32.0);
        let mut tiles = HashSet::new();

        // Create a large solid block (10x10 = 100 tiles)
        for x in 0..10 {
            for y in 0..10 {
                tiles.insert(TileCoords { x, y });
            }
        }

        let rectangles = merger.merge_tiles(&tiles);

        // Should merge 100 individual tiles into 1 large rectangle
        assert_eq!(rectangles.len(), 1);
        assert_eq!(rectangles[0].width, 10);
        assert_eq!(rectangles[0].height, 10);
        assert_eq!(rectangles[0].area(), 100);

        println!(
            "Efficiency test: {} tiles merged into {} colliders ({}% reduction)",
            tiles.len(),
            rectangles.len(),
            ((tiles.len() - rectangles.len()) as f32 / tiles.len() as f32 * 100.0) as i32
        );
    }

    #[test]
    fn test_complex_level_layout() {
        let merger = TileMerger::new(32.0);
        let mut tiles = HashSet::new();

        // Create a complex level layout with platforms and walls
        // Ground platform (20 tiles wide)
        for x in 0..20 {
            tiles.insert(TileCoords { x, y: 0 });
        }

        // Left wall (5 tiles high)
        for y in 1..6 {
            tiles.insert(TileCoords { x: 0, y });
        }

        // Right wall (5 tiles high)
        for y in 1..6 {
            tiles.insert(TileCoords { x: 19, y });
        }

        // Middle platform (8 tiles wide)
        for x in 6..14 {
            tiles.insert(TileCoords { x, y: 3 });
        }

        let rectangles = merger.merge_tiles(&tiles);

        // Should significantly reduce the number of physics bodies
        let original_count = tiles.len();
        let optimized_count = rectangles.len();

        println!(
            "Complex level test: {} tiles merged into {} colliders",
            original_count, optimized_count
        );

        // Verify all original tiles are covered
        let total_area: i64 = rectangles.iter().map(|r| r.area()).sum();
        assert_eq!(total_area, original_count as i64);

        // Should have significantly fewer colliders than original tiles
        assert!(optimized_count < original_count);
        assert!(optimized_count <= 4); // Should be very efficient for this layout
    }
}
