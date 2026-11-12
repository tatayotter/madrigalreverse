//! Reads the character's world position.
//!
//! Typing `/position` in chat makes the game print a green line in the chat box:
//! `Position: x = 7174.688477, y = 97.326149, z = 3514.464844, angle = 171.298828`.
//! The bot types the command, finds that line on screen and reads it with OCR.
//! `x` and `z` are the ground plane; `y` is height.

use libscreenshot::ImageBuffer;
use slog::Logger;

use crate::{data::Bounds, ocr};

/// Color of the "Position: ..." line in the chat box.
pub const POSITION_COLOR: [u8; 3] = [204, 255, 204];
const COLOR_TOLERANCE: u8 = 10;
/// Fewer matching pixels than this can't be a line of text.
const MIN_LINE_PIXELS: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub angle: Option<f32>,
}

impl Position {
    /// Distance on the ground (ignores height).
    pub fn distance_to(&self, other: &Position) -> f32 {
        ((self.x - other.x).powi(2) + (self.z - other.z).powi(2)).sqrt()
    }
}

fn matches(px: &[u8]) -> bool {
    (0..3).all(|i| px[i].abs_diff(POSITION_COLOR[i]) <= COLOR_TOLERANCE)
}

/// Find the newest (lowest) green Position line on screen.
pub fn find_line(image: &ImageBuffer) -> Option<Bounds> {
    // Pixel count and x range for every row with green pixels
    let mut rows: Vec<(u32, usize, u32, u32)> = Vec::new();
    for y in 0..image.height() {
        let (mut count, mut min_x, mut max_x) = (0usize, u32::MAX, 0u32);
        for x in 0..image.width() {
            if matches(&image.get_pixel(x, y).0) {
                count += 1;
                min_x = min_x.min(x);
                max_x = max_x.max(x);
            }
        }
        if count > 0 {
            rows.push((y, count, min_x, max_x));
        }
    }

    // Group rows into lines (gaps of up to 2 rows are inside a line), keep the lowest real one
    let mut best: Option<Bounds> = None;
    let mut i = 0;
    while i < rows.len() {
        let mut j = i;
        while j + 1 < rows.len() && rows[j + 1].0 <= rows[j].0 + 3 {
            j += 1;
        }
        let group = &rows[i..=j];
        let total: usize = group.iter().map(|r| r.1).sum();
        if total >= MIN_LINE_PIXELS {
            let min_x = group.iter().map(|r| r.2).min().unwrap_or(0);
            let max_x = group.iter().map(|r| r.3).max().unwrap_or(0);
            let (top, bottom) = (group[0].0, group[group.len() - 1].0);
            best = Some(Bounds::new(min_x, top, max_x - min_x + 1, bottom - top + 1));
        }
        i = j + 1;
    }
    best
}

/// Fix characters OCR commonly confuses inside numbers.
fn clean_number(raw: &str) -> Option<f32> {
    let fixed: String = raw
        .chars()
        .map(|c| match c {
            'O' | 'o' | 'D' => '0',
            'l' | 'I' | '|' | 'i' => '1',
            'S' | 's' => '5',
            'B' => '8',
            ',' => '.',
            c => c,
        })
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    // OCR sometimes reads the "," after a number as "."
    fixed.trim_end_matches('.').parse().ok()
}

/// The number after `key =` in the text.
fn value_after(text: &str, key: &str) -> Option<f32> {
    let lower = text.to_lowercase();
    // Look for "key =" or "key=" (OCR may drop the spaces)
    let start = [format!("{key} ="), format!("{key}=")]
        .iter()
        .filter_map(|pat| lower.find(pat.as_str()).map(|i| i + pat.len()))
        .min()?;
    let rest: Vec<char> = text[start..].trim_start().chars().collect();
    // The number ends at a space, or at a comma that isn't followed by a digit
    let mut end = rest.len();
    for (i, c) in rest.iter().enumerate() {
        let next_is_digit = rest.get(i + 1).map_or(false, |d| d.is_ascii_digit());
        if *c == ' ' || ((*c == ',' || *c == '.') && !next_is_digit) {
            end = i;
            break;
        }
    }
    let token: String = rest[..end].iter().collect();
    clean_number(&token)
}

