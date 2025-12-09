use crate::constants::*;
use crate::platform::Platform;

pub struct VersusLayout {
    pub spawn_p1: (f32, f32),
    pub spawn_p2: (f32, f32),
    pub platforms: Vec<Platform>,
}

impl VersusLayout {
    pub fn balanced_default() -> Self {
        // Expand arena width for more horizontal play
        let screen_w = (SCREEN_WIDTH as f32) * 1.5;
        let ground = Platform::new(-100.0, GROUND_Y, screen_w + 200.0, 60.0);

        // Lower layer platforms (safe approach)
        let lower_left = Platform::new(80.0, 460.0, 220.0, 24.0);
        let lower_right = Platform::new(screen_w - 300.0, 460.0, 220.0, 24.0);
        let lower_center = Platform::new(screen_w / 2.0 - 110.0, 430.0, 220.0, 22.0);

        // Mid layer, reachable with new jump, encourages crossings
        let mid_left = Platform::new(180.0, 340.0, 200.0, 22.0);
        let mid_right = Platform::new(screen_w - 380.0, 340.0, 200.0, 22.0);
        let mid_center_left = Platform::new(screen_w / 2.0 - 260.0, 310.0, 160.0, 20.0);
        let mid_center_right = Platform::new(screen_w / 2.0 + 100.0, 310.0, 160.0, 20.0);

        // High risk/reward platforms
        let high_left = Platform::new(160.0, 230.0, 140.0, 18.0);
        let high_right = Platform::new(screen_w - 300.0, 230.0, 140.0, 18.0);
        let high_center = Platform::new(screen_w / 2.0 - 90.0, 200.0, 180.0, 18.0);

        // Center clash area
        let center_pillar = Platform::new(screen_w / 2.0 - 50.0, 360.0, 100.0, 22.0);
        let center_bridge = Platform::new(screen_w / 2.0 - 160.0, 270.0, 320.0, 18.0);

        // Stepping stones / bumpers to reach highs from edges
        let step_left_low = Platform::new(40.0, 520.0, 70.0, 18.0);
        let step_right_low = Platform::new(screen_w - 110.0, 520.0, 70.0, 18.0);
        let step_left_mid = Platform::new(60.0, 490.0, 70.0, 18.0);
        let step_right_mid = Platform::new(screen_w - 130.0, 490.0, 70.0, 18.0);
        let step_left_high = Platform::new(90.0, 455.0, 80.0, 18.0);
        let step_right_high = Platform::new(screen_w - 170.0, 455.0, 80.0, 18.0);

        let platforms = vec![
            ground,
            lower_left,
            lower_right,
            lower_center,
            mid_left,
            mid_right,
            mid_center_left,
            mid_center_right,
            center_pillar,
            center_bridge,
            high_left,
            high_right,
            high_center,
            step_left_low,
            step_right_low,
            step_left_mid,
            step_right_mid,
            step_left_high,
            step_right_high,
        ];

        VersusLayout {
            spawn_p1: (screen_w / 2.0 - 140.0, GROUND_Y - PLAYER_HEIGHT),
            spawn_p2: (
                screen_w / 2.0 + 140.0 - PLAYER_WIDTH,
                GROUND_Y - PLAYER_HEIGHT,
            ),
            platforms,
        }
    }
}
