use std::fs::File;
use std::io::prelude::*;

use std::path::PathBuf;

use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;

use chrono::{DateTime, Duration};

use image::{RgbImage, Rgb};
use rusttype::Font;

use embedded_graphics::prelude::*;
use epd_waveshare::{
    color::*,
    epd7in5b_v2::{WIDTH as EPD_WIDTH, HEIGHT as EPD_HEIGHT},
    graphics::VarDisplay,
};
use epd_waveshare::buffer_len;

mod env_data;
pub use env_data::*;

mod weather;
pub use weather::*;

mod tasks;
pub use tasks::*;

mod draw;
pub use draw::*;


pub fn get_env_data() -> EnvData {
    let file_path = get_input_path();
    EnvData::from_file(&file_path)
}

pub fn get_input_path() -> PathBuf {
    let path_str = std::env::args().nth(1)
        .expect("1st cli argument did not exist");
    PathBuf::from(path_str) 
}

pub fn get_output_path() -> PathBuf {
    let path_str = std::env::args().nth(2)
        .expect("2nd cli argument did not exist");
    PathBuf::from(path_str) 
}

struct DisplayData {
    current_weather: CurrentWeather,
    avg_forecast: AvgForecast,
    todoist_tasks: Vec<Task>,
}

fn gather_data(env_data: &EnvData, tx: SyncSender<(String, String, String)>) {
    extrasafe::SafetyContext::new()
        // Allow reading for DNS/SSL certificates
        .enable(
            extrasafe::builtins::SystemIO::nothing()
            .allow_open_readonly()
            .allow_read()
            .allow_close()
            .allow_metadata()
            ).unwrap()
        // Allow opening tcp sockets for http requests
        // Allow opening udp socket for DNS unfortunately
        .enable(
            extrasafe::builtins::Networking::nothing()
            .allow_start_tcp_clients()
            .allow_start_udp_servers().yes_really()
            ).unwrap()
        // Enable threading for reqwest blocking mode
        .enable(
            extrasafe::builtins::danger_zone::Threads::nothing()
            .allow_create()
            ).unwrap()
        .apply_to_current_thread()
        .unwrap();
    let client = create_weather_client(&env_data);
    //let daily_forecast = get_daily_forecast(&env_data, &client);
    //println!("{daily_forecast:#?}");

    let todoist_client = create_todoist_client(&env_data);
    let tasks_json = get_tasks(&todoist_client);

    let current_weather_json = get_current_weather(&env_data, &client);
    let hourly_forecast_json = get_hourly_forecast(&env_data, &client);

    tx.send((current_weather_json, hourly_forecast_json, tasks_json)).unwrap();
}

fn parse_data(tx: SyncSender<DisplayData>, current_weather_json: String, hourly_forecast_json: String, tasks_json: String) {
    // start a new context for parsing the json
    extrasafe::SafetyContext::new()
        .enable(
            extrasafe::builtins::SystemIO::nothing()
            .allow_stdout()
            .allow_stderr()
            ).unwrap()
        .apply_to_current_thread()
        .unwrap();
    let todoist_tasks = parse_tasks(&tasks_json);
    let current_weather = parse_current_weather(&current_weather_json);
    let forecast = parse_hourly_forecast(&hourly_forecast_json);
    let avg_forecast = gather_5day_forecast(&forecast);

    let data = DisplayData {
        current_weather,
        avg_forecast,
        todoist_tasks,
    };
    tx.send(data).unwrap();
}