/// Parse `Position: x = 1.0, y = 2.0, z = 3.0, angle = 4.0`.
pub fn parse(text: &str) -> Option<Position> {
    Some(Position {
        x: value_after(text, "x")?,
        y: value_after(text, "y").unwrap_or(0.0),
        z: value_after(text, "z")?,
        angle: value_after(text, "angle"),
    })
}

/// Read the position line currently on screen.
pub fn read(image: &ImageBuffer, logger: &Logger) -> Option<Position> {
    let bounds = find_line(image)?;
    let text = ocr::read_name(image, bounds, POSITION_COLOR, COLOR_TOLERANCE / 2)?;
    let position = parse(&text);
    slog::debug!(logger, "Read position"; "ocr" => &text, "parsed" => format!("{:?}", position));
    position
}

/// Color of the region name in the map window's top bar ("Madrigal > Flaris > ...").
pub const REGION_COLOR: [u8; 3] = [255, 185, 0];
const REGION_TOLERANCE: u8 = 40;
/// Where the map's top bar text sits in the 800x600 client.
const REGION_AREA: (u32, u32, u32, u32) = (100, 25, 420, 58);

/// Find the region name (the orange word group in the map's top bar), without the ▼ after it.
pub fn find_region(image: &ImageBuffer) -> Option<Bounds> {
    let (x0, y0, x1, y1) = REGION_AREA;
    let (x1, y1) = (x1.min(image.width()), y1.min(image.height()));
    let near = |px: &[u8]| (0..3).all(|i| px[i].abs_diff(REGION_COLOR[i]) <= REGION_TOLERANCE);
    // For each column: does it have orange pixels, and their vertical extent
    let mut cols: Vec<(u32, u32, u32)> = Vec::new();
    for x in x0..x1 {
        let ys: Vec<u32> = (y0..y1).filter(|&y| near(&image.get_pixel(x, y).0)).collect();
        if let (Some(&top), Some(&bottom)) = (ys.first(), ys.last()) {
            cols.push((x, top, bottom));
        }
    }
    let first = cols.first()?;
    // Letters are at most a few pixels apart, words ~6; the ▼ comes after a wider gap.
    let mut end = 0;
    for i in 1..cols.len() {
        if cols[i].0 - cols[i - 1].0 > 8 {
            break;
        }
        end = i;
    }
    let group = &cols[..=end];
    let top = group.iter().map(|c| c.1).min()?;
    let bottom = group.iter().map(|c| c.2).max()?;
    let right = group[end].0;
    if right - first.0 < 8 {
        return None;
    }
    Some(Bounds::new(first.0, top, right - first.0 + 1, bottom - top + 1))
}

/// Read the region name from an open map window.
pub fn read_region(image: &ImageBuffer, logger: &Logger) -> Option<String> {
    let bounds = find_region(image)?;
    let text = ocr::read_name(image, bounds, REGION_COLOR, REGION_TOLERANCE / 2)?;
    let name = text.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
    slog::debug!(logger, "Read region"; "ocr" => &text, "bounds" => format!("{:?}", bounds));
    if name.is_empty() { None } else { Some(name) }
}

/// Same region, allowing for a misread letter.
pub fn same_region(a: &str, b: &str) -> bool {
    let (a, b) = (crate::monster_db::normalize(a), crate::monster_db::normalize(b));
    if a == b {
        return true;
    }
    let len = a.chars().count().max(b.chars().count());
    len >= 5 && strsim_distance(&a, &b) <= 1
}

