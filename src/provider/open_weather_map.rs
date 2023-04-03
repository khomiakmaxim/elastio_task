use anyhow::Context;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

use super::Provider;

static TIMEOUT_SECONDS: u64 = 5;

// Powered by https://openweathermap.org
pub struct OpenWeatherApi {
    https_client: Client,
    api_key: String,
}
#[derive(Serialize, Debug, Deserialize, Clone)]
struct Coordinates {
    lon: f64,
    lat: f64,
}
#[derive(Debug, Deserialize, Serialize)]
struct CurrentWeatherData {
    current: WeatherInfo,
    timezone: String,
    lat: f64,
    lon: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TimedWeatherData {
    data: Vec<WeatherInfo>,
    timezone: String,
    lat: f64,
    lon: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct WeatherInfo {
    temp: f64,
    feels_like: f64,
    pressure: i64,
    humidity: i64,
    wind_speed: f64,
    wind_deg: i64,
    weather: Vec<ConditionInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConditionInfo {
    main: String,
    description: String,
}

impl Provider for OpenWeatherApi {
    fn get_current_weather(&self, address: &str) -> anyhow::Result<String> {
        let place_coords = self.get_coordinates_per_place(address)?;
        let response = self.get_current_weather_parsed_data(&place_coords)?;
        Ok(response)
    }

    fn get_timed_weather(&self, address: &str, date: &str) -> anyhow::Result<String> {
        let datetime = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|err| {
            eprintln!(
                "Error: {}\nEntered date should be in the YYYY-MM-DD format",
                err
            );
            err
        })?;

        let midday_datetime = NaiveDateTime::new(
            datetime,
            NaiveTime::from_hms_opt(12, 0, 0).expect(
                "Failed during time parameter initialization. Contact developers for proceeding.",
            ),
        );

        let place_coords = self.get_coordinates_per_place(address)?;
        let response =
            self.get_timed_weather_parsed_data(&place_coords, midday_datetime.timestamp())?;

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

    fn get_response(&self, uri: &str) -> reqwest::Result<reqwest::blocking::Response> {
        self.https_client.get(uri).send()
    }

    fn get_coordinates_per_place(&self, address: &str) -> anyhow::Result<Coordinates> {
        let mut url = Url::parse("http://api.openweathermap.org/geo/1.0/direct")?;
        url.query_pairs_mut()
            .append_pair("q", address)
            .append_pair("limit", "1")
            .append_pair("appid", &self.api_key);

        let response = self
            .get_response(url.as_str())?
            .json::<Vec<Coordinates>>()
            .with_context(|| anyhow::anyhow!("Failed to parse response from openweathermap"))?;

        if let Some(coordinates) = response.get(0) {
            Ok(coordinates.clone())
        } else {
            Err(anyhow::anyhow!("No coordinates found for {}", address))
        }
    }

    fn get_current_weather_parsed_data(&self, coords: &Coordinates) -> anyhow::Result<String> {
        let mut url = Url::parse("https://api.openweathermap.org/data/3.0/onecall")?;
        url.query_pairs_mut()
            .append_pair("lat", &coords.lat.to_string())
            .append_pair("lon", &coords.lon.to_string())
            .append_pair("exclude", "daily")
            .append_pair("exclude", "minutely")
            .append_pair("exclude", "hourly")
            .append_pair("appid", &self.api_key)
            .append_pair("units", "metric");

        let response = self
            .get_response(url.as_str())?
            .json::<CurrentWeatherData>()
            .with_context(|| {
                anyhow::anyhow!(
                    "open-weather-map returned invalid data. Please, consider changing provider"
                )
            })?;

        Ok(serde_json::to_string_pretty(&response)?)
    }

    fn get_timed_weather_parsed_data(
        &self,
        coords: &Coordinates,
        timestamp: i64,
    ) -> anyhow::Result<String> {
        let mut url = Url::parse("https://api.openweathermap.org/data/3.0/onecall/timemachine")?;
        url.query_pairs_mut()
            .append_pair("lat", &coords.lat.to_string())
            .append_pair("lon", &coords.lon.to_string())
            .append_pair("dt", &timestamp.to_string())
            .append_pair("appid", &self.api_key)
            .append_pair("units", "metric");

        let response = self
            .get_response(url.as_str())?
            .json::<TimedWeatherData>()
            .with_context(|| anyhow::anyhow!("open-weather-map returned inconsistent data. Make sure your request has a reasonable date"))?;

        Ok(serde_json::to_string_pretty(&response)?)
    }
}

// These all tests must be integral, not unit
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
