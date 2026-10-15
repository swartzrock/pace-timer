use std::f64::consts::{PI, TAU};

use rand::{Rng, SeedableRng, rngs::StdRng};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

use crate::{
    color::indexed_rgb,
    graphics::{
        centered, centered_text, double_box, gradient, mix, paired_pixels, progress_color,
        put_color, text,
    },
    model::TimerDetails,
};

const MAP_SIZE: usize = 800;
// Related warm ramps stay saturated while crossfading, including after quantization.
const LAVA_GRADIENTS: &[&[u8]] = &[
    &[16, 52, 88, 124, 160, 196, 202, 208, 214, 220, 226, 230],
    &[16, 52, 88, 124, 160, 196, 202, 208, 214, 215, 216, 223],
    &[16, 52, 88, 94, 130, 166, 172, 208, 214, 220, 226, 230],
];

pub struct Effects {
    seed: u64,
    height_maps: Option<(Vec<f64>, Vec<f64>)>,
    circle_epoch: Option<(u64, u16, u16)>,
    circles: Vec<Circle>,
    lava_gradient: Option<(u64, &'static [u8], &'static [u8])>,
}

struct Circle {
    x: f64,
    y: f64,
    radius: f64,
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

    fn epoch_rng(&self, epoch: u64, salt: u64) -> StdRng {
        // An epoch's choices do not depend on how many frames were actually drawn.
        StdRng::seed_from_u64(self.seed.wrapping_add(epoch).wrapping_add(salt))
    }

    fn circles(&mut self, details: TimerDetails, area: Rect, buf: &mut Buffer) {
        let field = centered(area, area.width, area.height.saturating_sub(10));
        let epoch = (
            (details.elapsed / 20.0).floor() as u64,
            area.width,
            area.height,
        );
        if self.circle_epoch != Some(epoch) {
            let mut rng = self.epoch_rng(epoch.0, 0);
            let increments = [-2.0, -1.0, 1.0, 2.0];
            self.circles = [18, 88, 28, 92, 166, 118, 50]
                .into_iter()
                .map(|color| Circle {
                    x: rng.random_range(0..=field.width) as f64,
                    y: rng.random_range(0..=field.height) as f64 * 2.0,
                    radius: rng.random_range(5..=10) as f64 * 2.0,
                    vx: increments[rng.random_range(0..increments.len())] * 10.0,
                    vy: increments[rng.random_range(0..increments.len())] * 20.0,
                    color,
                })
                .collect();
            self.circle_epoch = Some(epoch);
        }
        let phase = details.elapsed.rem_euclid(20.0);
        // Both generations meet at the same dark field, including skipped frames.
        let fade = smoothstep(phase.min(20.0 - phase));
        let right = f64::from(field.width.saturating_sub(1));
        let bottom = f64::from(field.height.saturating_mul(2).saturating_sub(1));
        let positions: Vec<_> = self
            .circles
            .iter()
            .map(|circle| {
                (
                    reflected(circle.x.min(right), circle.vx, phase, right),
                    reflected(circle.y.min(bottom), circle.vy, phase, bottom),
                )
            })
            .collect();
        for row in 0..i32::from(field.height) {
            for col in 0..i32::from(field.width) {
                let sample = |half| {
                    let mut color = indexed_rgb(233);
                    for (circle, &(x, y)) in self.circles.iter().zip(&positions) {
                        let dx = f64::from(col) - x;
                        let dy = f64::from(row * 2 + half) - y;
                        let distance = dx.hypot(dy);
                        let coverage = (circle.radius + 0.5 - distance).clamp(0.0, 1.0);
                        if coverage == 0.0 {
                            continue;
                        }
                        let light = (0.72 - (dx + dy) / (circle.radius * 5.0)).clamp(0.35, 1.0);
                        let lit = mix(indexed_rgb(circle.color), Color::White, light * 0.3);
                        color = mix(color, lit, coverage * light * fade);
                    }
                    Some(color)
                };
                paired_pixels(buf, field, col, row, sample(0), sample(1));
            }
        }
    }

