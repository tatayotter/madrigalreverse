//! Walking to a point using only position reads.
//!
//! We don't know in advance which way the turn keys rotate the character relative to
//! the game's x/z axes, or how fast. So the heading is measured from movement (where
//! we were before a short walk vs. after), and the turn speed and direction are
//! learned from how much that heading changed after each turn.

/// Heading of a movement on the ground, in degrees.
pub fn heading(dx: f32, dz: f32) -> f32 {
    dz.atan2(dx).to_degrees()
}

/// Smallest signed difference `target - current`, in (-180, 180].
pub fn angle_diff(target: f32, current: f32) -> f32 {
    let mut d = (target - current) % 360.0;
    if d > 180.0 {
        d -= 360.0;
    } else if d <= -180.0 {
        d += 360.0;
    }
    d
}

/// Close enough to the target direction to just walk.
pub const ON_COURSE_DEG: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnKey {
    Left,
    Right,
}

impl TurnKey {
    pub fn key(&self) -> &'static str {
        match self {
            TurnKey::Left => "A",
            TurnKey::Right => "D",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Navigator {
    /// Degrees of heading change per millisecond of holding a turn key
    pub turn_rate: f32,
    /// +1 if holding Right increases `heading()`, -1 if it decreases it
    pub right_sign: f32,
    /// Game units per millisecond of walking
    pub speed: f32,
    /// Last turn: (Right = +ms / Left = -ms, heading before the turn)
    last_turn: Option<(f32, f32)>,
    /// The turn rate has been measured at least once
    rate_known: bool,
}

impl Default for Navigator {
    fn default() -> Self {
        Self {
            // Rough guesses, corrected after the first turn and walk
            turn_rate: 0.15,
            right_sign: -1.0,
            speed: 0.006,
            last_turn: None,
            rate_known: false,
        }
    }
}

impl Navigator {
    /// After a walk, learn from how far we went and which way.
    pub fn learn_walk(&mut self, moved: f32, walk_ms: u64, heading_now: f32) {
        if walk_ms > 0 && moved > 0.5 {
            let speed = moved / walk_ms as f32;
            self.speed = (self.speed + speed) / 2.0;
        }
        if let Some((signed_ms, before)) = self.last_turn.take() {
            let delta = angle_diff(heading_now, before);
            if delta.abs() > 3.0 && signed_ms.abs() > 0.0 {
                let rate = (delta.abs() / signed_ms.abs()).clamp(0.01, 3.0);
                self.turn_rate = if self.rate_known { (self.turn_rate + rate) / 2.0 } else { rate };
                self.rate_known = true;
                self.right_sign = (delta * signed_ms).signum();
            }
        }
    }

    /// Which way and how long to turn to change the heading by `diff` degrees.
    /// `None` when already on course.
    pub fn turn_for(&mut self, diff: f32, heading_now: f32) -> Option<(TurnKey, u64)> {
        if diff.abs() < ON_COURSE_DEG {
            return None;
        }
        let key = if diff * self.right_sign > 0.0 { TurnKey::Right } else { TurnKey::Left };
        // Until the turn speed is measured, turn at most ~30 degrees (by the guess), so a
        // fast turn can't spin past a full circle and teach us the wrong direction.
        let max_deg = if self.rate_known { 180.0 } else { 30.0 };
        let ms = (diff.abs().min(max_deg) / self.turn_rate).clamp(40.0, 2500.0);
        let signed = if key == TurnKey::Right { ms } else { -ms };
        self.last_turn = Some((signed, heading_now));
        Some((key, ms as u64))
    }

    /// How long to walk to cover `distance`, in short enough steps to correct course.
    pub fn walk_ms(&self, distance: f32) -> u64 {
        (distance / self.speed.max(0.0005)).clamp(400.0, 3000.0) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_diff_wraps() {
        assert_eq!(angle_diff(10.0, 350.0), 20.0);
        assert_eq!(angle_diff(350.0, 10.0), -20.0);
        assert_eq!(angle_diff(-170.0, 170.0), 20.0);
        assert_eq!(angle_diff(90.0, 90.0), 0.0);
    }

    /// A fake character: unknown turn direction and speeds, walks where it faces.
    struct World {
        x: f32,
        z: f32,
        facing: f32,
        right_turns: f32, // degrees per ms when holding Right (sign unknown to the navigator)
        speed: f32,
    }

    impl World {
        fn turn(&mut self, key: TurnKey, ms: u64) {
            let d = self.right_turns * ms as f32;
            self.facing += if key == TurnKey::Right { d } else { -d };
        }
        fn walk(&mut self, ms: u64) {
            let r = self.facing.to_radians();
            self.x += r.cos() * self.speed * ms as f32;
            self.z += r.sin() * self.speed * ms as f32;
        }
    }

    /// Same loop the farming behavior runs: walk, measure heading, turn, repeat.
    fn steps_to_reach(mut world: World, home: (f32, f32), radius: f32) -> Option<usize> {
        let mut nav = Navigator::default();
        let (mut px, mut pz) = (world.x, world.z);
        let mut last_walk = nav.walk_ms(100.0).min(600);
        world.walk(last_walk);
        for step in 0..15 {
            let (x, z) = (world.x, world.z);
            let dist = ((home.0 - x).powi(2) + (home.1 - z).powi(2)).sqrt();
            if dist <= radius {
                return Some(step);
            }
            let moved = ((x - px).powi(2) + (z - pz).powi(2)).sqrt();
            let h = heading(x - px, z - pz);
            nav.learn_walk(moved, last_walk, h);
            let want = heading(home.0 - x, home.1 - z);
            if let Some((key, ms)) = nav.turn_for(angle_diff(want, h), h) {
                world.turn(key, ms);
            }
            px = x;
            pz = z;
            last_walk = nav.walk_ms(dist);
            world.walk(last_walk);
        }
        None
    }

    #[test]
    fn reaches_home_whichever_way_keys_turn() {
        for right_turns in [0.2, -0.2, 0.08, -0.4, 0.6, -0.05] {
            for facing in [0.0, 90.0, 200.0, 315.0] {
                let world = World { x: 100.0, z: -40.0, facing, right_turns, speed: 0.01 };
                let steps = steps_to_reach(world, (0.0, 0.0), 20.0);
                assert!(steps.is_some(), "never arrived: turn {right_turns}, facing {facing}");
                println!("turn {right_turns} facing {facing}: {:?} steps", steps);
                assert!(steps.unwrap() <= 8, "too slow: {:?} steps", steps);
            }
        }
    }
}
