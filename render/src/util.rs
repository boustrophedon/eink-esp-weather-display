use crate::{CurrentWeather, DisplayData, Task, Forecast5Day};
use crate::test_data::test_data1;

use chrono::Duration;
//use chrono::{Duration, DateTime};

pub(crate) fn get_test_data() -> DisplayData {
    let current_weather = CurrentWeather {
        description: "test data".into(),
        temp_f: 69,
        rain_in: 0,
    };

    let full_forecast = test_data1();
    let forecast = Forecast5Day { full_forecast };

    let today = chrono::offset::Local::now().date_naive();
    let yesterday = today - Duration::days(1);
    let tomorrow = today + Duration::days(1);
    let todoist_tasks = vec![Task { description: "test task".into(), due_date: today },
    Task { description: "task 2".into(), due_date: yesterday },
    Task { description: "task 3".into(), due_date: tomorrow }];

    DisplayData {current_weather, forecast, todoist_tasks }
}
