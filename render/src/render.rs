/// Handles layout and orchestrating the calls from the `draw` module

use crate::DisplayData;
use crate::draw::*;

use chrono::{Duration, Datelike};

use image::{RgbImage, Rgb};

use rusttype::Font;

use embedded_graphics::prelude::*;
use epd_waveshare::{
    color::*,
    epd7in5b_v2::{WIDTH as EPD_WIDTH, HEIGHT as EPD_HEIGHT},
    graphics::VarDisplay,
};
use epd_waveshare::buffer_len;

pub type EInkBuffer = Vec<u8>;

pub fn render(local_timezone: chrono_tz::Tz, display_data: DisplayData) -> (EInkBuffer, RgbImage) {
    let white = image::Rgb([255u8, 255u8, 255u8]);
    let black = image::Rgb([0u8, 0u8, 0u8]);
    let red = image::Rgb([255u8, 0u8, 0u8]);

    let graph_x = 50i64;
    let graph_y = 150i64;
    let graph_width = 700i64;
    let graph_height = 200i64;
    let graph_text_x = graph_x as f32 - 10.0;
    let graph_text_y = graph_y as f32;

    let font_data: &[u8] = include_bytes!("../fonts/Comfortaa-Regular.ttf");
    let font: Font<'static> = Font::try_from_bytes(font_data)
        .expect("failed to open font");


    let current_weather = display_data.current_weather;
    let forecast = display_data.forecast;
    let todoist_tasks = display_data.todoist_tasks;

    let fiveday = draw_5day_graph(&forecast, graph_width, graph_height, &font);
    let (min_temp, max_temp) = forecast.week_minmax_temps();
    let daily_temps = forecast.daily_minmax_temps();

    let mut image = RgbImage::from_fn(800, 480, |_, _| -> Rgb<u8> { Rgb([255u8, 255u8, 255u8]) });


    let temp_x = 10.0;
    let temp_y = 0.0;
    let temp_size = 150.0;
    let temp_text = format!("{}°", current_weather.temp_f);

    let current_time = chrono::Utc::now().with_timezone(&local_timezone);
    let (today_low, today_high) = daily_temps[&current_time.day()];
    let today_temps_text = format!("{}° {}°", today_high, today_low);
    let time_text = format!("{}", current_time.format("%-m/%-d  %-I%P"));

    let (temp_width, temp_height) = measure_text(&font, &temp_text, temp_size);
    let desc_x = temp_x + temp_width + 10.0;
    let desc_y = temp_y + temp_height/2.0;

    let today_temps_x = temp_x + temp_width + 20.0;
    let today_temps_y = 10.0;

    let mintext = min_temp.to_string();
    let maxtext = max_temp.to_string();

    draw_text_left(&mut image, &temp_text, temp_x, temp_y, &font, temp_size);
    draw_text_left(&mut image, &current_weather.description, desc_x, desc_y, &font, 50.0);
    draw_text_left(&mut image, &today_temps_text, today_temps_x, today_temps_y, &font, 36.0);
    draw_text_right(&mut image, &time_text, 790.0, 10.0, &font, 36.0, black);
    draw_text_right(&mut image, &maxtext, graph_text_x, graph_text_y, &font, 24.0, red);
    draw_text_bottom_right(&mut image, &mintext, graph_text_x, graph_text_y+graph_height as f32, &font, 24.0, red);


    let mut task_y = (graph_y + graph_height + 20) as f32;
    let task_x = 50.0;
    let current_date = current_time.date_naive();
    for task in todoist_tasks {
        let date_desc: String;
        if task.due_date < current_date {
            date_desc = "yesterday".into();
        }
        else if task.due_date == current_date {
            date_desc = "today".into();
        }
        else if task.due_date == current_date + Duration::days(1) {
            date_desc = "tomorrow".into();
        }
        else {
            date_desc = task.due_date.format("%-m/%-d").to_string();
        }

        draw_text_left(&mut image, &date_desc, task_x, task_y, &font, 24.0);
        draw_text_left(&mut image, &task.description, task_x+150.0, task_y, &font, 24.0);
        task_y += 30.0;

        if task_y >= 480.0 {
            break;
        }
    }

    image::imageops::overlay(&mut image, &fiveday, graph_x, graph_y);


    let mut buffer = vec![TriColor::White.get_byte_value(); buffer_len(EPD_WIDTH as usize, 2 * EPD_HEIGHT as usize)];
    let mut display = VarDisplay::<TriColor>::new(EPD_WIDTH, EPD_HEIGHT, &mut buffer, false).expect("failed to create display");

    for (x, y, p) in image.enumerate_pixels() {
        let x = x as i32;
        let y = y as i32;
        let pt = Point::new(x,y);
        if *p == white {
            display.set_pixel(Pixel(pt, TriColor::White));
        }
        else if *p == black {
            display.set_pixel(Pixel(pt, TriColor::Black));
        }
        else if *p == red {
            display.set_pixel(Pixel(pt, TriColor::Chromatic));
        }
        else {
            display.set_pixel(Pixel(pt, TriColor::White));
        }
    }

    (buffer, image)
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn read_image_data(bytes: &[u8]) -> image::RgbImage {
        let decoder = image::codecs::png::PngDecoder::new(bytes).unwrap();

        image::DynamicImage::from_decoder(decoder).unwrap()
            .as_rgb8().unwrap()
            .clone()
    }

    #[test]
    fn test_render1() {
        let gold_master = include_bytes!("../tests/render_test1.png");

        let data = get_test_data();
        let tz = "America/New_York".parse().unwrap();
        let (buffer, image) = render(tz, data);

        // code to generate the initial file
        // use std::path::Path;
        // let mut output_image_file = File::create(Path::new("tests/render_test1.png")).expect("failed to create file");
        // image.write_to(&mut output_image_file, image::ImageOutputFormat::Png);

        assert_eq!(buffer.len(), 96000);
        assert_eq!(image, read_image_data(gold_master));

        let white = image::Rgb([255u8, 255u8, 255u8]);
        let black = image::Rgb([0u8, 0u8, 0u8]);
        let red = image::Rgb([255u8, 0u8, 0u8]);
        let colors = [white, black, red];
        for (x, y, p) in image.enumerate_pixels() {
            assert!(colors.contains(p), "color at {x} {y} did not match: {p:#?}");
        }
    }
}
