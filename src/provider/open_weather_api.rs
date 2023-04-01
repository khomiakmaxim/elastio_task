use super::Provider;
pub struct OpenWeatherApi {
    api_key: String,
}

struct Coordinates {
    longitude: String,
    latitude: String,
}

impl OpenWeatherApi {
    pub fn new(api_key: String) -> OpenWeatherApi {
        OpenWeatherApi { api_key }
    }

    // Можна робити навіть щось таке тут
    // mykolaiv,lviv oblast, ukraine. // TODO: implement such an input from a console
    fn get_coordinates_per_place(&self, address: &str) -> anyhow::Result<Coordinates> {
        let uri = format!(
            "http://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}",
            address, self.api_key
        );

        let response = reqwest::blocking::get(uri)?.json::<serde_json::Value>()?;

        let location = response
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("No location data found"))?;

        let longitude = location
            .get("lon")
            .ok_or_else(|| anyhow::anyhow!("Missing longitude value"))?
            .to_string();

        let latitude = location
            .get("lat")
            .ok_or_else(|| anyhow::anyhow!("Missing latitude value"))?
            .to_string();

        Ok(Coordinates {
            longitude,
            latitude,
        })
    }
}

impl Provider for OpenWeatherApi {
    fn get_weather(
        &self,
        timestamp: Option<i64>,
        address: String,
    ) -> anyhow::Result<serde_json::Value> {
        let place_coords = self.get_coordinates_per_place(&address)?;

        let uri = if let Some(timestamp) = timestamp {
            format!("https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&dt={}&exclude=daily,minutely,hourly&appid={}", place_coords.latitude, place_coords.longitude, timestamp, self.api_key)
        } else {
            format!("https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&exclude=daily,minutely,hourly&appid={}", place_coords.latitude, place_coords.longitude, self.api_key)
        };

        let response = reqwest::blocking::get(uri)?.json::<serde_json::Value>()?;
        Ok(response)
    }
}
