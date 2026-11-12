//! Reads monster name tags from the game screen.
//!
//! The name tag is cut out of the screenshot, turned into clean black text on a white
//! background (only pixels matching the name color are kept), enlarged, and passed to
//! the OCR engine built into Windows 10/11 (`Windows.Media.Ocr`). Nothing has to be
//! installed. On other platforms reading names is not supported and returns `None`.

use libscreenshot::ImageBuffer;

use crate::data::Bounds;

/// How much to enlarge the name tag before OCR. Name tags are 8-9px tall; OCR
/// engines want text around 30px tall.
const SCALE: u32 = 4;
/// White border added around the text, in output pixels.
const PADDING: u32 = 12;
/// Extra screen pixels taken around the detected name bounds. The bounds only
/// cover the brightest pixels, so the outer edges of letters can fall outside.
const MARGIN_X: u32 = 4;
const MARGIN_Y: u32 = 3;
/// Pixels farther than this (per channel, beyond the "full ink" distance) from the
/// name color are background.
const FADE: u32 = 50;

/// A black-on-white, enlarged copy of the name tag in BGRA8 layout.
pub struct PreparedText {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

/// How dark a pixel should be in the cleaned-up text: 0 (ink) when it is close
/// to the name color, fading to 255 (paper) as it gets farther away. Keeping the
/// soft edges, instead of a hard yes/no, is what makes the tiny game font readable.
fn ink_level(px: &[u8], color: &[u8; 3], full_ink: u32) -> u8 {
    let dist = (0..3).map(|i| px[i].abs_diff(color[i]) as u32).max().unwrap_or(255);
    if dist <= full_ink {
        0
    } else if dist >= full_ink + FADE {
        255
    } else {
        ((dist - full_ink) * 255 / FADE) as u8
    }
}

/// Cut out the name tag at `bounds`, keep only pixels close to `color` (with soft
/// edges), and enlarge it smoothly.
pub fn prepare(image: &ImageBuffer, bounds: Bounds, color: [u8; 3], tolerance: u8) -> Option<PreparedText> {
    use image::{imageops::FilterType, GrayImage, Luma};

    let x0 = bounds.x.saturating_sub(MARGIN_X);
    let y0 = bounds.y.saturating_sub(MARGIN_Y);
    let x1 = (bounds.x + bounds.w + MARGIN_X).min(image.width());
    let y1 = (bounds.y + bounds.h + MARGIN_Y).min(image.height());
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let (src_w, src_h) = (x1 - x0, y1 - y0);
    let full_ink = (tolerance as u32 * 2).max(20);

    let mut gray = GrayImage::new(src_w, src_h);
    let mut ink = 0;
    for sy in 0..src_h {
        for sx in 0..src_w {
            let level = ink_level(&image.get_pixel(x0 + sx, y0 + sy).0, &color, full_ink);
            if level < 128 {
                ink += 1;
            }
            gray.put_pixel(sx, sy, Luma([level]));
        }
    }
    if ink < 5 {
        return None;
    }

    let scaled = image::imageops::resize(&gray, src_w * SCALE, src_h * SCALE, FilterType::Lanczos3);
    let width = scaled.width() + 2 * PADDING;
    let height = scaled.height() + 2 * PADDING;
    let mut bgra = vec![255u8; (width * height * 4) as usize];
    for (sx, sy, px) in scaled.enumerate_pixels() {
        let i = (((sy + PADDING) * width + sx + PADDING) * 4) as usize;
        bgra[i] = px.0[0];
        bgra[i + 1] = px.0[0];
        bgra[i + 2] = px.0[0];
    }
    Some(PreparedText { width, height, bgra })
}

/// Read the text of a name tag. Returns `None` if nothing readable was found.
pub fn read_name(image: &ImageBuffer, bounds: Bounds, color: [u8; 3], tolerance: u8) -> Option<String> {
    let prepared = prepare(image, bounds, color, tolerance)?;
    let text = platform::recognize(&prepared)?;
    let text = text.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(windows)]
mod platform {
    use super::PreparedText;
    use windows::{
        core::HSTRING,
        Globalization::Language,
        Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap},
        Media::Ocr::OcrEngine,
        Storage::Streams::DataWriter,
    };

    thread_local! {
        static ENGINE: Option<OcrEngine> = create_engine();
    }

