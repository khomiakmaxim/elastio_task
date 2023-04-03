use anyhow::Context;
use chrono::{Local, NaiveDate};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::Provider;

static TIMEOUT_SECONDS: u64 = 5;

#[derive(Debug, Deserialize, Serialize)]
struct CurrentWeatherData {
    current: WeatherInfo,
    location: Location,
}
#[derive(Debug, Deserialize, Serialize)]
struct TimedWeatherData {
    forecast: Forecast,
    location: Location,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ConditionInfo {
    text: String,
}

// Powered by https://www.weatherapi.com
pub struct WeatherApi {
    api_key: String,
    https_client: Client,
}

impl Provider for WeatherApi {
    fn get_current_weather(&self, address: &str) -> anyhow::Result<String> {
        let response = self.get_current_weather_data(address)?;
        Ok(response)
    }

    fn get_timed_weather(&self, address: &str, date: &str) -> anyhow::Result<String> {
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

    fn get_current_weather_data(&self, address: &str) -> anyhow::Result<String> {
        println!("Current weather via weather-api is being retrieved");
        let uri = format!(
            "http://api.weatherapi.com/v1/current.json?key={}&q={}&aqi=no",
            self.api_key, address
        );
        let response = self.get_response(&uri)?.json::<CurrentWeatherData>()?;

        Ok(serde_json::to_string_pretty(&response)?)
    }

    fn get_timed_weather_data(&self, address: &str, date: &str) -> anyhow::Result<String> {
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
    ) -> anyhow::Result<String> {
        let uri = format!(
            "http://api.weatherapi.com/v1/forecast.json?key={}&q={}&days={}&aqi=no&alerts=no",
            self.api_key, address, days_from_now
        );
        let response = self
            .get_response(&uri)?
            .json::<TimedWeatherData>()
            .with_context(|| {
                anyhow::anyhow!(
                    "weather-api returned invalid data. \
    If your input is correct, this might be caused by limitations of current provider"
                )
            })?;

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

        Ok(serde_json::to_string_pretty(&response)?)
    }

    fn get_history_weather_data(&self, address: &str, date: &str) -> anyhow::Result<String> {
        let uri = format!(
            "http://api.weatherapi.com/v1/history.json?key={}&q={}&dt={}",
            self.api_key, address, date
        );
        let response = self
            .get_response(&uri)?
            .json::<TimedWeatherData>()
            .with_context(|| {
                anyhow::anyhow!(
                    "weather-api returned invalid data. \
         If your input is correct, this might be caused by limitations of current provider"
                )
            })?;

        Ok(serde_json::to_string_pretty(&response)?)
    }
}
