use std::{collections::HashMap, time::Duration};

use bevy::prelude::*;

use crate::aseprite_deserialize::{Aseprite, Slice};

use super::animation::{
    Animation, AnimationBundle, AnimationFrame, AnimationKey, AnimationMap, AnimationTimer,
    CurrentAnimation, NextAnimation, OnAnimationEndAction,
};

/// Complete animation metadata for a sprite
#[derive(Clone, Debug)]
pub struct AnimationData {
    /// All frames in the spritesheet
    pub frames: Vec<AnimationFrame>,
    /// Named animations (tags) mapped to their frame ranges
    pub animations: HashMap<String, AnimationTag>,
    /// Sprite sheet dimensions
    pub sheet_size: UVec2,
    /// Individual frame size
    pub frame_size: UVec2,
    /// Slices from Aseprite (e.g., hitboxes)
    pub slices: Vec<Slice>,
}

/// Metadata for a named animation (from Aseprite frame tags)
#[derive(Clone, Debug)]
pub struct AnimationTag {
    pub name: String,
    pub from: usize,
    pub to: usize,
    pub direction: String,
}

/// Configuration for an animation, allowing Rust code to override behavior
#[derive(Clone, Debug)]
pub struct AnimationConfig {
    /// The name of the Aseprite tag to use
    pub tag_name: &'static str,
    /// What to do when the animation ends
    pub on_end: OnAnimationEndAction,
    // Future extensibility:
    // pub speed_multiplier: f32,
    // pub can_be_interrupted: bool,
    // pub priority: u8,
    // pub events: Vec<(usize, AnimationEvent)>,
}

impl AnimationConfig {
    /// Create a looping animation configuration
    pub fn looping(tag_name: &'static str) -> Self {
        Self {
            tag_name,
            on_end: OnAnimationEndAction::Loop,
        }
    }

    /// Create a one-shot animation configuration
    pub fn once(tag_name: &'static str) -> Self {
        Self {
            tag_name,
            on_end: OnAnimationEndAction::Stop,
        }
    }
}

/// Resource that holds pre-loaded animation data for all entities
#[derive(Resource, Default)]
pub struct AnimationLibrary {
    pub player: Option<AnimationData>,
    // Add more entity types here as needed
    // pub enemy_bat: Option<AnimationData>,
    // pub boss: Option<AnimationData>,
}

impl AnimationLibrary {
    pub fn is_ready(&self) -> bool {
        self.player.is_some()
        // && self.enemy_bat.is_some() // etc.
    }

    /// Helper function to create an AnimationBundle from library data
    ///
    /// # Arguments
    /// * `anim_data` - The AnimationData from the library
    /// * `sprite_path` - Path to the sprite sheet image (e.g., "sprites/player.png")
    /// * `animation_configs` - HashMap mapping your custom animation keys to AnimationConfig
    /// * `default_animation` - The starting animation key
    /// * `asset_server` - Bevy AssetServer for loading the sprite
    /// * `texture_atlas_layouts` - Bevy resource for creating texture atlas layouts
    ///
    /// # Example
    /// ```rust
    /// let configs = HashMap::from([
    ///     (PlayerAnimations::Idle, AnimationConfig::looping("idle")),
    ///     (PlayerAnimations::Run, AnimationConfig::looping("run")),
    ///     (PlayerAnimations::Death, AnimationConfig::once("death")),
    /// ]);
    ///
    /// let bundle = AnimationLibrary::create_animation_bundle(
    ///     player_anim_data,
    ///     "sprites/player.png",
    ///     configs,
    ///     PlayerAnimations::Idle,
    ///     &asset_server,
    ///     &mut texture_atlas_layouts,
    /// );
    /// ```
    pub fn create_animation_bundle<K: AnimationKey>(
        anim_data: &AnimationData,
        sprite_path: &str,
        animation_configs: HashMap<K, AnimationConfig>,
        default_animation: K,
        asset_server: &AssetServer,
        texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> AnimationBundle<K> {
        let texture = asset_server.load(sprite_path);

        // Create texture atlas layout from animation data
        let frame_size = anim_data.frame_size;
        let columns = (anim_data.sheet_size.x / frame_size.x) as u32;
        let rows = (anim_data.sheet_size.y / frame_size.y) as u32;
        let layout = TextureAtlasLayout::from_grid(frame_size, columns, rows, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        // Map custom animation keys to Aseprite tag ranges with config overrides
        let animations: HashMap<K, Animation> = animation_configs
            .into_iter()
            .map(|(key, config)| {
                let tag = anim_data
                    .animations
                    .get(config.tag_name)
                    .unwrap_or_else(|| {
                        panic!(
                            "Animation tag '{}' not found in Aseprite data",
                            config.tag_name
                        )
                    });

                (
                    key,
                    Animation {
                        first_index: tag.from,
                        last_index: tag.to,
                        on_end: config.on_end,
                    },
                )
            })
            .collect();

        AnimationBundle {
            current_animation: CurrentAnimation::new(default_animation),
            next_animation: NextAnimation { key: None },
            timer: AnimationTimer::default(),
            animations: AnimationMap {
                animations,
                frames: anim_data.frames.clone(),
            },
            sprite: Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                },
            ),
        }
    }
}

