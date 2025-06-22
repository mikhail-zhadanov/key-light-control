use reqwest::{self, Client};
use serde_json::json;

#[tokio::main]
pub async fn set_state(
    turn_on: bool,
    ip: &str,
    port: u16,
    brightness: u8,
    temperature: u16,
) -> Result<(), reqwest::Error> {
    let url = format!("http://{}:{}/elgato/lights", ip, port);
    let body = json!({
        "numberOfLights": 1,
        "lights": [
            {
                "on": turn_on as u8,
                "brightness": brightness,
                "temperature": kelvin_to_api_temp(temperature),
            }
        ]
    });

    let req = Client::new()
        .put(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;
    if req.status().is_success() {
        println!(
            "Light is turned {}, brightness: {}, temperature: {}",
            if turn_on { "on" } else { "off" },
            brightness,
            temperature
        );
    } else {
        println!("Failed to set light state");
        println!("Status: {}", req.status());
        match req.json::<serde_json::Value>().await {
            Ok(data) => println!("Data sent: {}", data),
            Err(e) => println!("Failed to get response body: {}", e),
        }
        println!("Data sent: {}", body);
    }
    Ok(())
}

#[tokio::main]
pub async fn get_state(ip: &str, port: u16) -> Result<(bool, u8, u16), Box<dyn std::error::Error>> {
    let url = format!("http://{}:{}/elgato/lights", ip, port);
    let resp = Client::new()
        .get(&url)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(lights) = resp
        .get("lights")
        .and_then(|l| l.as_array())
        .and_then(|a| a.first())
    {
        let on = lights.get("on").and_then(|o| o.as_u64()).unwrap_or(0) != 0;
        let brightness = lights
            .get("brightness")
            .and_then(|b| b.as_u64())
            .unwrap_or(0) as u8;
        let temperature = lights
            .get("temperature")
            .and_then(|t| t.as_u64())
            .map(|v| api_temp_to_kelvin(v as u16))
            .unwrap_or(0);
        Ok((on, brightness, temperature))
    } else {
        // Return a custom error message
        Err("Failed to parse light state".into())
    }
}

/// Convert API temperature value to Kelvin (rounded to nearest 50K)
pub fn api_temp_to_kelvin(api_value: u16) -> u16 {
    let a = -0.04902439;
    let b = 486.1951;
    let kelvin = ((api_value as f32 - b) / a).round() as u16;
    kelvin.clamp(2900, 7000)
}

/// Convert Kelvin to API temperature value
pub fn kelvin_to_api_temp(kelvin: u16) -> u16 {
    let a = -0.04902439;
    let b = 486.1951;
    let api_val = (a * (kelvin as f32) + b).round() as u16;
    api_val.clamp(143, 344)
}
