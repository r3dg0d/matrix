//! Per-column rain simulation.

use crate::charset::Charset;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Cell {
    pub ch: char,
    /// Age in [0, 1] where 0 is freshest (near head).
    pub age: f32,
    pub is_lead: bool,
}

#[derive(Debug)]
pub struct Column {
    pub x: u16,
    /// Head y position (float for sub-cell motion).
    pub head: f32,
    pub speed: f32,
    /// Speed this column was given at creation or the last retune.
    /// Respawn jitter is applied against this so it cannot random-walk.
    base_speed: f32,
    pub trail_len: usize,
    pub cells: Vec<Option<Cell>>,
    /// Persistent glyphs so mid-trail characters flicker instead of constantly changing.
    glyphs: Vec<char>,
    pub height: usize,
    /// Frames until this column respawns after going off-screen.
    pub delay: u32,
}

/// Trail length in `1..=height`. A hard floor of 4 panics: `f32::clamp` requires
/// min <= max, and a terminal shorter than 4 rows makes the ceiling smaller.
fn scaled_trail_len(height: usize, trail_factor: f32, rng: &mut impl Rng) -> usize {
    let h = height.max(1) as f32;
    let floor = 4.0_f32.min(h);
    let raw = (h * trail_factor * rng.gen_range(0.55..1.15)).round();
    raw.clamp(floor, h) as usize
}

/// Head position above the top row. `1.0..(height * 0.5)` is empty when the
/// terminal is 1 or 2 rows, and `gen_range` panics on an empty range.
fn offscreen_head(height: usize, rng: &mut impl Rng) -> f32 {
    let span = (height.max(1) as f32 * 0.5).max(1.0);
    -rng.gen_range(0.0..=span)
}

impl Column {
    pub fn new(
        x: u16,
        height: usize,
        speed_base: f32,
        trail_factor: f32,
        rng: &mut impl Rng,
    ) -> Self {
        let h = height.max(1);
        let trail_len = scaled_trail_len(h, trail_factor, rng);
        let speed = speed_base * rng.gen_range(0.45..1.55);
        Self {
            x,
            head: -(rng.gen_range(0.0..(h as f32 * 0.8))),
            speed,
            base_speed: speed,
            trail_len,
            cells: vec![None; h],
            glyphs: vec![' '; h],
            height: h,
            delay: rng.gen_range(0..40),
        }
    }

    pub fn resize(&mut self, height: usize, trail_factor: f32, rng: &mut impl Rng) {
        let h = height.max(1);
        let mut new_cells = vec![None; h];
        let copy = self.cells.len().min(h);
        new_cells[..copy].clone_from_slice(&self.cells[..copy]);
        self.cells = new_cells;
        let mut new_glyphs = vec![' '; h];
        let gcopy = self.glyphs.len().min(h);
        new_glyphs[..gcopy].clone_from_slice(&self.glyphs[..gcopy]);
        self.glyphs = new_glyphs;
        self.height = h;
        self.trail_len = scaled_trail_len(h, trail_factor, rng);
    }

    pub fn update(&mut self, charset: &Charset, rng: &mut impl Rng) {
        if self.delay > 0 {
            self.delay -= 1;
            return;
        }

        self.head += self.speed;

        for c in self.cells.iter_mut() {
            *c = None;
        }

        let head_y = self.head.floor() as i32;
        for i in 0..self.trail_len {
            let y = head_y - i as i32;
            if y < 0 || y >= self.height as i32 {
                continue;
            }
            let yi = y as usize;
            let age = i as f32 / self.trail_len.max(1) as f32;
            let is_lead = i == 0;
            // Lead always refreshes; trail occasionally glitches.
            if is_lead || self.glyphs[yi] == ' ' || rng.gen_bool(0.12) {
                self.glyphs[yi] = charset.pick(rng);
            }
            self.cells[yi] = Some(Cell {
                ch: self.glyphs[yi],
                age,
                is_lead,
            });
        }

        if head_y - self.trail_len as i32 > self.height as i32 {
            self.head = offscreen_head(self.height, rng);
            // Jitter around the column base. Multiplying the live speed
            // would random-walk without bound.
            self.speed = self.base_speed * rng.gen_range(0.85..1.15);
            self.delay = rng.gen_range(0..25);
            self.glyphs.fill(' ');
        }
    }
}

#[derive(Debug)]
pub struct RainField {
    pub columns: Vec<Column>,
    pub width: u16,
    pub height: u16,
}

impl RainField {
    pub fn new(
        width: u16,
        height: u16,
        density: f32,
        speed: f32,
        trail: f32,
        rng: &mut impl Rng,
    ) -> Self {
        let mut field = Self {
            columns: Vec::new(),
            width: 0,
            height: 0,
        };
        field.rebuild(width, height, density, speed, trail, rng);
        field
    }

