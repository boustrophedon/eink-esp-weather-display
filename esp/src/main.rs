use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;

use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::*;
use esp_idf_hal::gpio;
use esp_idf_hal::spi;
use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::modem::Modem;
use esp_idf_hal::delay::Delay;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use std::time::Duration;

use epd_waveshare::{
    color::*,
    epd7in5b_v2::{Epd7in5, WIDTH, HEIGHT},
    graphics::VarDisplay,
};
use epd_waveshare::prelude::*;

mod request_image;
mod wifi;
mod config;

use request_image::request_image;
use wifi::WifiConfig;
use config::{IMAGE_DATA_URL, WIFI_CONFIG_DATA};

type SpiDev = SpiDeviceDriver<'static, SpiDriver<'static>>;

type EpdDriver = Epd7in5<
    SpiDev,
    PinDriver<'static, AnyOutputPin, Output>,
    PinDriver<'static, AnyInputPin, Input>,
    PinDriver<'static, AnyOutputPin, Output>,
    PinDriver<'static, AnyOutputPin, Output>,
    Delay>;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("wakeup: {:?}", esp_idf_hal::reset::WakeupReason::get());

    let peripherals = Peripherals::take().unwrap();
    let (led_pin, wakeup_pin, modem, spi_driver, epd, delay) =
        gather_peripherals(peripherals)?;
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;


    disable_onboard_led(led_pin)?;



    let wifi = wifi::wifi(
        WIFI_CONFIG_DATA.ssid,
        WIFI_CONFIG_DATA.psk,
        modem,
        sysloop,
        nvs,
    ).expect("failed to connect to wifi");


    log::info!("request image from server");
    let image_result = request_image(IMAGE_DATA_URL);

    log::info!("turning off wifi");
    // turn off wifi
    drop(wifi);
    if let Ok(image_data) = image_result {
        log::info!("render image");
        draw_epd(image_data, spi_driver, epd, delay)?;
    }
    else {
        log::error!("getting image data failed: {:?}", image_result.unwrap_err());
    }

    // deep sleep for 1.5 hours or on wakeup button press
    enter_deep_sleep(wakeup_pin.into(), Duration::from_secs(60*30*3));

    unreachable!("in sleep");
}

fn gather_peripherals(peripherals: Peripherals) -> anyhow::Result<(Gpio2, Gpio4, Modem, SpiDev, EpdDriver, Delay)> {
    let pins = peripherals.pins;
    let ledpin = pins.gpio2;
    let wakeup_pin = pins.gpio4;

    let modem = peripherals.modem;

	let spi_p = peripherals.spi3;
	let sclk: AnyOutputPin = pins.gpio13.into();
	let sdo: AnyOutputPin = pins.gpio14.into();
	let cs: AnyOutputPin = pins.gpio15.into();
    let busy_in: AnyInputPin = pins.gpio25.into();
    let rst: AnyOutputPin = pins.gpio26.into();
    let dc: AnyOutputPin = pins.gpio27.into();

    info!("create epd driver");
    let (spi_driver, epd, delay) = create_epd_driver(spi_p, sclk, sdo, cs, busy_in, rst, dc)?;

    Ok((ledpin, wakeup_pin, modem, spi_driver, epd, delay))
}

fn enter_deep_sleep(wakeup_pin: AnyInputPin, sleep_time: Duration) {
    let wakeup_pin = PinDriver::input(wakeup_pin).expect("wakeup pin sleep");
    unsafe { esp_idf_sys::esp_sleep_enable_ext0_wakeup(wakeup_pin.pin(), 0); }
    info!("entering deep sleep");
    unsafe {
        // TODO: measure current draw vs gpio_deep_sleep_hold_en
        //esp_idf_sys::rtc_gpio_hold_en(led.pin());
        //esp_idf_sys::gpio_deep_sleep_hold_en()
        // TODO see if these need to be configured
        //  esp_sleep_pd_config(ESP_PD_DOMAIN_RTC_PERIPH, ESP_PD_OPTION_OFF);
        // esp_sleep_pd_config(ESP_PD_DOMAIN_RTC_SLOW_MEM, ESP_PD_OPTION_OFF);
        // esp_sleep_pd_config(ESP_PD_DOMAIN_RTC_FAST_MEM, ESP_PD_OPTION_OFF);
        // esp_sleep_pd_config(ESP_PD_DOMAIN_XTAL, ESP_PD_OPTION_OFF);
        esp_idf_sys::esp_deep_sleep(sleep_time.as_micros() as u64);
    }
    // unreachable!("we will be asleep by now");
}

/// Disable the onboard led during deep sleep
/// TODO: measure current draw vs gpio_deep_sleep_hold_en
fn disable_onboard_led(ledpin: gpio::Gpio2) -> anyhow::Result<()> {
    log::info!("disable onboard led");
    let mut led = PinDriver::output(ledpin)?;
    led.set_low()?;
    unsafe { esp_idf_sys::rtc_gpio_hold_en(led.pin()); }

    Ok(())
}

fn create_epd_driver(
	spi_p: spi::SPI3,
	sclk: AnyOutputPin,
	sdo: AnyOutputPin,
	cs: AnyOutputPin,
    busy_in: AnyInputPin,
    rst: AnyOutputPin,
    dc: AnyOutputPin,
    ) -> anyhow::Result<(SpiDev, EpdDriver, Delay)> {
    let mut driver = spi::SpiDeviceDriver::new_single(
        spi_p,
        sclk,
        sdo,
        Option::<gpio::AnyIOPin>::None,
        Option::<gpio::AnyOutputPin>::None,
        &spi::config::DriverConfig::new(),
        &spi::config::Config::new().baudrate(10.MHz().into()),
    )?;

    info!("driver setup completed");
    let mut delay = Delay {};

    // Setup EPD
    let epd_driver = Epd7in5::new(
        &mut driver,
        PinDriver::output(cs)?,
        PinDriver::input(busy_in)?,
        PinDriver::output(dc)?,
        PinDriver::output(rst)?,
        &mut delay,
        None,
    )
    .unwrap();

    info!("epd setup completed");

    Ok((driver, epd_driver, delay))
}

fn draw_epd(mut buffer: Vec<u8>, mut driver: SpiDev, mut epd: EpdDriver, mut delay: Delay) -> anyhow::Result<()> {
	let expected_len = epd_waveshare::buffer_len(WIDTH as usize, HEIGHT as usize * 2);
    let buffer_len = buffer.len();
    if buffer_len != expected_len {
        anyhow::bail!("buffer len expected {}, got {}", expected_len, buffer_len);
    }
    let display = VarDisplay::<TriColor>::new(WIDTH, HEIGHT, &mut buffer, false).expect("failed to create display");

    epd
        .update_and_display_frame(&mut driver, display.buffer(), &mut delay)
        .expect("display frame");
    info!("called display frame");
    Delay::delay_ms(20_000u32);

    info!("done waiting, putting display to sleep");
    epd.sleep(&mut driver, &mut delay).expect("failed to sleep");

    Ok(())
}

