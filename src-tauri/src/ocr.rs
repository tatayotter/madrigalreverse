//! Reads monster name tags from the game screen.
//!
//! The name tag is cut out of the screenshot, turned into clean black text on a white
//! background (only pixels matching the name color are kept), enlarged, and passed to
//! the OCR engine built into Windows 10/11 (`Windows.Media.Ocr`). Nothing has to be
//! installed. On other platforms reading names is not supported and returns `None`.

use libscreenshot::ImageBuffer;

use crate::data::Bounds;

/// How much to enlarge the name tag before OCR. Windows OCR works best on text
/// that is at least ~20px tall; name tags are around 10px.
const SCALE: u32 = 3;
/// White border added around the text, in output pixels.
const PADDING: u32 = 12;
/// Extra screen pixels taken around the detected name bounds.
const MARGIN: u32 = 2;

/// A black-on-white, enlarged copy of the name tag in BGRA8 layout.
pub struct PreparedText {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

fn matches(px: &[u8], color: &[u8; 3], tolerance: u8) -> bool {
    px[0].abs_diff(color[0]) <= tolerance
        && px[1].abs_diff(color[1]) <= tolerance
        && px[2].abs_diff(color[2]) <= tolerance
}

/// Cut out the name tag at `bounds` and keep only pixels close to `color`.
pub fn prepare(image: &ImageBuffer, bounds: Bounds, color: [u8; 3], tolerance: u8) -> Option<PreparedText> {
    let x0 = bounds.x.saturating_sub(MARGIN);
    let y0 = bounds.y.saturating_sub(MARGIN);
    let x1 = (bounds.x + bounds.w + MARGIN).min(image.width());
    let y1 = (bounds.y + bounds.h + MARGIN).min(image.height());
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let (src_w, src_h) = (x1 - x0, y1 - y0);
    let width = src_w * SCALE + 2 * PADDING;
    let height = src_h * SCALE + 2 * PADDING;

    // Start all white, then paint matching pixels black.
    let mut bgra = vec![255u8; (width * height * 4) as usize];
    let mut ink = 0;
    for sy in 0..src_h {
        for sx in 0..src_w {
            let px = image.get_pixel(x0 + sx, y0 + sy).0;
            // Slightly looser than detection, so letter edges are kept.
            if !matches(&px, &color, tolerance.saturating_mul(2).max(20)) {
                continue;
            }
            ink += 1;
            for dy in 0..SCALE {
                for dx in 0..SCALE {
                    let ox = PADDING + sx * SCALE + dx;
                    let oy = PADDING + sy * SCALE + dy;
                    let i = ((oy * width + ox) * 4) as usize;
                    bgra[i] = 0;
                    bgra[i + 1] = 0;
                    bgra[i + 2] = 0;
                }
            }
        }
    }
    if ink < 5 {
        return None;
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
        let mut img = ImageBuffer::from_pixel(20, 10, Rgba([10, 10, 10, 255]));
        // A 6x3 block of "name colored" pixels
        for x in 5..11 {
            for y in 4..7 {
                img.put_pixel(x, y, Rgba([234, 234, 149, 255]));
            }
        }
        let bounds = Bounds::new(5, 4, 6, 3);
        let out = prepare(&img, bounds, [234, 234, 149], 5).unwrap();
        assert_eq!(out.width, (6 + 2 * MARGIN) * SCALE + 2 * PADDING);
        let black = out.bgra.chunks(4).filter(|p| p[0] == 0).count() as u32;
        assert_eq!(black, 6 * 3 * SCALE * SCALE);
    }

    #[test]
    fn prepare_rejects_empty_area() {
        let img = ImageBuffer::from_pixel(20, 10, Rgba([10, 10, 10, 255]));
        assert!(prepare(&img, Bounds::new(2, 2, 5, 3), [234, 234, 149], 5).is_none());
    }
}
