use bevy::prelude::{Bundle, Camera2d, Component};

#[derive(Component)]
pub struct MainCamera;

#[derive(Bundle)]
pub struct CameraBundle {
    pub camera: Camera2d,
    pub main_camera: MainCamera,
}

impl Default for CameraBundle {
    fn default() -> Self {
        Self {
            camera: Camera2d,
            main_camera: MainCamera,
        }
    }
}
