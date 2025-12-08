use crate::constants::*;

pub struct CameraShake {
    pub timer: f32,
    pub intensity: f32,
}

impl CameraShake {
    pub fn new() -> Self {
        Self {
            timer: 0.0,
            intensity: 0.0,
        }
    }

    pub fn trigger(&mut self, intensity: f32, duration: f32) {
        self.intensity = intensity;
        self.timer = duration;
    }

    pub fn trigger_footstep(&mut self) {
        self.trigger(CAMERA_SHAKE_FOOTSTEP, CAMERA_SHAKE_DURATION * 0.3);
    }

    pub fn trigger_jump(&mut self) {
        self.trigger(CAMERA_SHAKE_JUMP, CAMERA_SHAKE_DURATION * 0.5);
    }

    pub fn trigger_kill(&mut self) {
        self.trigger(CAMERA_SHAKE_KILL, CAMERA_SHAKE_DURATION);
    }

    pub fn update(&mut self, dt: f32) {
        if self.timer > 0.0 {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.intensity = 0.0;
            }
        }
    }

    pub fn get_offset(&self) -> f32 {
        if self.timer > 0.0 {
            use macroquad::prelude::*;
            let random = ((get_time() * 1000.0) as u32 % 100) as f32 / 100.0;
            (random - 0.5) * self.intensity
        } else {
            0.0
        }
    }
}

