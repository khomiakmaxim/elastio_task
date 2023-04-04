//! Provider implementation, powered by <https://www.weatherapi.com>.
use std::time::Duration;

use anyhow::Context;
use chrono::{Local, NaiveDate};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{Provider, Weather};

static TIMEOUT_SECONDS: u64 = 5;
static WEATHER_API_ERROR: &str = "weather-api returned invalid data. \
        If your input is correct, this might be caused by limitations of current provider";

/// Concrete structure, which implements 'Provider' trait for weather-api API requests.
pub struct WeatherApi {
    api_key: String,
    https_client: Client,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrentWeatherData {
    current: WeatherInfo,
    location: Location,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct TimedWeatherData {
    forecast: Forecast,
    location: Location,
}

#[derive(Debug, Deserialize, Serialize)]
struct Location {
    name: String,
    region: String,
    country: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct WeatherInfo {
    temp_c: f64,
    temp_f: f64,
    condition: ConditionInfo,
}

#[derive(Debug, Deserialize, Serialize)]
struct Forecast {
    forecastday: Vec<ForecastDay>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ForecastDay {
    day: Day,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Day {
    avgtemp_c: f64,
    avgtemp_f: f64,
    maxwind_mph: f64,
    maxwind_kph: f64,
    condition: ConditionInfo,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ConditionInfo {
    text: String,
}

impl Provider for WeatherApi {
    /// Implementation of 'Provider' trait method. Returns the required JSON object in a readable format.
    ///
    /// # Errors:
    ///
    /// Backpropagates in case of invalid 'address', or API limitations.
    fn get_current_weather(&self, address: &str) -> anyhow::Result<Weather> {
        let response = self.get_current_weather_data(address)?;
        Ok(response)
    }

    /// Implementation of 'Provider' trait method. Returns the required JSON object in a readable format.
    ///
    /// # Errors:
    ///
    /// Backpropagates in case of invalid 'address' or 'date' or API limitations.
    fn get_timed_weather(&self, address: &str, date: &str) -> anyhow::Result<Weather> {
        let response = self.get_timed_weather_data(address, date)?;
        Ok(response)
    }
}

impl WeatherApi {
    pub fn new(api_key: String) -> WeatherApi {
        let https_client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(TIMEOUT_SECONDS))
            .build()
            .expect("Unable to build HTTPS client for weather-api provider. Contact developers for proceeding.");

        WeatherApi {
            api_key,
            https_client,
        }
    }

    fn get_response(&self, uri: &str) -> reqwest::Result<reqwest::blocking::Response> {
        self.https_client.get(uri).send()
    }

    fn get_current_weather_data(&self, address: &str) -> anyhow::Result<Weather> {
        let mut url = Url::parse("http://api.weatherapi.com/v1/current.json")?;
        url.query_pairs_mut()
            .append_pair("key", &self.api_key)
            .append_pair("q", address)
            .append_pair("aqi", "no");

        let response = self
            .get_response(url.as_str())?
            .json::<CurrentWeatherData>()
            .with_context(|| anyhow::anyhow!(WEATHER_API_ERROR))?;

        Ok(Weather::FromWeatherApiCurrent(response))
    }

    fn get_timed_weather_data(&self, address: &str, date: &str) -> anyhow::Result<Weather> {
        let date_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;
        let now_date = Local::now().date_naive();

        match date_date.cmp(&now_date) {
            std::cmp::Ordering::Greater => {
                let days_from_now = (date_date - now_date).num_days() + 1;
                self.get_forecast_weather_data(address, days_from_now)
            }
            _ => self.get_history_weather_data(address, date),
        }
    }

    fn get_forecast_weather_data(
        &self,
        address: &str,
        days_from_now: i64,
    ) -> anyhow::Result<Weather> {
        let mut url = Url::parse("http://api.weatherapi.com/v1/forecast.json")?;
        url.query_pairs_mut()
            .append_pair("key", &self.api_key)
            .append_pair("q", address)
            .append_pair("days", &days_from_now.to_string())
            .append_pair("aqi", "no")
            .append_pair("alerts", "no");

        let response = self
            .get_response(url.as_str())?
            .json::<TimedWeatherData>()
            .with_context(|| anyhow::anyhow!(WEATHER_API_ERROR))?;

        let last_day = response
            .forecast
            .forecastday
            .last()
            .ok_or(anyhow::anyhow!("weather-api returned invalid data"))?;

        let forecast = Forecast {
            forecastday: vec![(*last_day).clone()],
        };

        let response = TimedWeatherData {
            forecast,
            location: response.location,
        };

        Ok(Weather::FromWeatherApiTimed(response))
    }

    fn get_history_weather_data(&self, address: &str, date: &str) -> anyhow::Result<Weather> {
        let mut url = Url::parse("http://api.weatherapi.com/v1/history.json")?;

        url.query_pairs_mut()
            .append_pair("key", &self.api_key)
            .append_pair("q", address)
            .append_pair("dt", date);

        let response = self
            .get_response(url.as_str())?
            .json::<TimedWeatherData>()
            .with_context(|| anyhow::anyhow!(WEATHER_API_ERROR))?;

        Ok(Weather::FromWeatherApiTimed(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::provider::ProviderName;
    use chrono::{Duration, Utc};
    use dotenvy::dotenv;

    lazy_static::lazy_static! {
        static ref API_KEY: String = {
            let provider_name = ProviderName::WeatherApi;
            dotenv().ok();
            std::env::var(provider_name.to_string()).expect(format!("{}_API_KEY not found in .env", provider_name).as_str())
        };
    }

    #[test]
    #[ignore]
    fn test_get_weather_api_current() {
        let provider = WeatherApi::new(API_KEY.to_string());
        let weather = provider.get_current_weather("Mykolaiv, Lviv oblast, Ukraine");
        assert!(weather.is_ok());
    }

    #[test]
    #[ignore]
    fn test_get_weather_api_timed_yesterday_weather() {
        let provider = WeatherApi::new(API_KEY.to_string());

        let now = Utc::now();
        let yesterday = now - Duration::days(1);
        let formatted_yesterday = yesterday.format("%Y-%m-%d");

        let weather =
            provider.get_timed_weather("Odesa, Ukraine", &formatted_yesterday.to_string());
        assert!(weather.is_ok());
    }

    #[test]
    #[ignore]
    fn test_get_weather_api_timed_tommorow_weather() {
        let provider = WeatherApi::new(API_KEY.to_string());

        let now = Utc::now();
        let tommorow = now + Duration::days(1);
        let formatted_tommorow = tommorow.format("%Y-%m-%d");

        let weather = provider.get_timed_weather(
            "Mykolaiv, Lviv oblast, Ukraine",
            &formatted_tommorow.to_string(),
        );
        assert!(weather.is_ok());
    }

    #[test]
    #[ignore]
    fn test_get_weather_api_timed_invalid_timestamp() {
        let provider = WeatherApi::new(API_KEY.to_string());
        let date = "088-04-01";
        let result = provider.get_timed_weather("Mykolaiv, Lviv oblast, Ukraine", date);
        assert!(result.is_err());
    }
}
