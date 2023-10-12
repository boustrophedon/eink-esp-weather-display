use imageproc::drawing::Canvas;

use image::{RgbImage, Rgb};

use rusttype::{point, Font, PositionedGlyph, Rect, Scale};
use std::cmp::max;

// Code mostly taken wholesale from 
// https://github.com/image-rs/imageproc/blob/master/src/drawing/text.rs

fn layout_glyphs(
    scale: Scale,
    font: &Font,
    text: &str,
    mut f: impl FnMut(PositionedGlyph, Rect<i32>),
) -> (i32, i32) {
    let v_metrics = font.v_metrics(scale);

    let (mut w, mut h) = (0, 0);

    for g in font.layout(text, scale, point(0.0, v_metrics.ascent)) {
        if let Some(bb) = g.pixel_bounding_box() {
            w = max(w, bb.max.x);
            h = max(h, bb.max.y);
            f(g, bb);
        }
    }

    (w, h)
}

/// Draws colored text on an image in place.
///
/// `scale` is augmented font scaling on both the x and y axis (in pixels).
///
/// Note that this function *does not* support newlines, you must do this manually.
pub fn draw_text_mut<'a>(
    canvas: &'a mut RgbImage,
    color: Rgb<u8>,
    x: i32,
    y: i32,
    scale: Scale,
    font: &'a Font<'a>,
    text: &'a str,
) where
{
    let image_width = canvas.width() as i32;
    let image_height = canvas.height() as i32;

    layout_glyphs(scale, font, text, |g, bb| {
        g.draw(|gx, gy, gv| {
            let gx = gx as i32 + bb.min.x;
            let gy = gy as i32 + bb.min.y;

            let image_x = gx + x;
            let image_y = gy + y;

            if (0..image_width).contains(&image_x) && (0..image_height).contains(&image_y) {

                // code edited here from original, if there's any coverage just make it uniformly
                // the same color, else don't draw
                if gv > 0.1 {
                    canvas.draw_pixel(image_x as u32, image_y as u32, color);
                }
            }
        })
    });
}
