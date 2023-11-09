use std::fs::File;
use std::io::prelude::*;

use std::path::PathBuf;

use std::sync::mpsc::sync_channel;
use std::thread;

pub(crate) mod test_data;

mod util;
use util::*;

mod env_data;
pub use env_data::*;

mod weather;
pub use weather::*;

mod tasks;
pub use tasks::*;

pub(crate) mod text;

pub mod draw;
pub use draw::*;

mod render;
pub use render::*;

pub fn get_env_data() -> EnvData {
    let file_path = get_envdata_filepath();
    EnvData::from_file(&file_path)
}

pub fn get_envdata_filepath() -> PathBuf {
    let path_str = std::env::args().nth(1)
        .expect("1st cli argument did not exist");
    PathBuf::from(path_str)
}

pub fn get_output_path() -> PathBuf {
    let path_str = std::env::args().nth(2)
        .expect("2nd cli argument did not exist");
    PathBuf::from(path_str) 
}

pub struct DisplayData {
    current_weather: CurrentWeather,
    forecast: Forecast5Day,
    todoist_tasks: Vec<Task>,
}

/// Returns (current_weather_json, hourly_weather_json, tasks_json)
fn gather_data(env_data: &EnvData) -> (String, String, String) {
    extrasafe::SafetyContext::new()
        .enable(
            extrasafe::builtins::SystemIO::nothing()
                .allow_dns_files()
                .allow_ssl_files()
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

    (current_weather_json, hourly_forecast_json, tasks_json)
}

fn parse_data(current_weather_json: String, hourly_forecast_json: String, tasks_json: String) -> DisplayData {
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
    let full_forecast = parse_hourly_forecast(&hourly_forecast_json);
    let forecast = Forecast5Day::new(&full_forecast);

    DisplayData {
        current_weather,
        forecast,
        todoist_tasks,
    }
}

fn main() {
    if std::env::args().len() != 3 {
        eprintln!("usage: halldisplay <env json filename> <output filename>");
        return;
    }
    let env_data = get_env_data();
    let output_filepath = get_output_path();

    let mut output_data_file = File::create(&output_filepath).expect("failed to create file");
    let mut output_image_file = File::create(&output_filepath.with_extension("png")).expect("failed to create file");

    // I'm basically doing a state machine manually here so technically this would be a good place
    // for async. I think it would be better to just build this into extrasafe somehow.
    let (parse_start, parse_start_rx) = sync_channel::<()>(0);

    let (json_sender, json_receiver) = sync_channel(1);
    let (data_sender, data_receiver) = sync_channel(1);

    let display_data: DisplayData;

    let use_debug_data = false;
    if !use_debug_data {
        let env_data = env_data.clone();
        thread::spawn(move || {
            parse_start_rx.recv().unwrap();
            let data = gather_data(&env_data);
            json_sender.send(data).unwrap();
        });

        thread::spawn(move || {
            let (current_weather_json, hourly_weather_json, tasks_json) = json_receiver.recv()
                .expect("failed to get json");
            let display_data = parse_data(current_weather_json, hourly_weather_json, tasks_json);
            data_sender.send(display_data).unwrap();
        });

    }
    else {
        data_sender.send(get_test_data()).unwrap();
    }

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
    parse_start.send(()).expect("failed to start json thread");
    display_data = data_receiver.recv()
            .expect("failed to get data");

    let current_time = chrono::Utc::now().with_timezone(&env_data.local_timezone);
    let (buffer, image) = render(current_time, display_data);

    println!("image file {:?}", image.write_to(&mut output_image_file, image::ImageOutputFormat::Png));

    println!("binary data {:?}", output_data_file.write_all(&buffer));
}
