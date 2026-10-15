use std::f64::consts::{PI, TAU};

use rand::{Rng, SeedableRng, rngs::StdRng};
use ratatui::{buffer::Buffer, layout::Rect};

use crate::{
    graphics::{centered, centered_text, double_box, progress_color, put, text},
    model::TimerDetails,
    palette::{DOUBLE_GRADIENTS, SINGLE_GRADIENTS},
};

const MAP_SIZE: usize = 800;

pub struct Effects {
    seed: u64,
    height_maps: Option<(Vec<f64>, Vec<f64>)>,
    circle_epoch: Option<(u64, u16, u16)>,
    circles: Vec<Circle>,
    lava_gradient: Option<(u64, &'static [u8])>,
}

struct Circle {
    x: f64,
    y: f64,
    radius: i32,
    vx: f64,
    vy: f64,
    color: u8,
}

impl Default for Effects {
    fn default() -> Self {
        Self::new()
    }
}

impl Effects {
    pub fn new() -> Self {
        Self {
            seed: rand::random(),
            height_maps: None,
            circle_epoch: None,
            circles: Vec::new(),
            lava_gradient: None,
        }
    }

    pub fn draw(&mut self, name: &str, details: TimerDetails, area: Rect, buf: &mut Buffer) {
        match name {
            "bar" => bar(details, area, buf),
            "pie" | "sweep" | "colorwheel" => disk(name, details, area, buf),
            "dots" => dots(details, area, buf),
            "circles" => self.circles(details, area, buf),
            "sine" => sine(details, area, buf),
            "lava" => self.lava(details, area, buf),
            _ => {}
        }
    }

    fn epoch_rng(&self, details: TimerDetails, salt: u64) -> StdRng {
        // An epoch's choices do not depend on how many frames were actually drawn.
        StdRng::seed_from_u64(
            self.seed
                .wrapping_add((details.elapsed / 20.0).floor() as u64)
                .wrapping_add(salt),
        )
    }

    fn circles(&mut self, details: TimerDetails, area: Rect, buf: &mut Buffer) {
        let height = area.height.saturating_sub(10);
        let field = centered(area, area.width, height);
        for y in 0..i32::from(field.height) {
            for x in 0..i32::from(field.width) {
                put(buf, field, x, y, ".", 233);
            }
        }
        let epoch = (
            (details.elapsed / 20.0).floor() as u64,
            area.width,
            area.height,
        );
        if self.circle_epoch != Some(epoch) {
            let mut rng = self.epoch_rng(details, 0);
            let increments = [-2.0, -1.0, 1.0, 2.0];
            self.circles = [18, 88, 28, 92, 166, 118, 50]
                .into_iter()
                .map(|color| Circle {
                    x: rng.random_range(0..=area.width) as f64,
                    y: rng.random_range(0..=area.height) as f64,
                    radius: rng.random_range(5..=10),
                    vx: increments[rng.random_range(0..increments.len())] * 10.0,
                    vy: increments[rng.random_range(0..increments.len())] * 10.0,
                    color,
                })
                .collect();
            self.circle_epoch = Some(epoch);
        }
        let phase = details.elapsed.rem_euclid(20.0);
        let remaining = 200 - (phase * 10.0).floor() as usize;
        let fade = [
            233, 233, 233, 233, 233, 233, 234, 235, 236, 237, 238, 239, 240,
        ];
        for circle in &self.circles {
            let radius = circle.radius;
            let right = f64::from(field.width.saturating_sub(1));
            let bottom = f64::from(field.height.saturating_sub(1));
            let x = reflected(circle.x.min(right), circle.vx, phase, right);
            let y = reflected(circle.y.min(bottom), circle.vy, phase, bottom);
            let color = if remaining < fade.len() {
                fade[remaining]
            } else {
                circle.color
            };
            for cy in -radius..=radius {
                for cx in -radius..=radius {
                    if cx * cx + cy * cy < radius * radius {
                        for half in 0..2 {
                            put(
                                buf,
                                field,
                                x.round() as i32 + cx * 2 - 1 + half,
                                y.round() as i32 + cy,
                                "•",
                                color,
                            );
                        }
                    }
                }
            }
        }
    }