fn strsim_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn parses_game_text() {
        let p = parse("Position: x = 7174.688477, y = 97.326149, z = 3514.464844, angle = 171.298828").unwrap();
        assert_eq!(p.x, 7174.688477);
        assert_eq!(p.y, 97.326149);
        assert_eq!(p.z, 3514.464844);
        assert_eq!(p.angle, Some(171.298828));
    }

    #[test]
    fn parses_ocr_noise() {
        // Missing spaces, comma decimal, O for 0, l for 1
        let p = parse("Position: x=7l74.688477, y = 97.326149, z = 35O4,5, angle = -12.5").unwrap();
        assert_eq!(p.x, 7174.688477);
        assert_eq!(p.z, 3504.5);
        assert_eq!(p.angle, Some(-12.5));
        assert!(parse("Hackern00b : /position").is_none());
        // Separators read as "." and junk characters
        let p = parse("Position: x = 7174.686477. y = $7.326149. z = 3514.464644. angle = 171.298828").unwrap();
        assert_eq!((p.x, p.y, p.z), (7174.686477, 7.326149, 3514.464644));
    }

    #[test]
    fn distance_ignores_height() {
        let a = Position { x: 0.0, y: 0.0, z: 0.0, angle: None };
        let b = Position { x: 3.0, y: 50.0, z: 4.0, angle: None };
        assert_eq!(a.distance_to(&b), 5.0);
    }

    #[test]
    fn finds_lowest_line() {
        let mut img = ImageBuffer::from_pixel(200, 100, Rgba([10, 10, 10, 255]));
        for (top, len) in [(20u32, 80u32), (60, 100)] {
            for y in top..top + 8 {
                for x in (10..10 + len).step_by(2) {
                    img.put_pixel(x, y, Rgba([204, 255, 204, 255]));
                }
            }
        }
        // A few stray green pixels below the lines are ignored
        img.put_pixel(50, 90, Rgba([204, 255, 204, 255]));
        let b = find_line(&img).unwrap();
        assert_eq!((b.x, b.y, b.h), (10, 60, 8));
    }

    #[test]
    fn same_region_tolerates_one_typo() {
        assert!(same_region("Flaris", "flaris"));
        assert!(same_region("Flaris", "Fiaris"));
        assert!(!same_region("Flaris", "Darkon"));
        assert!(!same_region("Saint Morning", "Flaris"));
    }

    #[test]
    fn finds_region_without_arrow() {
        let mut img = ImageBuffer::from_pixel(800, 600, Rgba([40, 20, 0, 255]));
        // "word" columns 175..=221 with 2-3px letter gaps, then an arrow at 231..=242
        for x in (175..=221).filter(|x| (x - 175) % 4 != 3) {
            for y in 41..54 {
                img.put_pixel(x, y, Rgba([255, 185, 0, 255]));
            }
        }
        for x in 231..=242 {
            for y in 42..52 {
                img.put_pixel(x, y, Rgba([255, 185, 0, 255]));
            }
        }
        let b = find_region(&img).unwrap();
        assert_eq!((b.x, b.x + b.w - 1, b.y), (175, 221, 41));
    }

    /// NEUZ_REGION_IMG=map.png (an 800x600 client crop) cargo test region_from_screenshot -- --ignored --nocapture
    #[test]
    #[ignore]
    fn region_from_screenshot() {
        let path = std::env::var("NEUZ_REGION_IMG").expect("NEUZ_REGION_IMG");
        let img = image::open(&path).unwrap().to_rgba8();
        let b = find_region(&img).expect("no region found");
        println!("region bounds: {:?}", b);
        let out = ocr::prepare(&img, b, REGION_COLOR, REGION_TOLERANCE / 2).unwrap();
        let gray = image::GrayImage::from_fn(out.width, out.height, |x, y| {
            image::Luma([out.bgra[((y * out.width + x) * 4) as usize]])
        });
        gray.save(std::env::var("NEUZ_REGION_OUT").unwrap_or_else(|_| "region.png".into())).unwrap();
    }

    /// NEUZ_POS_IMG=shot.png cargo test position::tests::read_from_screenshot -- --ignored --nocapture
    #[test]
    #[ignore]
    fn read_from_screenshot() {
        let path = std::env::var("NEUZ_POS_IMG").expect("NEUZ_POS_IMG");
        let img = image::open(&path).unwrap().to_rgba8();
        let b = find_line(&img).expect("no position line found");
        println!("line bounds: {:?}", b);
        let out = ocr::prepare(&img, b, POSITION_COLOR, COLOR_TOLERANCE / 2).unwrap();
        let gray = image::GrayImage::from_fn(out.width, out.height, |x, y| {
            image::Luma([out.bgra[((y * out.width + x) * 4) as usize]])
        });
        gray.save(std::env::var("NEUZ_POS_OUT").unwrap_or_else(|_| "position.png".into())).unwrap();
    }
}