/// Resource holding handles to Aseprite JSON files during loading
#[derive(Resource)]
pub struct AnimationDataHandles {
    pub player: Handle<Aseprite>,
    // Add more handles as needed
}

/// Converts Aseprite data into AnimationData
pub fn aseprite_to_animation_data(aseprite: &Aseprite) -> AnimationData {
    let frames: Vec<AnimationFrame> = aseprite
        .frames
        .iter()
        .enumerate()
        .map(|(index, frame)| {
            AnimationFrame::new(index, Duration::from_millis(frame.duration as u64))
        })
        .collect();

    let animations: HashMap<String, AnimationTag> = aseprite
        .meta
        .frame_tags
        .iter()
        .map(|tag| {
            (
                tag.name.clone(),
                AnimationTag {
                    name: tag.name.clone(),
                    from: tag.from,
                    to: tag.to,
                    direction: tag.direction.clone(),
                },
            )
        })
        .collect();

    // Extract frame size from first frame if available
    let frame_size = aseprite
        .frames
        .first()
        .map(|f| UVec2::new(f.frame.w as u32, f.frame.h as u32))
        .unwrap_or(UVec2::ZERO);

    AnimationData {
        frames,
        animations,
        sheet_size: UVec2::new(aseprite.meta.size.w as u32, aseprite.meta.size.h as u32),
        frame_size,
        slices: aseprite.meta.slices.clone(),
    }
}

/// Startup system to begin loading animation data
pub fn load_animation_data(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AnimationDataHandles {
        player: asset_server.load("sprites/player.json"),
        // Add more loads as needed
    });
    commands.insert_resource(AnimationLibrary::default());
}

/// System that checks if assets are loaded and builds the animation library
pub fn build_animation_library(
    mut library: ResMut<AnimationLibrary>,
    aseprite_assets: Res<Assets<Aseprite>>,
    handles: Res<AnimationDataHandles>,
) {
    // Only run if library isn't ready yet
    if library.is_ready() {
        return;
    }

    // Check if player animation data is loaded
    if library.player.is_none() {
        if let Some(player_data) = aseprite_assets.get(&handles.player) {
            let anim_data = aseprite_to_animation_data(player_data);
            info!(
                "Loaded player animations: {} frames, {} tags",
                anim_data.frames.len(),
                anim_data.animations.len()
            );
            for (name, tag) in &anim_data.animations {
                info!("  - '{}': frames {}-{}", name, tag.from, tag.to);
            }
            library.player = Some(anim_data);
        }
    }

    // Add more entity types here as they're loaded
    // if library.enemy_bat.is_none() { ... }

    if library.is_ready() {
        info!("Animation library fully loaded!");
    }
}

pub struct AnimationLibraryPlugin;

impl Plugin for AnimationLibraryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_animation_data)
            .add_systems(Update, build_animation_library);
    }
}