    pub fn rebuild(
        &mut self,
        width: u16,
        height: u16,
        density: f32,
        speed: f32,
        trail: f32,
        rng: &mut impl Rng,
    ) {
        self.width = width;
        self.height = height;
        let cols = ((width as f32) * density.clamp(0.05, 1.0)).round() as usize;
        let cols = cols.max(1).min(width.max(1) as usize);
        let mut xs: Vec<u16> = (0..width).collect();
        for i in 0..cols.min(xs.len()) {
            let j = rng.gen_range(i..xs.len());
            xs.swap(i, j);
        }
        xs.truncate(cols);
        xs.sort_unstable();

        self.columns = xs
            .into_iter()
            .map(|x| Column::new(x, height as usize, speed, trail, rng))
            .collect();
    }

    pub fn resize(
        &mut self,
        width: u16,
        height: u16,
        density: f32,
        speed: f32,
        trail: f32,
        rng: &mut impl Rng,
    ) {
        if width != self.width {
            self.rebuild(width, height, density, speed, trail, rng);
            return;
        }
        if height != self.height {
            for col in &mut self.columns {
                col.resize(height as usize, trail, rng);
            }
            self.height = height;
        }
    }

    pub fn set_speed(&mut self, speed: f32, rng: &mut impl Rng) {
        for col in &mut self.columns {
            let tuned = speed * rng.gen_range(0.45..1.55);
            col.base_speed = tuned;
            col.speed = tuned;
        }
    }

    pub fn tick(&mut self, charset: &Charset, rng: &mut impl Rng) {
        for col in &mut self.columns {
            col.update(charset, rng);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charset::{Charset, CharsetId};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn short_terminal_does_not_panic() {
        let mut rng = StdRng::seed_from_u64(4);
        let charset = Charset::from_id(CharsetId::Binary);

        for height in [0_usize, 1, 2, 3] {
            for trail in [0.1_f32, 1.0] {
                let mut col = Column::new(0, height, 4.0, trail, &mut rng);
                assert_eq!(col.height, height.max(1));
                assert!(col.trail_len >= 1 && col.trail_len <= col.height);
                assert_eq!(col.cells.len(), col.height);

                // Resize path used to clamp(4.0, height) after copying cells.
                let mut tall = Column::new(1, 24, 1.0, trail, &mut rng);
                tall.resize(height, trail, &mut rng);
                assert!(tall.trail_len >= 1 && tall.trail_len <= tall.height);

                for _ in 0..80 {
                    col.update(&charset, &mut rng);
                    tall.update(&charset, &mut rng);
                }
                assert!(col.trail_len <= col.height);
                assert!(col.cells.iter().flatten().count() <= col.trail_len);
            }
        }

        let mut field = RainField::new(12, 20, 0.5, 2.0, 0.8, &mut rng);
        field.resize(12, 3, 0.5, 2.0, 0.8, &mut rng);
        field.resize(12, 1, 0.5, 2.0, 0.1, &mut rng);
        field.resize(2, 2, 1.0, 3.0, 1.0, &mut rng);
        for _ in 0..80 {
            field.tick(&charset, &mut rng);
        }
        assert!(!field.columns.is_empty());
        for col in &field.columns {
            assert!(col.height <= 2);
            assert!(col.trail_len >= 1 && col.trail_len <= col.height);
            assert_eq!(col.cells.len(), col.height);
        }
    }

    #[test]
    fn respawn_speed_stays_near_base() {
        let mut rng = StdRng::seed_from_u64(11);
        let charset = Charset::from_id(CharsetId::Binary);
        let mut col = Column::new(0, 10, 1.5, 0.6, &mut rng);
        let base = col.base_speed;
        assert!(base > 0.0);
        assert!((col.speed - base).abs() < f32::EPSILON);

        // A drifted live speed must be pulled back onto the base, not scaled further.
        for _ in 0..40 {
            col.speed = base * 8.0;
            col.head = col.height as f32 + col.trail_len as f32 + 2.0;
            col.delay = 0;
            col.update(&charset, &mut rng);
            let ratio = col.speed / base;
            assert!(
                (0.85..1.15).contains(&ratio),
                "respawn speed ratio {ratio} left the band around base {base}"
            );
        }

        let mut field = RainField::new(4, 12, 1.0, 2.0, 0.5, &mut rng);
        field.set_speed(3.0, &mut rng);
        for col in &mut field.columns {
            let base = col.base_speed;
            assert!((col.speed - base).abs() < f32::EPSILON);
            assert!((0.45..1.55).contains(&(base / 3.0)));
            col.speed = base * 20.0;
            col.head = col.height as f32 + col.trail_len as f32 + 2.0;
            col.delay = 0;
            col.update(&charset, &mut rng);
            let ratio = col.speed / base;
            assert!(
                (0.85..1.15).contains(&ratio),
                "retuned column respawn ratio {ratio} left the band around base {base}"
            );
        }
    }
}
