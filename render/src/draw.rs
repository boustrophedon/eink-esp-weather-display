/// Handles drawing the actual pixels onto a canvas
/// See the `render` module for where these are called from

use crate::Forecast5Day;

use chrono::prelude::*;

use crate::text::draw_text_mut;
use imageproc::drawing::{Canvas, draw_line_segment_mut, BresenhamLineIter};
use image::{RgbImage, Rgb};
use rusttype::{point, Font, Scale};


pub fn draw_5day_graph(forecast: &Forecast5Day,
        width: i64, height: i64, font: &Font) -> RgbImage {
    let mut image = RgbImage::from_fn(width as u32, height as u32, |_, _| -> Rgb<u8> { Rgb([255u8, 255u8, 255u8]) });
    let black = image::Rgb([0u8, 0u8, 0u8]);
    let red = image::Rgb([255u8, 0u8, 0u8]);

    let height = height as f32;
    let width = width as f32;

    let forecast_data = forecast.filtered_forecast();
    let horiz_spacing = (width-1.0) / (forecast_data.len()-1) as f32;


    let daily_minmax = forecast.daily_minmax_temps();
    let (_min_temp, _max_temp) = forecast.week_minmax_temps();
    // scale the temp values so that the temperature graph doesn't go to right to the border
    let min_temp_scale = 0.9 * _min_temp as f32;
    let max_temp_scale = 1.1 * _max_temp as f32;
    // convert rain probabilities 0-100 into pixel heights
    // convert temp into pixel heights based on max and min temps
    // y axis points down so we subtract from height

    // compute the pixel y coordinate for each graph's data points based on the total height of the graph region
    // the points are computed by scaling each point relative to the overall height, and for the
    // temperature graph, the min and max temperature. the rain precipitation is a % so we just
    // scale from 0 to 100. there's a bit of tricky off by one stuff as well.
    let temp_x = (0..forecast_data.len()).map(|x| horiz_spacing * x as f32);
    let temp_y = forecast_data.iter().map(|(_d, temp, _r)| {
        height - ((*temp as f32 - min_temp_scale)/ (max_temp_scale - min_temp_scale))*(height-1.0).floor() - 1.0
    });

    let rain_x = (0..forecast_data.len()).map(|x| horiz_spacing * x as f32);
    let rain_y = forecast_data.iter().map(|(_d, _t, rain_p)| {
        height - ((*rain_p as f32 / 100.0)*(height-1.0)).floor() - 1.0
    });

    let dates = forecast_data.iter().map(|(d, _t, _r)| d.clone());

    let temp_points: Vec<(f32, f32)> = temp_x.zip(temp_y).collect();
    let rain_points: Vec<(DateTime<_>, (f32, f32))> =
        dates.zip(rain_x.zip(rain_y)).collect();

    let lastday = rain_points.iter().last().unwrap().0.day();
    // draw vertical lines with legends to split into days of the week
    for w in rain_points.windows(2) {
        let d1 = w[0].0;
        let d2 = w[1].0;
        if d1.day() != d2.day() {
            let x = w[1].1.0;
            let day = d2.day();
            let day_letter = &d2.weekday().to_string()[..1];
            draw_line_segment_mut(&mut image, (x, 0.0), (x, height), black);
            if day != lastday {
                let (day_min, day_max) = daily_minmax.get(&day).expect(&format!("daily minmax not found for day {day}"));
                let text = format!("{day_letter} {day_max} {day_min}");
                draw_text_left(&mut image, &text, x + 5.0, 0.0, &font, 36.0);
            }
        }
    }

    // draw the actual graph, iterating over the pixels directly so that we can keep track of the
    // heights of the graph for shading in under the graph later.
    let mut max_y = vec![None; width as usize];
    for w in rain_points.windows(2) {
        let p1 = w[0].1;
        let p2 = w[1].1;
        for (x, y) in BresenhamLineIter::new(p1, p2) {
            let x = x.max(0).min((width - 1.0) as i32) as usize;
            let y = y.max(0).min((height - 1.0) as i32) as u32;

            if max_y[x].is_none() || max_y[x].filter(|ym| y > *ym).is_some() {
                max_y[x] = Some(y);
            }
            image.draw_pixel(x as u32, y, Rgb([0u8,0u8,0u8]));
        }
    }

    // using some math that took me a bit to get correct, draw a nice little shading pattern under
    // the graph, using the above max_y to determine the limits of the pattern
    for (x, y, p) in image.enumerate_pixels_mut() {
        let ym = y%6;
        let yd = y/6;
        let xm = (x+2*yd)%6;
        let xcond = (xm == ym) || (xm == ym+1);
        let ycond = ym < 3;

        let g_cond = max_y[x as usize].filter(|ym| y > *ym).is_some();

        if xcond && ycond && g_cond {
            *p = Rgb([0u8,0u8,0u8]);
        }
    }

    // Draw the actual temperature graph last so it goes on top of everything
    for w in temp_points.windows(2) {
        let p1 = w[0];
        let p2 = w[1];
        draw_line_segment_mut(&mut image, (p1.0, p1.1 + 1.0), (p2.0, p2.1 + 1.0), red);
        draw_line_segment_mut(&mut image, p1, p2, red);
        draw_line_segment_mut(&mut image, (p1.0, p1.1 - 1.0), (p2.0, p2.1 - 1.0), red);
    }

    image
}