    fn lava(&mut self, details: TimerDetails, area: Rect, buf: &mut Buffer) {
        let epoch = (details.elapsed / 20.0).floor() as u64;
        if self.lava_gradient.map(|(cached, _)| cached) != Some(epoch) {
            let mut rng = self.epoch_rng(details, 1);
            self.lava_gradient = Some((
                epoch,
                DOUBLE_GRADIENTS[rng.random_range(0..DOUBLE_GRADIENTS.len())].1,
            ));
        }
        let gradient = self.lava_gradient.unwrap().1;
        let maps = self.height_maps.get_or_insert_with(height_maps);
        let cols = (f64::from(area.width) * 0.75).floor() as u16;
        let rows = (f64::from(area.height) * 0.6).floor() as u16;
        let frame = centered(
            area,
            cols.saturating_add(4).min(area.width),
            rows.saturating_add(4).min(area.height),
        );
        let seed_x = details.elapsed * 0.2;
        let seed_y = details.elapsed * 0.3;
        let origin =
            |phase: f64| (((phase.cos() + 1.0) / 2.0) * MAP_SIZE as f64 / 2.0).floor() as usize;
        let (x1, y1) = (origin(seed_x + 0.4 + PI), origin(seed_y - 0.1));
        let (x2, y2) = (origin(-seed_x + 1.2), origin(-seed_y - 0.8 + PI));
        for row in 0..rows {
            for col in 0..cols {
                let sample = |map: &[f64], x: usize, y: usize| {
                    map[((usize::from(row) + y) % MAP_SIZE) * MAP_SIZE
                        + (usize::from(col) + x) % MAP_SIZE]
                };
                let height = sample(&maps.0, x1, y1) + sample(&maps.1, x2, y2);
                let index = (height * (gradient.len() * 2) as f64 * 7.0).floor() as usize
                    % (gradient.len() * 2);
                let color = gradient[if index < gradient.len() {
                    index
                } else {
                    gradient.len() * 2 - 1 - index
                }];
                put(
                    buf,
                    frame,
                    i32::from(col) + 2,
                    i32::from(row) + 2,
                    "•",
                    color,
                );
            }
        }
        double_box(buf, frame, progress_color(details.progress()));
        centered_text(
            buf,
            frame,
            0,
            &format!(" {} remaining ", details.remaining_text()),
            15,
        );
    }
}

fn bar(details: TimerDetails, area: Rect, buf: &mut Buffer) {
    let length = i32::from(area.width).saturating_sub(19).max(0);
    let complete = (details.progress() * f64::from(length)).round() as i32;
    let elapsed = details.elapsed_text();
    let remaining = details.remaining_text();
    let width = elapsed.len() as i32 + 3 + length + 3 + remaining.len() as i32 + 1;
    let x = ((i32::from(area.width) - width) / 2).max(0);
    let y = i32::from(area.height.saturating_sub(1)) / 2;
    text(buf, area, x, y, &format!("{elapsed} | "), 15);
    let bar_x = x + elapsed.len() as i32 + 3;
    for col in 0..length {
        let (symbol, color) = if col < complete {
            ("█", progress_color(details.progress()))
        } else {
            ("░", 239)
        };
        put(buf, area, bar_x + col, y, symbol, color);
    }
    text(
        buf,
        area,
        bar_x + length,
        y,
        &format!(" | {remaining} "),
        15,
    );
}

fn dots(details: TimerDetails, area: Rect, buf: &mut Buffer) {
    let cols = area.width.saturating_sub(10);
    if cols == 0 {
        return;
    }
    let max_rows = (area.height / 2).saturating_sub(8);
    let rows = ((details.duration * 10.0 / f64::from(cols)).ceil() as u16).min(max_rows);
    let frame = centered(area, cols.saturating_add(4), rows.saturating_add(4));
    let total = u32::from(cols) * u32::from(rows);
    let complete = (details.progress() * f64::from(total)).round() as u32;
    let color = progress_color(details.progress());
    for index in 0..total {
        put(
            buf,
            frame,
            (index % u32::from(cols)) as i32 + 2,
            (index / u32::from(cols)) as i32 + 2,
            "●",
            if index < complete { color } else { 232 },
        );
    }
    double_box(buf, frame, color);
    centered_text(
        buf,
        frame,
        0,
        &format!(" {} remaining ", details.remaining_text()),
        15,
    );
}

fn disk(name: &str, details: TimerDetails, area: Rect, buf: &mut Buffer) {
    let margin = if name == "sweep" { 4 } else { 2 };
    let radius = (i32::from(area.height).min(i32::from(area.width) / 2) / 2 - margin).max(0);
    let width = (radius * 2 + 1) * 2;
    let height = radius * 2 + 1;
    let left = (i32::from(area.width) - width) / 2;
    let top = (i32::from(area.height) - height) / 2;
    let wheel = if name == "colorwheel" {
        wheel_gradient(details.progress())
    } else {
        [0; 24]
    };
    let rotation = (details.legacy_tick().floor() as usize + 1) % 24;
    for y in -radius..=radius {
        for x in -radius..=radius {
            if x * x + y * y >= radius * radius {
                continue;
            }
            // Inverse of SquarePieChart's clockwise matrix rotation.
            let percent = (-f64::from(x)).atan2(f64::from(y)) / TAU + 0.5;
            let (symbol, color) = match name {
                "pie" => (
                    "█",
                    if percent <= details.progress() {
                        progress_color(details.progress())
                    } else {
                        233
                    },
                ),
                "sweep" => {
                    if percent <= 1.0 - details.progress() {
                        (".", 233)
                    } else if percent <= 1.01 - details.progress() {
                        ("█", 54)
                    } else {
                        continue;
                    }
                }
                _ => {
                    let slice = (percent / 0.0417).ceil().max(1.0) as usize - 1;
                    ("█", wheel[(slice + 24 - rotation) % 24])
                }
            };
            for half in 0..2 {
                put(
                    buf,
                    area,
                    left + (x + radius) * 2 + half,
                    top + y + radius,
                    symbol,
                    color,
                );
            }
        }
    }
}

fn gradient(name: &str) -> &'static [u8] {
    SINGLE_GRADIENTS
        .iter()
        .find(|(key, _)| *key == name)
        .unwrap()
        .1
}

