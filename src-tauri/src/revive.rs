//! The Resurrection window that shows when the character dies.
//!
//! It has four stacked gold buttons: Lodestar (nearest town), Lodelight (saved spawn
//! point), Flyff Shop and a greyed out Current Location. We click Lodestar, the top one.

use libscreenshot::ImageBuffer;

use crate::data::Point;

/// Where to look (800x600 client); the window sits near the top middle.
const AREA: (u32, u32, u32, u32) = (120, 30, 680, 420);
/// A row of a button: this many gold pixels spread over at least `MIN_SPAN`.
const MIN_GOLD: u32 = 110;
const MIN_SPAN: u32 = 160;

/// The gold of the buttons, e.g. (251, 198, 65), darker at the edges.
fn is_button_gold(p: &[u8]) -> bool {
    let (r, g, b) = (p[0] as i32, p[1] as i32, p[2] as i32);
    r >= 185 && (145..=230).contains(&g) && b <= 115 && r - b >= 110 && r > g
}

/// Center of the Lodestar button, if the Resurrection window is open.
pub fn find_lodestar(image: &ImageBuffer) -> Option<Point> {
    let (x0, y0) = (AREA.0.min(image.width()), AREA.1.min(image.height()));
    let (x1, y1) = (AREA.2.min(image.width()), AREA.3.min(image.height()));

    // Rows that look like part of a button: (y, first x, last x)
    let mut rows: Vec<(u32, u32, u32)> = vec![];
    for y in y0..y1 {
        let mut count = 0;
        let (mut first, mut last) = (u32::MAX, 0);
        for x in x0..x1 {
            if is_button_gold(&image.get_pixel(x, y).0) {
                count += 1;
                first = first.min(x);
                last = last.max(x);
            }
        }
        if count >= MIN_GOLD && last >= first && last - first >= MIN_SPAN {
            rows.push((y, first, last));
        }
    }

    // Group neighbouring rows into buttons
    let mut buttons: Vec<Vec<(u32, u32, u32)>> = vec![];
    for row in rows {
        match buttons.last_mut() {
            Some(b) if row.0 - b.last().unwrap().0 <= 2 => b.push(row),
            _ => buttons.push(vec![row]),
        }
    }
    buttons.retain(|b| b.len() >= 5);

    // At least two buttons stacked at the same x, so a random gold thing doesn't count
    let first = buttons.first()?;
    let left = |b: &Vec<(u32, u32, u32)>| b.iter().map(|r| r.1).min().unwrap_or(0);
    let right = |b: &Vec<(u32, u32, u32)>| b.iter().map(|r| r.2).max().unwrap_or(0);
    let stacked = buttons
        .iter()
        .skip(1)
        .any(|b| left(b).abs_diff(left(first)) <= 6 && right(b).abs_diff(right(first)) <= 6);
    if !stacked {
        return None;
    }
    let y = (first.first().unwrap().0 + first.last().unwrap().0) / 2;
    let x = (left(first) + right(first)) / 2;
    Some(Point::new(x, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn button(img: &mut ImageBuffer, x: u32, y: u32) {
        for yy in y..y + 20 {
            for xx in x..x + 205 {
                img.put_pixel(xx, yy, Rgba([251, 198, 65, 255]));
            }
        }
        // Black text in the middle
        for yy in y + 6..y + 14 {
            for xx in x + 70..x + 135 {
                if xx % 3 == 0 {
                    img.put_pixel(xx, yy, Rgba([0, 0, 0, 255]));
                }
            }
        }
    }

    #[test]
    fn finds_the_top_button() {
        let mut img = ImageBuffer::from_pixel(800, 600, Rgba([90, 90, 100, 255]));
        button(&mut img, 295, 105);
        button(&mut img, 295, 133);
        button(&mut img, 295, 161);
        let p = find_lodestar(&img).expect("not found");
        assert!((p.x as i32 - 397).abs() <= 2 && (p.y as i32 - 114).abs() <= 2, "{:?}", p);
    }

    #[test]
    fn nothing_without_the_window() {
        let mut img = ImageBuffer::from_pixel(800, 600, Rgba([90, 90, 100, 255]));
        // One lone gold bar (e.g. a quest window) isn't the Resurrection window
        button(&mut img, 295, 105);
        assert!(find_lodestar(&img).is_none());
    }

    /// NEUZ_REVIVE_IMG=shot.png cargo test revive -- --ignored --nocapture
    #[test]
    #[ignore]
    fn from_screenshot() {
        let path = std::env::var("NEUZ_REVIVE_IMG").expect("NEUZ_REVIVE_IMG");
        let img = image::open(&path).unwrap().to_rgba8();
        println!("lodestar: {:?}", find_lodestar(&img));
    }
}