    fn create_engine() -> Option<OcrEngine> {
        // Monster names are English; fall back to the user's languages if
        // the English OCR pack isn't installed.
        Language::CreateLanguage(&HSTRING::from("en-US"))
            .ok()
            .and_then(|lang| OcrEngine::TryCreateFromLanguage(&lang).ok())
            .or_else(|| OcrEngine::TryCreateFromUserProfileLanguages().ok())
    }

    pub fn recognize(text: &PreparedText) -> Option<String> {
        ENGINE.with(|engine| {
            let engine = engine.as_ref()?;
            let max = OcrEngine::MaxImageDimension().ok()?;
            if text.width > max || text.height > max {
                return None;
            }
            let writer = DataWriter::new().ok()?;
            writer.WriteBytes(&text.bgra).ok()?;
            let buffer = writer.DetachBuffer().ok()?;
            let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
                &buffer,
                BitmapPixelFormat::Bgra8,
                text.width as i32,
                text.height as i32,
            )
            .ok()?;
            let result = engine.RecognizeAsync(&bitmap).ok()?.get().ok()?;
            Some(result.Text().ok()?.to_string())
        })
    }
}

#[cfg(not(windows))]
mod platform {
    use super::PreparedText;

    pub fn recognize(_text: &PreparedText) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn prepare_keeps_only_name_color() {
        let mut img = ImageBuffer::from_pixel(30, 16, Rgba([10, 10, 10, 255]));
        // A 6x3 block of "name colored" pixels
        for x in 10..16 {
            for y in 6..9 {
                img.put_pixel(x, y, Rgba([234, 234, 149, 255]));
            }
        }
        let bounds = Bounds::new(10, 6, 6, 3);
        let out = prepare(&img, bounds, [234, 234, 149], 5).unwrap();
        assert_eq!(out.width, (6 + 2 * MARGIN_X) * SCALE + 2 * PADDING);
        assert_eq!(out.height, (3 + 2 * MARGIN_Y) * SCALE + 2 * PADDING);
        // The middle of the block is ink, the corners are paper.
        let at = |x: u32, y: u32| out.bgra[((y * out.width + x) * 4) as usize];
        let cx = PADDING + (MARGIN_X + 3) * SCALE;
        let cy = PADDING + (MARGIN_Y + 1) * SCALE + SCALE / 2;
        assert!(at(cx, cy) < 30, "center should be dark, got {}", at(cx, cy));
        assert_eq!(at(PADDING, PADDING), 255);
    }

    #[test]
    fn ink_level_fades_with_distance() {
        let c = [234, 234, 149];
        assert_eq!(ink_level(&[234, 234, 149, 255], &c, 20), 0);
        assert_eq!(ink_level(&[214, 214, 149, 255], &c, 20), 0);
        let mid = ink_level(&[189, 189, 149, 255], &c, 20);
        assert!(mid > 0 && mid < 255);
        assert_eq!(ink_level(&[10, 10, 10, 255], &c, 20), 255);
    }

    /// Writes the prepared name tag as a PNG, to check it with an OCR tool:
    /// NEUZ_OCR_IMG=shot.png NEUZ_OCR_BOUNDS=x,y,w,h cargo test dump_prepared -- --ignored
    #[test]
    #[ignore]
    fn dump_prepared() {
        let path = std::env::var("NEUZ_OCR_IMG").expect("NEUZ_OCR_IMG");
        let b: Vec<u32> = std::env::var("NEUZ_OCR_BOUNDS").expect("NEUZ_OCR_BOUNDS")
            .split(',').map(|v| v.trim().parse().unwrap()).collect();
        let img = image::open(&path).unwrap().to_rgba8();
        let out = prepare(&img, Bounds::new(b[0], b[1], b[2], b[3]), [234, 234, 149], 5).unwrap();
        let gray = image::GrayImage::from_fn(out.width, out.height, |x, y| {
            image::Luma([out.bgra[((y * out.width + x) * 4) as usize]])
        });
        let dest = std::env::var("NEUZ_OCR_OUT").unwrap_or_else(|_| "prepared.png".into());
        gray.save(dest).unwrap();
    }

    #[test]
    fn prepare_rejects_empty_area() {
        let img = ImageBuffer::from_pixel(20, 10, Rgba([10, 10, 10, 255]));
        assert!(prepare(&img, Bounds::new(2, 2, 5, 3), [234, 234, 149], 5).is_none());
    }
}