fn main() {
    let env_data = get_env_data();
    let output_path = get_output_path();

    let font_data: &[u8] = include_bytes!("../fonts/Comfortaa-Regular.ttf");
    let font: Font<'static> = Font::try_from_bytes(font_data)
        .expect("failed to open font");

    let mut output_data_file = File::create(&output_path).expect("failed to create file");
    let mut output_image_file = File::create(&output_path.with_extension("png")).expect("failed to create file");

    let (json_sender, json_receiver) = sync_channel(1);
    let (data_sender, data_receiver) = sync_channel(1);

    let current_weather;
    let avg_forecast;
    let mut todoist_tasks;

    let use_debug_data = false;
    if !use_debug_data {
        let env_data = env_data.clone();
        thread::spawn(move || gather_data(&env_data, json_sender));
        let (current_weather_json, hourly_weather_json, tasks_json) = json_receiver.recv()
            .expect("failed to get json");

        thread::spawn(move || parse_data(data_sender, current_weather_json, hourly_weather_json, tasks_json));

        let data = data_receiver.recv()
            .expect("failed to get data");
        current_weather = data.current_weather;
        avg_forecast = data.avg_forecast;
        todoist_tasks = data.todoist_tasks;
    }
    else {
        current_weather = CurrentWeather {
            description: "test data".into(),
            temp_f: 69,
            rain_in: 0,
        };
        let avg_forecast_ = [("2023-05-17T21:00:00-04:00", 53, 0), ("2023-05-18T01:00:00-04:00", 47, 0), ("2023-05-18T05:00:00-04:00", 46, 0), ("2023-05-18T09:00:00-04:00", 54, 0), ("2023-05-18T13:00:00-04:00", 59, 0), ("2023-05-18T17:00:00-04:00", 57, 0), ("2023-05-18T21:00:00-04:00", 54, 0), ("2023-05-19T01:00:00-04:00", 51, 0), ("2023-05-19T05:00:00-04:00", 54, 0), ("2023-05-19T09:00:00-04:00", 62, 1), ("2023-05-19T13:00:00-04:00", 66, 4), ("2023-05-19T17:00:00-04:00", 64, 6), ("2023-05-19T21:00:00-04:00", 60, 12), ("2023-05-20T01:00:00-04:00", 59, 26), ("2023-05-20T05:00:00-04:00", 60, 39), ("2023-05-20T09:00:00-04:00", 64, 66), ("2023-05-20T13:00:00-04:00", 66, 63), ("2023-05-20T17:00:00-04:00", 67, 55), ("2023-05-20T21:00:00-04:00", 66, 36), ("2023-05-21T01:00:00-04:00", 61, 27), ("2023-05-21T05:00:00-04:00", 60, 21), ("2023-05-21T09:00:00-04:00", 67, 10), ("2023-05-21T13:00:00-04:00", 75, 6), ("2023-05-21T17:00:00-04:00", 74, 5), ("2023-05-21T21:00:00-04:00", 68, 4), ("2023-05-22T01:00:00-04:00", 62, 4), ("2023-05-22T05:00:00-04:00", 62, 4), ("2023-05-22T09:00:00-04:00", 69, 4), ("2023-05-22T13:00:00-04:00", 74, 6), ("2023-05-22T17:00:00-04:00", 70, 6)];
        avg_forecast = avg_forecast_.into_iter().map(|(dt, t, r)| (DateTime::parse_from_rfc3339(dt).unwrap(), t, r)).collect();

        let today = chrono::offset::Local::now().date_naive();
        let yesterday = today - Duration::days(1);
        let tomorrow = today + Duration::days(1);
        todoist_tasks = vec![Task { description: "test task".into(), due_date: today },
        Task { description: "task 2".into(), due_date: yesterday },
        Task { description: "task 3".into(), due_date: tomorrow }];
    }
    todoist_tasks.sort_by_key(|t| t.due_date);

    extrasafe::SafetyContext::new()
        .enable(
            extrasafe::builtins::SystemIO::nothing()
            .allow_stdout()
            .allow_stderr()
            .allow_file_write(&output_data_file)
            .allow_file_write(&output_image_file)
            .allow_close()
            ).unwrap()
        .apply_to_current_thread()
        .unwrap();
    let white = image::Rgb([255u8, 255u8, 255u8]);
    let black = image::Rgb([0u8, 0u8, 0u8]);
    let red = image::Rgb([255u8, 0u8, 0u8]);

    let graph_x = 50i64;
    let graph_y = 150i64;
    let graph_width = 700i64;
    let graph_height = 200i64;
    let graph_text_x = graph_x as f32 - 10.0;
    let graph_text_y = graph_y as f32;


    let fiveday = draw_5day_graph(&avg_forecast, graph_width, graph_height, &font);
    let min_temp = avg_forecast.iter()
        .min_by_key(|(_, t, _)| t).expect("no values in forecast")
        .1;
    let max_temp = avg_forecast.iter()
        .max_by_key(|(_, t, _)| t).expect("no values in forecast")
        .1;

    let mut image = RgbImage::from_fn(800, 480, |_, _| -> Rgb<u8> { Rgb([255u8, 255u8, 255u8]) });


    let temp_x = 10.0;
    let temp_y = 0.0;
    let temp_size = 150.0;
    let temp_text = format!("{}°", current_weather.temp_f);

    let current_time = chrono::Utc::now().with_timezone(&env_data.local_timezone);
    let time_text = format!("{}", current_time.format("%-m/%-d  %-I%P"));

    let (temp_width, temp_height) = measure_text(&font, &temp_text, temp_size);
    let desc_x = temp_x + temp_width + 10.0;
    let desc_y = temp_y + temp_height/2.0;


    let mintext = min_temp.to_string();
    let maxtext = max_temp.to_string();

    draw_text_left(&mut image, &temp_text, temp_x, temp_y, &font, temp_size);
    draw_text_left(&mut image, &current_weather.description, desc_x, desc_y, &font, 50.0);
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
    
    image::imageops::colorops::dither(&mut image, &TriColorDither);


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

    println!("image file {:?}", image.write_to(&mut output_image_file, image::ImageOutputFormat::Png));

    println!("binary data {:?}", output_data_file.write_all(&buffer));
}
