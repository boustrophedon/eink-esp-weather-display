use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use embedded_svc::utils::io;

use embedded_svc::http::client::Client as HttpClient;
use embedded_svc::http::Method;

use log::*;

pub fn request_image(image_data_url: &str) -> anyhow::Result<Vec<u8>> {
    let mut client = create_client()?;

    get_data(&mut client, image_data_url)
}

fn create_client() -> anyhow::Result<HttpClient<EspHttpConnection>> {
    let config = Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    };

    Ok(HttpClient::wrap(EspHttpConnection::new(&config)?))
}

// This code is mostly taken from some sample code somewhere
fn get_data(client: &mut HttpClient<EspHttpConnection>, url: &str) -> anyhow::Result<Vec<u8>> {
    let headers = [("accept", "application/octet-stream"), ("connection", "close")];
    let request = client.request(Method::Get, &url, &headers)?;
    info!("making request {}", url);
    let mut response = request.submit()?;

    // Process response
    let status = response.status();
    info!("response status: {}", status);
    if status != 200 {
        anyhow::bail!("response status was not 200: {}", status);
    }

    let buffer_size = crate::get_buffer_size();
    let (_headers, mut body) = response.split();
    let mut buf = vec![0u8; buffer_size];
    let bytes_read = io::try_read_full(&mut body, &mut buf).map_err(|e| e.0)?;
    info!("Read {} bytes", bytes_read);

    // Drain the remaining response bytes
    // TODO: probably should error here since we should get exactly the buffer size
    while body.read(&mut buf)? > 0 {}

    if bytes_read == 0 {
        anyhow::bail!("Image data body was empty");
    }
    if bytes_read != buffer_size {
        anyhow::bail!("Image data body was wrong size. Expected {}, got {}", buffer_size, bytes_read);
    }
    Ok(buf)
}

