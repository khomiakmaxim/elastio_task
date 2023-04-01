use reqwest::blocking::Client;
use std::time::Duration;

use super::Provider;

static TIMEOUT_SECONDS: u64 = 5;

pub struct OpenWeatherApi {
    https_client: Client,
    api_key: String,
}

struct Coordinates {
    longitude: String,
    latitude: String,
}

impl OpenWeatherApi {
    pub fn new(api_key: String) -> OpenWeatherApi {
        let https_client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(TIMEOUT_SECONDS))
            .build()
            .expect("Unable to build HTTPS client");
        OpenWeatherApi {
            https_client,
            api_key,
        }
    }

    fn get_response(&self, uri: &str) -> reqwest::Result<serde_json::Value> {
        self.https_client
            .get(uri)
            .send()?
            .json::<serde_json::Value>()
    }

    // TODO: implement such an input "Mykolaiv, Lviv oblast, Ukraine. " being possible to enter from a console
    fn get_coordinates_per_place(&self, address: &str) -> anyhow::Result<Coordinates> {
        let uri = format!(
            "http://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}",
            address, self.api_key
        );

        let response = self.get_response(&uri)?;

        let location = response
            .as_array()
            .ok_or_else(|| {
                anyhow::anyhow!("open-weather-api returned an invalid response format.")
            })?
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("No location data found in API response."))?;

        let longitude = location
            .get("lon")
            .ok_or_else(|| anyhow::anyhow!("Missing longitude value from API."))?
            .to_string();

        let latitude = location
            .get("lat")
            .ok_or_else(|| anyhow::anyhow!("Missing latitude value from API."))?
            .to_string();

        Ok(Coordinates {
            longitude,
            latitude,
        })
    }

    fn get_timed_weather_data(
        &self,
        coords: &Coordinates,
        timestamp: i64,
    ) -> anyhow::Result<serde_json::Value> {
        let uri = format!("https://api.openweathermap.org/data/3.0/onecall/timemachine?lat={}&lon={}&dt={}&appid={}&units=metric", coords.latitude, coords.longitude, timestamp, self.api_key);
        let response = self.get_response(&uri)?;

        if let Some(data) = response.get("data").and_then(|d| d.as_array()) {
            if let Some(first_data_point) = data.get(0) {
                return Ok(first_data_point.to_owned());
            }
        }

        Err(anyhow::anyhow!("open-weather-api returned an invalid response. Please make sure your request has a valid date and try again."))
    }

    fn get_current_weather_data(&self, coords: &Coordinates) -> anyhow::Result<serde_json::Value> {
        let uri = format!("https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&exclude=daily,minutely,hourly&appid={}&units=metric", coords.latitude, coords.longitude, self.api_key);
        let response = self.get_response(&uri)?;

        if let Some(current) = response.get("current") {
            Ok(current.to_owned())
        } else {
            Err(anyhow::anyhow!(
                "open-weather-api returned an invalid response."
            ))
        }
    }
}

impl Provider for OpenWeatherApi {
    fn get_weather(
        &self,
        timestamp: Option<i64>,
        address: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let response;

        let place_coords = self.get_coordinates_per_place(address)?;
        if let Some(timestamp) = timestamp {
            response = self.get_timed_weather_data(&place_coords, timestamp)?;
        } else {
            response = self.get_current_weather_data(&place_coords)?;
        }

        Ok(response)
    }
}
