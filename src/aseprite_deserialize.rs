use bevy::asset::Handle;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Resource)]
pub struct AsepriteHandle(pub Handle<Aseprite>);

#[derive(serde::Deserialize, bevy::asset::Asset, bevy::reflect::TypePath)]
pub struct Aseprite {
    pub frames: Vec<Frame>,
    pub meta: Meta,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub filename: String,
    pub frame: Rect,
    pub rotated: bool,
    pub trimmed: bool,
    pub sprite_source_size: Rect,
    pub source_size: Size,
    pub duration: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Size {
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub app: String,
    pub version: String,
    pub image: String,
    pub format: String,
    pub size: Size,
    pub scale: String,
    pub frame_tags: Vec<FrameTag>,
    pub slices: Vec<Slice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameTag {
    pub name: String,
    pub from: usize, // inclusive index in frames
    pub to: usize,   // inclusive index in frames
    pub direction: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Slice {
    // Aseprite 'slices' can be nested structures; leave minimal fields for now.
    // Keep as generic so empty slices array deserializes fine.
    pub name: Option<String>,
    pub color: Option<String>,
    pub keys: Option<Vec<serde_json::Value>>,
}

impl Aseprite {
    /// Return a map from frame tag name (animation name) to the frames in that range.
    pub fn animation_frames(&self) -> HashMap<String, Vec<&Frame>> {
        let mut map = HashMap::new();
        for tag in &self.meta.frame_tags {
            // Aseprite 'from' and 'to' are inclusive indices into the frames array.
            let from = tag.from;
            let to = tag.to;
            let slice = self
                .frames
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx >= from && *idx <= to)
                .map(|(_, f)| f)
                .collect::<Vec<&Frame>>();
            map.insert(tag.name.clone(), slice);
        }
        map
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Save the JSON you pasted as `player_aseprite.json` in the project root
    let json = fs::read_to_string("player_aseprite.json")?;
    let data: Aseprite = serde_json::from_str(&json)?;

    println!("Image: {}", data.meta.image);
    println!("Total frames: {}", data.frames.len());
    println!(
        "Sprite sheet size: {}x{}",
        data.meta.size.w, data.meta.size.h
    );

    let animations = data.animation_frames();
    for (name, frames) in &animations {
        let durations: Vec<u32> = frames.iter().map(|f| f.duration).collect();
        println!(
            "Animation '{}' -> {} frames (durations: {:?})",
            name,
            frames.len(),
            durations
        );
        // Example: show frame rectangles
        for (i, f) in frames.iter().enumerate() {
            println!("  {}: {} -> rect: {:?}", i, f.filename, f.frame);
        }
    }

    // If you want to fetch a specific animation (e.g. "idle"):
    if let Some(idle_frames) = animations.get("idle") {
        println!("Idle animation has {} frames", idle_frames.len());
    }

    Ok(())
}
