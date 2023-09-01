use chrono::NaiveDate;
use reqwest::blocking::Client;
use serde_json::Value;

use crate::EnvData;

const TASKS_URL: &'static str = "https://api.todoist.com/rest/v2/tasks";

const KEY: &[u8] = &[99, 46, 112, 105, 110, 100, 115, 39, 99, 46, 112, 100, 105, 110, 99, 46, 112, 105, 100, 110, 97, 99, 46, 103, 112, 100, 117, 97, 110, 99, 46, 112, 103, 105, 100, 117, 110, 99, 46, 112];

fn get_tasks_key(env_data: &EnvData) -> String {
    let api_key = env_data.tasks_api_key.clone();

    let mut out = Vec::new();
    assert_eq!(api_key.len(), KEY.len());

    for i in 0..api_key.len() {
        out.push(api_key[i] ^ KEY[i]);
    }

    String::from_utf8(out)
        .expect("result api key wasn't utf-8")
}

#[derive(Debug, Clone)]
pub struct Task {
    pub due_date: NaiveDate,
    pub description: String,
}

pub fn create_todoist_client(env_data: &EnvData) -> Client {
    let api_key = get_tasks_key(env_data);
    let bearer = format!("Bearer {api_key}");


    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Authorization", bearer.parse().unwrap());
    reqwest::blocking::Client::builder()
        .user_agent(&env_data.user_agent)
        .default_headers(headers)
        .build().expect("couldn't create requests client")
}

pub fn get_tasks(client: &Client) -> String {
    let json_str = client.get(TASKS_URL)
        .query(&[("filter", "due before: +48 hours")])
        //.query(&[("filter", "due before: +48 hours & due after: -24 hours")])
        .send()
        .expect("failed to make todoist tasks request")
        .text()
        .expect("failed to get text from todoist tasks request");

    return json_str;
}

pub fn parse_tasks(json_str: &str) -> Vec<Task> {
    let mut output = Vec::new();

    let data: Value = serde_json::from_str(&json_str)
        .expect("failed to parse todoist tasks json");

    for obj in data.as_array().expect("data wasn't a list") {
        let obj = obj.as_object().expect("item wasn't task object");
        let description = obj["content"].as_str()
            .expect("failed getting task description/content").to_string();
        let due_date: NaiveDate = obj["due"]["date"].as_str()
            .expect("failed to get due date string")
            .parse()
            .expect("failed to parse due date");

        // remove when api 500 is fixed and just push always
        let current_date = chrono::Utc::now().date_naive();
        let offset = chrono::Duration::days(2);
        if due_date > current_date - offset {
            output.push(Task {
                description,
                due_date
            });
        }
    }

    output
}
