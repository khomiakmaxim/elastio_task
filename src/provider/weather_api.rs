use super::Provider;

use serde_json::Value;

pub struct WeatherApi {
    api_key: String,
}

impl WeatherApi {
    pub fn new(api_key: String) -> WeatherApi {
        WeatherApi { api_key }
    }
}

impl WeatherApi {
    fn parse_response(resp: Value) -> Value {
        println!("{:?}", resp["current"]);
        resp
    }
}

impl Provider for WeatherApi {
    fn get_weather(
        &self,
        timestamp: Option<i64>,
        address: String,
    ) -> anyhow::Result<serde_json::Value> {
        // Maybe this result must be in some unifying format as well
        let uri = if let Some(timestamp) = timestamp {
            // Here it should be more complex
            format!(
                "http://api.weatherapi.com/v1/current.json?key={}&dt={}&q={}&aqi=no",
                self.api_key, timestamp, address
            )
        } else {
            format!(
                "http://api.weatherapi.com/v1/current.json?key={}&q={}&aqi=no",
                self.api_key, address
            )
        };
        let resp = Self::parse_response(reqwest::blocking::get(uri)?.json::<Value>()?);
        Ok(resp)
    }
}
