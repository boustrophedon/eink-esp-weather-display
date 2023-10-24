This is the rendering code for https://harrystern.net/halldisplay.html

This directory contains code that gathers weather and todo data from standard HTTP REST APIs, draws graphs and text displaying that data into an image buffer, and then formats the image data so that it can be directly displayed on a waveshare e-ink display. The data is saved to a file which can be served from a normal webserver like nginx.

# Running

To run this code, you will need to create a `env_data.json` (any name is fine) of the form

```
{
    "local_timezone": "IANA TZ identifier e.g. America/New_York",
    "user_agent": "user agent used when making http requests",
    "tasks_api_key":"todoist api key",
    "weather_station":"See the weather.gov api documentation https://www.weather.gov/documentation/services-web-api",
    "weather_office":"see api docs",
    "weather_gridpoint":"see api docs",
}
```

Then to actually run the program

```
cargo run -- <env_file.json> <output_file.img>
```

This will put the output file for the device into `<output_file.img>` and a png version in `<output_file.png>`.
