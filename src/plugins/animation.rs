use std::{collections::HashMap, default, marker::PhantomData, time::Duration};

use bevy::prelude::*;

pub trait AnimationKey: Clone + Eq + std::hash::Hash + Send + Sync + 'static {}

#[derive(Bundle)]
pub struct AnimationBundle<K: AnimationKey> {
    pub current_animation: CurrentAnimation<K>,
    pub timer: AnimationTimer,
    pub animations: AnimationMap<K>,
    pub sprite: Sprite,
}

#[derive(Component)]
pub struct CurrentAnimation<K: AnimationKey> {
    pub key: K,
}

impl<K: AnimationKey> CurrentAnimation<K> {
    pub fn new(key: K) -> Self {
        Self { key }
    }
}

#[derive(Component, Clone, Default)]
pub struct AnimationTimer(Timer);

#[derive(Component, Clone)]
pub struct AnimationMap<K: AnimationKey> {
    pub animations: HashMap<K, Animation>,
    pub frames: Vec<AnimationFrame>,
}

#[derive(Component, Clone, Debug)]
pub struct AnimationFrame {
    pub index: usize,
    pub duration: Duration,
}

impl AnimationFrame {
    pub fn new(index: usize, duration: Duration) -> Self {
        Self { index, duration }
    }
}

#[derive(Component, Clone, Debug)]
pub enum OnAnimationEndAction {
    Loop,
    Stop,
}

#[derive(Component, Clone)]
pub struct Animation {
    pub first_index: usize,
    pub last_index: usize,
    pub on_end: OnAnimationEndAction,
}

impl<K: AnimationKey> AnimationBundle<K> {
    fn new(
        &self,
        sprite_sheet: Handle<Image>,
        atlas: Handle<TextureAtlasLayout>,
        animations: HashMap<K, Animation>,
        frames: Vec<AnimationFrame>,
        default_animation: CurrentAnimation<K>,
    ) -> Self {
        let start_frame = 0;
        let timer = Timer::from_seconds(0.1, TimerMode::Once);
        let sprite = Sprite::from_atlas_image(
            sprite_sheet,
            TextureAtlas {
                layout: atlas,
                index: start_frame,
            },
        );

        AnimationBundle {
            current_animation: default_animation,
            timer: AnimationTimer(timer),
            animations: AnimationMap { animations, frames },
            sprite,
        }
    }
}

pub fn update_animations<K: AnimationKey>(
    mut query: Query<(
        &CurrentAnimation<K>,
        &mut Sprite,
        &mut AnimationTimer,
        &AnimationMap<K>,
    )>,
    time: Res<Time>,
) {
    for (current_animation, mut sprite, mut timer, animation_map) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            // Get the current animation from the map using the key
            let animation = animation_map
                .animations
                .get(&current_animation.key)
                .expect("Current animation key should always exist in map");

            let next_frame = if let Some(atlas) = &mut sprite.texture_atlas {
                let next_frame_index = atlas.index + 1;
                if next_frame_index > animation.last_index {
                    match animation.on_end {
                        OnAnimationEndAction::Loop => {
                            animation_map.frames.get(animation.first_index)
                        }
                        OnAnimationEndAction::Stop => {
                            animation_map.frames.get(animation.last_index)
                        }
                    }
                } else {
                    animation_map.frames.get(next_frame_index)
                }
            } else {
                panic!("Texture atlas not found")
            };

            sprite.texture_atlas.as_mut().unwrap().index = next_frame.unwrap().index;
            timer.0.reset();
            timer.0.set_duration(next_frame.unwrap().duration);
        }
    }
}

pub struct AnimationPlugin<K: AnimationKey> {
    _phantom: PhantomData<K>,
}

impl<K: AnimationKey> Default for AnimationPlugin<K> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<K: AnimationKey> Plugin for AnimationPlugin<K> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_animations::<K>);
    }
}
