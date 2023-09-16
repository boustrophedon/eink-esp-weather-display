use crate::{CurrentWeather, DisplayData, Task};

use chrono::{Duration, DateTime};

pub(crate) fn get_test_data() -> DisplayData {
    let current_weather = CurrentWeather {
        description: "test data".into(),
        temp_f: 69,
        rain_in: 0,
    };
    let avg_forecast_ = [("2023-05-17T21:00:00-04:00", 53, 0), ("2023-05-18T01:00:00-04:00", 47, 0), ("2023-05-18T05:00:00-04:00", 46, 0), ("2023-05-18T09:00:00-04:00", 54, 0), ("2023-05-18T13:00:00-04:00", 59, 0), ("2023-05-18T17:00:00-04:00", 57, 0), ("2023-05-18T21:00:00-04:00", 54, 0), ("2023-05-19T01:00:00-04:00", 51, 0), ("2023-05-19T05:00:00-04:00", 54, 0), ("2023-05-19T09:00:00-04:00", 62, 1), ("2023-05-19T13:00:00-04:00", 66, 4), ("2023-05-19T17:00:00-04:00", 64, 6), ("2023-05-19T21:00:00-04:00", 60, 12), ("2023-05-20T01:00:00-04:00", 59, 26), ("2023-05-20T05:00:00-04:00", 60, 39), ("2023-05-20T09:00:00-04:00", 64, 66), ("2023-05-20T13:00:00-04:00", 66, 63), ("2023-05-20T17:00:00-04:00", 67, 55), ("2023-05-20T21:00:00-04:00", 66, 36), ("2023-05-21T01:00:00-04:00", 61, 27), ("2023-05-21T05:00:00-04:00", 60, 21), ("2023-05-21T09:00:00-04:00", 67, 10), ("2023-05-21T13:00:00-04:00", 75, 6), ("2023-05-21T17:00:00-04:00", 74, 5), ("2023-05-21T21:00:00-04:00", 68, 4), ("2023-05-22T01:00:00-04:00", 62, 4), ("2023-05-22T05:00:00-04:00", 62, 4), ("2023-05-22T09:00:00-04:00", 69, 4), ("2023-05-22T13:00:00-04:00", 74, 6), ("2023-05-22T17:00:00-04:00", 70, 6)];
    let avg_forecast = avg_forecast_.into_iter().map(|(dt, t, r)| (DateTime::parse_from_rfc3339(dt).unwrap(), t, r)).collect();

    let today = chrono::offset::Local::now().date_naive();
    let yesterday = today - Duration::days(1);
    let tomorrow = today + Duration::days(1);
    let todoist_tasks = vec![Task { description: "test task".into(), due_date: today },
    Task { description: "task 2".into(), due_date: yesterday },
    Task { description: "task 3".into(), due_date: tomorrow }];

    DisplayData {current_weather, avg_forecast, todoist_tasks }
}
