This is the firmware code for https://harrystern.net/halldisplay.html

This directory contains esp-idf/esp-rs based firmware to download an image over wifi and display it on a waveshare e-ink display.

# Building

You must create a `src/config.rs` file containing your Wifi connection credentials and the location of the file to download and display.

Sample file:

```
use super::WifiConfig;

pub const WIFI_CONFIG_DATA: WifiConfig = WifiConfig {
    ssid: "your_ssid",
    psk: "your_psk",
};

pub const IMAGE_DATA_URL: &str = "https://example.com/data_file.img";
```

Then you can run `just build` to build assuming you have the rest of your environment set up as in the [embedded rust book](https://docs.rust-embedded.org/book/intro/install.html).

If you have an esp device plugged in over usb, you should be able to use `just run` to upload your code to the device. This is configured in `.cargo/config.toml` in the current directory.
