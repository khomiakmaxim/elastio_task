use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use reqwest::blocking::Client;
use std::time::Duration;

use super::Provider;

static TIMEOUT_SECONDS: u64 = 5;

// Powered by https://openweathermap.org
pub struct OpenWeatherApi {
    https_client: Client,
    api_key: String,
}

struct Coordinates {
    longitude: String,
    latitude: String,
}

impl Provider for OpenWeatherApi {
    fn get_current_weather(&self, address: &str) -> anyhow::Result<serde_json::Value> {
        let place_coords = self.get_coordinates_per_place(address)?;
        let response = self.get_current_weather_data(&place_coords)?;

        Ok(response)
    }

    fn get_timed_weather(&self, address: &str, date: &str) -> anyhow::Result<serde_json::Value> {
        let datetime = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|err| {
            eprintln!(
                "Error: {}\nEntered date should be in the YYYY-MM-DD format",
                err
            );
            err
        })?;
        let midday_datetime =
            NaiveDateTime::new(datetime, NaiveTime::from_hms_opt(12, 0, 0).unwrap()); // TODO: think of this unwrap()
        let place_coords = self.get_coordinates_per_place(address)?;
        let response = self.get_timed_weather_data(&place_coords, midday_datetime.timestamp())?;
        Ok(response)
    }
}

impl OpenWeatherApi {
    pub fn new(api_key: String) -> OpenWeatherApi {
        let https_client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(TIMEOUT_SECONDS))
            .build()
            .expect("Unable to build HTTPS client for open-weather-map provider. Contact developers for proceeding.");
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

    fn get_coordinates_per_place(&self, address: &str) -> anyhow::Result<Coordinates> {
        let uri = format!(
            "http://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}",
            address, self.api_key
        );

        let response = self.get_response(&uri)?;

        let location = response
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("open-weather-map returned an invalid response format. Please, consider changing provider"))?
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("open-weather-map response has no location data. Please, consider changing provider"))?;

        let longitude = location
            .get("lon")
            .ok_or_else(|| anyhow::anyhow!("open-weather-map response is missing longitude value. Please, consider changing provider"))?
            .to_string();

        let latitude = location
            .get("lat")
            .ok_or_else(|| anyhow::anyhow!("open-weather-map response is missing latitude value. Please, consider changing provider"))?
            .to_string();

        Ok(Coordinates {
            longitude,
            latitude,
        })
    }

    fn get_current_weather_data(&self, coords: &Coordinates) -> anyhow::Result<serde_json::Value> {
        let uri = format!("https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&exclude=daily,minutely,hourly&appid={}&units=metric", coords.latitude, coords.longitude, self.api_key);
        let response = self.get_response(&uri)?;

        if let Some(current) = response.get("current") {
            Ok(current.to_owned())
        } else {
            Err(anyhow::anyhow!(
                "open-weather-map returned an invalid response. Please, consider changing provider"
            ))
        }
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

        Err(anyhow::anyhow!("open-weather-map returned an invalid response. Make sure your request has a valid date. If yes, consider changing provider"))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::provider::ProviderName;
    use dotenvy::dotenv;

    lazy_static::lazy_static! {
        static ref API_KEY: String = {
            let provider_name = ProviderName::OpenWeatherMap;
            dotenv().ok();
            std::env::var(provider_name.to_string()).expect(format!("{}_API_KEY not found in .env", provider_name).as_str())
        };
    }

    #[test]
    fn test_get_weather_current() {
        let provider = OpenWeatherApi::new(API_KEY.to_string());
        let weather = provider.get_current_weather("Mykolaiv, Lviv oblast, Ukraine");
        assert!(weather.is_ok());
    }

    #[test]
    fn test_get_weather_current_invalid_address() {
        let provider = OpenWeatherApi::new(API_KEY.to_string());
        let weather = provider.get_current_weather("SO INVALID ADDRESS");
        assert!(weather.is_err());
    }

    #[test]
    fn test_get_weather_timed() {
        let provider = OpenWeatherApi::new(API_KEY.to_string());
        let date = "2022-04-02";
        let weather = provider.get_timed_weather("Mykolaiv, Lviv oblast, Ukraine", date);
        assert!(weather.is_ok());
    }

    #[test]
    fn test_get_weather_timed_invalid_timestamp() {
        let provider = OpenWeatherApi::new(API_KEY.to_string());
        let date = "988-04-01";
        let result = provider.get_timed_weather("Mykolaiv, Lviv oblast, Ukraine", date);
        assert!(result.is_err());
    }
}
