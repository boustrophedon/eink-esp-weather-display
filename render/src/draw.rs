/// Handles drawing the actual pixels onto a canvas
/// See the `render` module for where these are called from

use std::collections::HashMap;

use chrono::prelude::*;

use imageproc::drawing::{Canvas, draw_text_mut, draw_line_segment_mut, BresenhamLineIter};
use image::{RgbImage, Rgb, Pixel};
use image::imageops::ColorMap;
use rusttype::{point, Font, Scale};

pub struct TriColorDither;

impl ColorMap for TriColorDither {
    type Color = Rgb<u8>;

    #[inline(always)]
    fn index_of(&self, color: &Rgb<u8>) -> usize {
        let l = color.to_luma().0[0];
        let c = color.0;
        // check for red
        if c[0] > 250 && c[1] < 210 && c[2] < 210 {
            return 2;
        }

        if l < 250 {
            return 0;
        }
        else {
            return 1;
        }
    }

    #[inline(always)]
    fn lookup(&self, idx: usize) -> Option<Self::Color> {
        let white = Rgb([255u8, 255u8, 255u8]);
        let black = Rgb([0u8, 0u8, 0u8]);
        let red = Rgb([255u8, 0u8, 0u8]);
        match idx {
            0 => Some(black),
            1 => Some(white),
            2 => Some(red),
            _ => Some(white),
        }
    }

    fn has_lookup(&self) -> bool {
        true
    }

    #[inline(always)]
    fn map_color(&self, color: &mut Rgb<u8>) {
        *color = self.lookup(self.index_of(color)).expect("color not found");
    }
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

pub fn draw_5day_graph(forecast: &[(DateTime<FixedOffset>, i32, u64)], width: i64, height: i64, font: &Font) -> RgbImage {
    let mut image = RgbImage::from_fn(width as u32, height as u32, |_, _| -> Rgb<u8> { Rgb([255u8, 255u8, 255u8]) });
    let black = image::Rgb([0u8, 0u8, 0u8]);
    let red = image::Rgb([255u8, 0u8, 0u8]);

    let height = height as f32;
    let width = width as f32;
    let horiz_spacing = (width / (forecast.len()-1) as f32).floor();
  
    // gets daily min and max temps
    // note that it does not get the temps of the very last day
    // because otherwise i'd have to calculate if there's enough space to draw it remaining in the
    // image which would be annoying but possible to do with measure_text

    let mut daily_minmax: HashMap<u32, (i32, i32)> = HashMap::new();
    let mut day_min: i32 = 200;
    let mut day_max: i32 = -100;
    let mut current_day: u32 = forecast[0].0.day();
    for (d, t, _) in forecast {
        let day = d.day();
        let t = *t;
        if day != current_day {
            daily_minmax.insert(current_day, (day_min, day_max));
            day_min = 200;
            day_max = -100;
            current_day = day;
        }
        if t < day_min && d.hour() > 12 {
            day_min = t;
        }
        else if t > day_max {
            day_max = t;
        }
    }
   
    // convert rain probabilities 0-100 into pixel heights
    // convert temp into pixel heights based on max and min temps
    // y axis points down so we subtract from height

    let min_temp = 0.9 * forecast.iter()
                .min_by_key(|(_, t, _)| t).expect("no values in forecast").1 as f32;
    let max_temp = 1.1 * forecast.iter()
                .max_by_key(|(_, t, _)| t).expect("no values in forecast").1 as f32;
    let mut rain_points = Vec::new();
    let mut temp_points = Vec::new();
    let mut x = 0f32;

    // compute the pixel y coordinate for each graph's data points based on the total height of the graph region
    // the points are computed by scaling each point relative to the overall height, and for the
    // temperature graph, the min and max temperature. the rain precipitation is a % so we just
    // scale from 0 to 100. there's a bit of tricky off by one stuff as well.
    for (d, temp, rain_p) in forecast {
        let rain_y: f32 = height - ((*rain_p as f32 / 100.0)*(height-1.0)).floor() - 1.0;

        let temp_y: f32 = height - ((*temp as f32 - min_temp)/ (max_temp - min_temp))*(height-1.0).floor() - 1.0;
        rain_points.push((d, (x, rain_y)));
        temp_points.push((x, temp_y));
        x+=horiz_spacing;
    }

    let lastday = rain_points.iter().last().unwrap().0.day();
    // draw vertical lines with legends to split into days of the week
    for w in rain_points.windows(2) {
        let d1 = w[0].0;
        let d2 = w[1].0;
        if d1.day() != d2.day() {
            let x = w[1].1.0;
            draw_line_segment_mut(&mut image, (x, 0.0), (x, height), black);
            let day = d2.day();
            let day_letter = &d2.weekday().to_string()[..1];
            let text: String;
            if day != lastday {
                let (day_min, day_max) = daily_minmax.get(&day).expect(&format!("daily minmax not found for day {day}"));
                text = format!("{day_letter} {day_max} {day_min}");
            }
            else {
                text = day_letter.to_string();
            }
            draw_text_left(&mut image, &text, x + 5.0, 0.0, &font, 36.0);
        }
    }

    // draw the actual graph, iterating over the pixels directly so that we can keep track of the
    // heights of the graph for shading in under the graph later.
    let mut max_y = vec![None; width as usize];
    for w in rain_points.windows(2) {
        let p1 = w[0].1;
        let p2 = w[1].1;
        for (x, y) in BresenhamLineIter::new(p1, p2) {
            let x = x as usize;
            let y = y as u32;
            
            if max_y[x].is_none() || max_y[x].filter(|ym| y > *ym).is_some() {
                max_y[x] = Some(y);
            }
            image.draw_pixel(x as u32, y, Rgb([0u8,0u8,0u8]));
        }
    }

    // using some math that took me a bit to get correct, draw a nice little shading pattern under
    // the graph, using the above max_y to determine where to shade
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

    for w in temp_points.windows(2) {
        let p1 = w[0];
        let p2 = w[1];
        draw_line_segment_mut(&mut image, (p1.0, p1.1 + 1.0), (p2.0, p2.1 + 1.0), red);
        draw_line_segment_mut(&mut image, p1, p2, red);
        draw_line_segment_mut(&mut image, (p1.0, p1.1 - 1.0), (p2.0, p2.1 - 1.0), red);
    }

    //let max_text = (max_temp as i32).to_string();
    //let min_text = (min_temp as i32).to_string();
    //draw_text_left_color(&mut image, &max_text, 0.0, 0.0, &font, 24.0, red);
    //draw_text_bottom(&mut image, &min_text, 0.0, height, &font, 24.0, red);

    image
}

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
