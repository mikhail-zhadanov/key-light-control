use reqwest::{self, Client};
use serde_json::json;

#[tokio::main]
pub async fn change(turn_on: bool, ip: &str, port: u16) -> Result<(), reqwest::Error> {
    let url = format!("http://{}:{}/elgato/lights", ip, port);
    let body = json!({
    "numberOfLights": 1,
    "lights": [
        {
            "on": turn_on as u8,
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
        println!("Light is turned {}", if turn_on { "on" } else { "off" });
    } else {
        println!("Failed to turn light {}", if turn_on { "on" } else { "off" });
    }
    Ok(())
}
