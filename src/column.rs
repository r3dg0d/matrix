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
    pub trail_len: usize,
    pub cells: Vec<Option<Cell>>,
    /// Persistent glyphs so mid-trail characters flicker instead of constantly changing.
    glyphs: Vec<char>,
    pub height: usize,
    /// Frames until this column respawns after going off-screen.
    pub delay: u32,
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
        let trail_len = ((h as f32) * trail_factor * rng.gen_range(0.55..1.15))
            .round()
            .clamp(4.0, h as f32) as usize;
        let speed = speed_base * rng.gen_range(0.45..1.55);
        Self {
            x,
            head: -(rng.gen_range(0.0..(h as f32 * 0.8))),
            speed,
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
        self.trail_len = ((h as f32) * trail_factor * rng.gen_range(0.55..1.15))
            .round()
            .clamp(4.0, h as f32) as usize;
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
            self.head = -(rng.gen_range(1.0..(self.height as f32 * 0.5)));
            self.speed *= rng.gen_range(0.85..1.15);
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
            col.speed = speed * rng.gen_range(0.45..1.55);
        }
    }

    pub fn tick(&mut self, charset: &Charset, rng: &mut impl Rng) {
        for col in &mut self.columns {
            col.update(charset, rng);
        }
    }
}
