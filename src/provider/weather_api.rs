use super::Provider;

pub struct WeatherApi {
    api_key: String,
}

impl WeatherApi {
    pub fn new(api_key: String) -> WeatherApi {
        WeatherApi { api_key }
    }
}

// TODO: check for cases, when server is unreachable
impl Provider for WeatherApi {
    fn get_weather(
        &self,
        timestamp: Option<i64>,
        address: String,
    ) -> anyhow::Result<serde_json::Value> {
        let uri = if let Some(timestamp) = timestamp {
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
        let resp = reqwest::blocking::get(uri)?.json::<serde_json::Value>()?;
        Ok(resp)
    }
}
