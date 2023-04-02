use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(
    Debug,
    Clone,
    Copy,
    clap::Subcommand,
    Serialize,
    Deserialize,
    strum_macros::Display,
    EnumIter,
    Hash,
    PartialEq,
    Eq,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderName {
    OpenWeatherMap,
    WeatherApi,
}

impl Default for ProviderName {
    fn default() -> Self {
        ProviderName::OpenWeatherMap
    }
}

impl ProviderName {
    pub fn get_provider_instance(&self, api_key: String) -> Box<dyn Provider> {
        match *self {
            ProviderName::OpenWeatherMap => {
                Box::new(open_weather_map::OpenWeatherApi::new(api_key))
            }
            ProviderName::WeatherApi => Box::new(weather_api::WeatherApi::new(api_key)),
        }
    }
}

pub trait Provider {
    fn get_current_weather(&self, address: &str) -> anyhow::Result<serde_json::Value>;
    fn get_timed_weather(
        &self,
        address: &str,
        date: &str, //Option<i64>,
    ) -> anyhow::Result<serde_json::Value>;
}

// TODO: add 2 more providers
pub mod open_weather_map;
pub mod weather_api;
