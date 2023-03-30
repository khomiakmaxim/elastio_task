use super::Provider;
pub struct OpenWeatherApi {
    api_key: String,
}

impl OpenWeatherApi {
    pub fn new(api_key: String) -> OpenWeatherApi {
        OpenWeatherApi { api_key }
    }
}

// Тепер треба продумати методи отримання і парсинг відповідей від провайдера

impl Provider for OpenWeatherApi {
    fn get_weather(
        &self,
        timestamp: Option<i64>,
        address: String,
    ) -> anyhow::Result<serde_json::Value> {
        let uri = format!(
            "https://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}",
            address, self.api_key
        );
        let response = reqwest::blocking::get(uri)?.json::<serde_json::Value>()?;
        let (lat, lon) = (&response[0]["lat"], &response[0]["lon"]);
        let uri = if let Some(timestamp) = timestamp {
            format!("https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&dt={}&exclude=daily,minutely,hourly&appid={}", lat, lon, timestamp, self.api_key)
        } else {
            format!("https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&exclude=daily,minutely,hourly&appid={}", lat, lon, self.api_key)
        };

        let response = reqwest::blocking::get(uri)?.json::<serde_json::Value>()?;
        Ok(response)
    }
}