fn wheel_gradient(progress: f64) -> [u8; 24] {
    let names = if progress > 0.9 {
        [
            "RED_3B_TO_MAGENTA_2A",
            "DARKORANGE_3B_TO_MEDIUMORCHID_1A",
            "ORANGERED_1_TO_MEDIUMORCHID_1B",
            "RED_1_TO_MAGENTA_1",
        ]
    } else if progress > 0.7 {
        [
            "YELLOW_3B_TO_LIGHTSTEELBLUE_1",
            "YELLOW_2_TO_LIGHTCYAN_1",
            "YELLOW_1_TO_GREY_100",
            "GOLD_1_TO_THISTLE_1",
        ]
    } else {
        [
            "GREEN_4_TO_DODGERBLUE_1",
            "GREEN_3A_TO_DEEPSKYBLUE_1",
            "GREEN_3B_TO_TURQUOISE_2",
            "GREEN_1_TO_CYAN_1",
        ]
    };
    let mut wheel = [0; 24];
    for (i, name) in names.iter().enumerate() {
        let colors = gradient(name);
        for j in 0..6 {
            wheel[i * 6 + j] = colors[if i % 2 == 1 { 5 - j } else { j }];
        }
    }
    wheel
}

fn sine(details: TimerDetails, area: Rect, buf: &mut Buffer) {
    let height = (area.height / 4 * 2 + 1).min(area.height);
    let graph = centered(area, area.width, height);
    let columns = (details.progress() * f64::from(area.width.saturating_sub(8)))
        .floor()
        .max(5.0) as i32;
    let colors = gradient(if details.progress() > 0.9 {
        "RED_3B_TO_MAGENTA_2A"
    } else if details.progress() > 0.7 {
        "YELLOW_3B_TO_LIGHTSTEELBLUE_1"
    } else {
        "BLUE_1_TO_CYAN_1"
    });
    let origin = i32::from(height) / 2;
    let amplitude = i32::from(height.saturating_sub(2)) / 2;
    for col in 0..columns.min(i32::from(graph.width)) {
        let value = ((f64::from(col) + details.legacy_tick()) / 40.0 * TAU).sin();
        let row = (f64::from(origin) - value * f64::from(amplitude)).floor() as i32;
        for y in row.min(origin)..=row.max(origin) {
            let color = colors
                [(y as usize * colors.len() / usize::from(height.max(1))).min(colors.len() - 1)];
            put(buf, graph, col, y, "•", color);
        }
    }
}

fn reflected(start: f64, velocity: f64, elapsed: f64, limit: f64) -> f64 {
    if limit == 0.0 {
        return 0.0;
    }
    let position = (start + velocity * elapsed).rem_euclid(limit * 2.0);
    if position > limit {
        limit * 2.0 - position
    } else {
        position
    }
}

fn height_maps() -> (Vec<f64>, Vec<f64>) {
    let mut first = Vec::with_capacity(MAP_SIZE * MAP_SIZE);
    let mut second = Vec::with_capacity(MAP_SIZE * MAP_SIZE);
    for row in 0..MAP_SIZE {
        for col in 0..MAP_SIZE {
            let x = col as f64 - MAP_SIZE as f64 / 2.0;
            let y = row as f64 - MAP_SIZE as f64 / 2.0;
            first.push(((x.hypot(y) * (3.0 * PI / (MAP_SIZE as f64 / 2.0))).sin() + 1.0) / 2.0);
            second.push(
                (((x * 0.8).hypot(y * 1.3) * 0.022).sin()
                    + ((x * 1.35).hypot(y * 0.45) * 0.022).cos()
                    + 2.0)
                    / 4.0,
            );
        }
    }
    (first, second)
}