pub fn measure_text(font: &Font, text: &str, font_size: f32) -> (f32, f32) {
    let font_size = Scale::uniform(font_size);
    let v_metrics = font.v_metrics(font_size);

    let xpad = 0f32;
    let ypad = 0f32;

    let glyphs: Vec<_> = font
        .layout(text, font_size, point(xpad, ypad + v_metrics.ascent))
        .collect();

    let height = (v_metrics.ascent - v_metrics.descent).ceil();
    let width = {
            let min_x = glyphs
                    .first()
                    .map(|g| g.pixel_bounding_box().unwrap().min.x)
                    .unwrap();
            let max_x = glyphs
                    .last()
                    .map(|g| g.pixel_bounding_box().unwrap().max.x)
                    .unwrap();
            (max_x - min_x) as f32
    };

    (width, height)
}

// TODO: figure out a way to reduce duplication of all these functions here? I guess just pass an
// align enum to a single draw_text method or something

pub fn draw_text_centered(image: &mut RgbImage, text: &str, x: f32, y: f32, font: &Font, scale: f32) {
    let black = image::Rgb([0u8, 0u8, 0u8]);
    let (text_width, text_height) = measure_text(&font, &text, scale);
    let scale = Scale::uniform(scale);
    let text_x = (x - text_width/2.0).ceil() as i32;
    let text_y = (y - text_height/2.0).ceil() as i32;

    draw_text_mut(image, black, text_x, text_y, scale, &font, &text);
}

pub fn draw_text_left(image: &mut RgbImage, text: &str, x: f32, y: f32, font: &Font, scale: f32) {
    let black = image::Rgb([0u8, 0u8, 0u8]);
    let scale = Scale::uniform(scale);

    draw_text_mut(image, black, x as i32, y as i32, scale, &font, &text);
}

pub fn draw_text_right(image: &mut RgbImage, text: &str, x: f32, y: f32, font: &Font, scale: f32, color: Rgb<u8>) {
    let (text_width, _text_height) = measure_text(&font, &text, scale);
    let scale = Scale::uniform(scale);
    let text_x = x - text_width;

    draw_text_mut(image, color, text_x as i32, y as i32, scale, &font, &text);
}

pub fn draw_text_left_color(image: &mut RgbImage, text: &str, x: f32, y: f32, font: &Font, scale: f32, color: Rgb<u8>) {
    let scale = Scale::uniform(scale);

    draw_text_mut(image, color, x as i32, y as i32, scale, &font, &text);
}


pub fn draw_text_bottom(image: &mut RgbImage, text: &str, x: f32, y: f32, font: &Font, scale: f32, color: Rgb<u8>) {
    let (_text_width, text_height) = measure_text(&font, &text, scale);
    let scale = Scale::uniform(scale);

    draw_text_mut(image, color, x as i32, (y-text_height) as i32, scale, &font, &text);
}

pub fn draw_text_bottom_right(image: &mut RgbImage, text: &str, x: f32, y: f32, font: &Font, scale: f32, color: Rgb<u8>) {
    let (text_width, text_height) = measure_text(&font, &text, scale);
    let scale = Scale::uniform(scale);

    draw_text_mut(image, color, (x-text_width) as i32, (y-text_height) as i32, scale, &font, &text);
}
