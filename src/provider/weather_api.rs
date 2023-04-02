use chrono::{Local, NaiveDate};
use reqwest::blocking::Client;
use std::time::Duration;

use super::Provider;

static TIMEOUT_SECONDS: u64 = 5;

// Powered by https://www.weatherapi.com
pub struct WeatherApi {
    api_key: String,
    https_client: Client,
}

impl Provider for WeatherApi {
    fn get_current_weather(&self, address: &str) -> anyhow::Result<serde_json::Value> {
        let response = self.get_current_weather_data(address)?;
        Ok(response)
    }

    fn get_timed_weather(&self, address: &str, date: &str) -> anyhow::Result<serde_json::Value> {
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

    fn get_response(&self, uri: &str) -> reqwest::Result<serde_json::Value> {
        self.https_client
            .get(uri)
            .send()?
            .json::<serde_json::Value>()
    }

    fn get_current_weather_data(&self, address: &str) -> anyhow::Result<serde_json::Value> {
        println!("Current weather via weather-api is being retrieved");
        let uri = format!(
            "http://api.weatherapi.com/v1/current.json?key={}&q={}&aqi=no",
            self.api_key, address
        );
        let response = self.get_response(&uri)?;

        if response.get("current").is_some() {
            Ok(response)
        } else {
            Err(anyhow::anyhow!(
                "weather-api returned an invalid response. Please, consider changing provider"
            ))
        }
    }

    fn get_timed_weather_data(
        &self,
        address: &str,
        date: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let date_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;
        let now_date = Local::now().date_naive();

        match date_date.cmp(&now_date) {
            std::cmp::Ordering::Greater => {
                let days_from_now = (date_date - now_date).num_days();
                self.get_forecast_weather_data(address, days_from_now)
            }
            _ => self.get_history_weather_data(address, date),
        }
    }

    fn get_forecast_weather_data(
        &self,
        address: &str,
        days_from_now: i64,
    ) -> anyhow::Result<serde_json::Value> {
        println!("Forecast via weather-api is being retrieved");
        let uri = format!(
            "http://api.weatherapi.com/v1/forecast.json?key={}&q={}&days={}&aqi=no&alerts=no",
            self.api_key, address, days_from_now
        );
        let response = self.get_response(&uri)?;

        if response.get("location").is_some() {
            let extracted_response = self.get_location_and_day_for_forecast(response)?;
            Ok(extracted_response)
        } else {
            Err(anyhow::anyhow!(
                "weather-api returned an invalid response. Please, consider changing provider"
            ))
        }
    }

    fn get_history_weather_data(
        &self,
        address: &str,
        date: &str,
    ) -> anyhow::Result<serde_json::Value> {
        println!("History weather via weather-api is being retrieved");
        let uri = format!(
            "http://api.weatherapi.com/v1/history.json?key={}&q={}&dt={}",
            self.api_key, address, date
        );
        let response = self.get_response(&uri)?;

        if response.get("location").is_some() {
            let extracted_response = self.get_location_and_day_for_history(response)?;
            Ok(extracted_response)
        } else {
            Err(anyhow::anyhow!("weather-api returned an invalid response. Make sure you date is within 3 months to present for current provider"))
        }
    }

    fn get_location_and_day_for_forecast(
        &self,
        response: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let mut result_map = serde_json::Map::new();

        let location = response
            .get("location")
            .ok_or_else(|| {
                anyhow::anyhow!("weather-map response does not contain 'location' field. Please, consider changing provider")
            })?
            .clone();
        result_map.insert("location".to_owned(), location);

        let forecastday_array = response
            .get("forecast")
            .ok_or_else(|| {
                anyhow::anyhow!("weather-map response does not contain 'forecast' field. Please, consider changing provider")
            })?
            .get("forecastday")
            .ok_or_else(|| {
                anyhow::anyhow!("weather-map 'forecast' field does not contain 'forecastday' field. Please, consider changing provider")
            })?;

        let last_forecast_day = forecastday_array
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("weather-map 'forecastday' field is not an array. Please, consider changing provider"))?
            .last()
            .ok_or_else(|| anyhow::anyhow!("weather-map 'forecastday' array is empty. Please, consider changing provider"))?
            .get("day")
            .ok_or_else(|| {
                anyhow::anyhow!("weather-map 'forecastday' element does not contain 'day' field. Please, consider changing provider")
            })?
            .clone();

        result_map.insert("day".to_owned(), last_forecast_day);

        Ok(serde_json::Value::Object(result_map))
    }

    fn get_location_and_day_for_history(
        &self,
        response: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let mut result_map = serde_json::Map::new();

        let location = response
            .get("location")
            .ok_or_else(|| {
                anyhow::anyhow!("weather-map response does not contain 'location' field. Please, consider changing provider")
            })?
            .clone();
        result_map.insert("location".to_owned(), location);

        let forecast_day = response
            .get("forecast")
            .ok_or_else(|| {
                anyhow::anyhow!("weather-map response does not contain 'forecast' field. Please, consider changing provider")
            })?
            .get("forecastday")
            .ok_or_else(|| {
                anyhow::anyhow!("weather-map 'forecast' field does not contain 'forecastday' field. Please, consider changing provider")
            })?
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("weather-map 'forecastday' array is empty. Please, consider changing provider"))?
            .get("day")
            .ok_or_else(|| {
                anyhow::anyhow!("weather-map 'forecastday' element does not contain 'day' field. Please, consider changing provider")
            })?
            .clone();
        result_map.insert("day".to_owned(), forecast_day);

        Ok(serde_json::Value::Object(result_map))
    }
}