    fn lava(&mut self, details: TimerDetails, area: Rect, buf: &mut Buffer) {
        let epoch = (details.elapsed / 20.0).floor() as u64;
        if self.lava_gradient.map(|(cached, _, _)| cached) != Some(epoch) {
            let select = |epoch| {
                let mut rng = self.epoch_rng(epoch, 1);
                LAVA_GRADIENTS[rng.random_range(0..LAVA_GRADIENTS.len())]
            };
            self.lava_gradient = Some((epoch, select(epoch), select(epoch.wrapping_add(1))));
        }
        let (_, from, to) = self.lava_gradient.unwrap();
        let blend = smoothstep(details.elapsed.rem_euclid(20.0) / 20.0);
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
        let origin = |phase: f64| (phase.cos() + 1.0) * MAP_SIZE as f64 / 4.0;
        let (x1, y1) = (origin(seed_x + 0.4 + PI), origin(seed_y - 0.1));
        let (x2, y2) = (origin(-seed_x + 1.2), origin(-seed_y - 0.8 + PI));
        for row in 0..rows {
            for col in 0..cols {
                let sample = |half| {
                    let x = f64::from(col);
                    let y = f64::from(row) * 2.0 + half;
                    let height = bilinear(&maps.0, MAP_SIZE, x + x1, y + y1)
                        + bilinear(&maps.1, MAP_SIZE, x + x2, y + y2);
                    // Broad lobes avoid sub-cell rainbow stripes in limited palettes.
                    let phase = (height * 2.0 + details.elapsed * 0.03).rem_euclid(2.0);
                    let position = if phase <= 1.0 { phase } else { 2.0 - phase };
                    Some(mix(gradient(from, position), gradient(to, position), blend))
                };
                paired_pixels(
                    buf,
                    frame,
                    i32::from(col) + 2,
                    i32::from(row) + 2,
                    sample(0.0),
                    sample(1.0),
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
    let complete = details.progress() * f64::from(length);
    let elapsed = details.elapsed_text();
    let remaining = details.remaining_text();
    let width = elapsed.len() as i32 + 3 + length + 3 + remaining.len() as i32 + 1;
    let x = ((i32::from(area.width) - width) / 2).max(0);
    let y = i32::from(area.height.saturating_sub(1)) / 2;
    text(buf, area, x, y, &format!("{elapsed} | "), 15);
    let bar_x = x + elapsed.len() as i32 + 3;
    let fractions = ["░", "▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"];
    for col in 0..length {
        let coverage = (complete - f64::from(col)).clamp(0.0, 1.0);
        let symbol = fractions[(coverage * 8.0).round() as usize];
        let color = mix(
            indexed_rgb(239),
            indexed_rgb(progress_color(details.progress())),
            coverage,
        );
        put_color(buf, area, bar_x + col, y, symbol, color);
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
    let complete = details.progress() * f64::from(total);
    let color = progress_color(details.progress());
    for index in 0..total {
        let coverage = (complete - f64::from(index)).clamp(0.0, 1.0);
        let symbol = if coverage == 0.0 {
            "○"
        } else if coverage == 1.0 {
            "●"
        } else {
            ["░", "▒", "▓"][((coverage * 3.0) as usize).min(2)]
        };
        put_color(
            buf,
            frame,
            (index % u32::from(cols)) as i32 + 2,
            (index / u32::from(cols)) as i32 + 2,
            symbol,
            mix(indexed_rgb(232), indexed_rgb(color), coverage),
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
    let margin = if name == "sweep" { 8.0 } else { 4.0 };
    let radius = (f64::from(area.width).min(f64::from(area.height) * 2.0) / 2.0 - margin).max(0.0);
    let progress = details.progress();
    for row in 0..i32::from(area.height) {
        for col in 0..i32::from(area.width) {
            let sample = |half| {
                let x = f64::from(col) + 0.5 - f64::from(area.width) / 2.0;
                let y = f64::from(row * 2 + half) + 0.5 - f64::from(area.height);
                let distance = x.hypot(y);
                let coverage = (radius + 0.5 - distance).clamp(0.0, 1.0);
                if coverage == 0.0 || radius == 0.0 {
                    return None;
                }
                let angle = x.atan2(-y).rem_euclid(TAU) / TAU;
                let color = match name {
                    "pie" => {
                        let fill = if progress == 0.0 {
                            0.0
                        } else if progress == 1.0 {
                            1.0
                        } else {
                            ((progress - angle) * TAU * distance + 0.5).clamp(0.0, 1.0)
                        };
                        mix(
                            indexed_rgb(233),
                            indexed_rgb(progress_color(progress)),
                            fill,
                        )
                    }
                    "sweep" => {
                        if progress == 1.0 {
                            return None;
                        }
                        let boundary = (angle - (1.0 - progress)) * TAU * distance;
                        let hand = (1.0 - boundary.abs() / 1.2).clamp(0.0, 1.0);
                        if boundary > 1.2 {
                            return None;
                        }
                        mix(indexed_rgb(233), indexed_rgb(171), hand)
                    }
                    _ => hue(angle - details.elapsed / 2.4),
                };
                Some(mix(Color::Black, color, coverage))
            };
            paired_pixels(buf, area, col, row, sample(0), sample(1));
        }
    }
}

fn hue(position: f64) -> Color {
    let channel = |offset: f64| {
        let wave = ((position + offset).rem_euclid(1.0) * 6.0 - 3.0).abs();
        (45.0 + 210.0 * (wave - 1.0).clamp(0.0, 1.0)).round() as u8
    };
    Color::Rgb(channel(0.0), channel(2.0 / 3.0), channel(1.0 / 3.0))
}

fn sine(details: TimerDetails, area: Rect, buf: &mut Buffer) {
    let height = (area.height / 4 * 2 + 1).min(area.height);
    let graph = centered(area, area.width, height);
    let columns = (details.progress() * f64::from(area.width.saturating_sub(8))).max(5.0);
    let origin = f64::from(height);
    let amplitude = f64::from(height.saturating_sub(2));
    let phase = details.elapsed.rem_euclid(4.0) / 4.0 * TAU;
    for col in 0..i32::from(graph.width) {
        let horizontal = (columns - f64::from(col)).clamp(0.0, 1.0);
        if horizontal == 0.0 {
            continue;
        }
        let waves = [(0.24, 0.10), (0.12, 0.20), (0.0, 1.0)].map(|(lag, opacity)| {
            let value = (f64::from(col) / 40.0 * TAU + phase - lag / 4.0 * TAU).sin();
            (origin - value * amplitude, opacity)
        });
        for row in 0..i32::from(graph.height) {
            let sample = |half| {
                let y = f64::from(row * 2 + half) + 0.5;
                let mut coverage: f64 = 0.0;
                for (wave, opacity) in waves {
                    let edge = ((y - origin.min(wave) + 0.5).min(origin.max(wave) - y + 0.5))
                        .clamp(0.0, 1.0);
                    coverage = coverage.max(edge * opacity);
                }
                if coverage == 0.0 {
                    return None;
                }
                let color = gradient(&[21, 33, 45, 51], y / (origin * 2.0).max(1.0));
                Some(mix(Color::Black, color, coverage * horizontal))
            };
            paired_pixels(buf, graph, col, row, sample(0), sample(1));
        }
    }
}

fn smoothstep(value: f64) -> f64 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn bilinear(map: &[f64], size: usize, x: f64, y: f64) -> f64 {
    let x = x.rem_euclid(size as f64);
    let y = y.rem_euclid(size as f64);
    let (left, top) = (x.floor() as usize, y.floor() as usize);
    let (right, bottom) = ((left + 1) % size, (top + 1) % size);
    let upper = map[top * size + left] * (1.0 - x.fract()) + map[top * size + right] * x.fract();
    let lower =
        map[bottom * size + left] * (1.0 - x.fract()) + map[bottom * size + right] * x.fract();
    upper * (1.0 - y.fract()) + lower * y.fract()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lava_256_avoids_dense_high_contrast_stripes() {
        use crate::color::{OutputPolicy, adapt_buffer, color_rgb};

        let area = Rect::new(0, 0, 120, 40);
        for seed in 0..4 {
            let mut effects = Effects::new();
            effects.seed = seed;
            for elapsed in [0.0, 6.0, 10.0, 19.9, 20.1] {
                let mut buf = Buffer::empty(area);
                effects.draw(
                    "lava",
                    TimerDetails {
                        elapsed,
                        duration: 60.0,
                    },
                    area,
                    &mut buf,
                );
                adapt_buffer(&mut buf, OutputPolicy::Ansi256);
                let (mut abrupt, mut edges) = (0, 0);
                for y in 0..area.height {
                    for x in 0..area.width - 1 {
                        let (a, b) = (&buf[(x, y)], &buf[(x + 1, y)]);
                        if a.symbol() != "▀" || b.symbol() != "▀" {
                            continue;
                        }
                        for (a, b) in [(a.fg, b.fg), (a.bg, b.bg)] {
                            let (r, g, b1) = color_rgb(a).unwrap();
                            let (r2, g2, b2) = color_rgb(b).unwrap();
                            edges += 1;
                            // One cube step is unavoidable; repeated jumps beyond it alias.
                            if r.abs_diff(r2).max(g.abs_diff(g2)).max(b1.abs_diff(b2)) > 95 {
                                abrupt += 1;
                            }
                        }
                    }
                }
                assert!(edges > 1000);
                assert!(
                    abrupt * 100 < edges,
                    "seed {seed}, {elapsed}s: {abrupt}/{edges} abrupt edges"
                );
            }
        }
    }

    #[test]
    fn bilinear_preserves_points_and_interpolates_interior_and_wrapped_edges() {
        let map = [0.0, 2.0, 4.0, 6.0];
        assert_eq!(bilinear(&map, 2, 0.0, 0.0), 0.0);
        assert_eq!(bilinear(&map, 2, 1.0, 1.0), 6.0);
        assert_eq!(bilinear(&map, 2, 0.25, 0.75), 3.5);
        assert_eq!(bilinear(&map, 2, 1.5, 1.5), 3.0);
        assert_eq!(bilinear(&map, 2, -0.5, -0.5), 3.0);
        assert_eq!(bilinear(&map, 2, 2.0, 2.0), 0.0);
    }

    #[test]
    fn hue_wrap_and_palette_stop_are_continuous() {
        use crate::color::color_rgb;
        let distance = |a, b| {
            let (ar, ag, ab) = color_rgb(a).unwrap();
            let (br, bg, bb) = color_rgb(b).unwrap();
            u16::from(ar.abs_diff(br)) + u16::from(ag.abs_diff(bg)) + u16::from(ab.abs_diff(bb))
        };
        assert!(distance(hue(0.9999), hue(0.0001)) <= 2);
        let stops = crate::palette::DOUBLE_GRADIENTS[0].1;
        let stop = 1.0 / (stops.len() - 1) as f64;
        assert!(
            distance(
                gradient(stops, stop - 0.0001),
                gradient(stops, stop + 0.0001)
            ) <= 2
        );
    }

    #[test]
    fn seeded_lava_maps_are_retained_after_frames_and_resize() {
        let mut effects = Effects::new();
        effects.seed = 42;
        let details = TimerDetails {
            elapsed: 5.0,
            duration: 60.0,
        };
        let area = Rect::new(0, 0, 40, 20);
        effects.draw("lava", details, area, &mut Buffer::empty(area));
        let maps = effects.height_maps.as_ref().unwrap();
        assert_eq!(maps.0.len(), MAP_SIZE * MAP_SIZE);
        assert_eq!(maps.1.len(), MAP_SIZE * MAP_SIZE);
        let pointers = (maps.0.as_ptr(), maps.1.as_ptr());
        let resized = Rect::new(2, 3, 20, 10);
        effects.draw(
            "lava",
            details,
            resized,
            &mut Buffer::empty(Rect::new(0, 0, 40, 20)),
        );
        let maps = effects.height_maps.as_ref().unwrap();
        assert_eq!(pointers, (maps.0.as_ptr(), maps.1.as_ptr()));
    }
}
