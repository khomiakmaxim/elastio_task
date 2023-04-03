use serde::{Deserialize, Serialize};
use strum::EnumIter;

///To add another provider, do the following:
/// 1. Add and declare in the bottom of this file module with valid Provider trait implementation
/// 2. Add representative variant in ProviderName enumeration
/// 3. Add appropriate match expression hand in get_provider_instance() method in the current file
/// 4. Add YOUR_NEW_PROVIDER_API_NAME={api_key} line in the .env file, preserving case and style for YOUR_NEW_PROVIDER_API_NAME
pub trait Provider {
    fn get_current_weather(&self, address: &str) -> anyhow::Result<String>;
    fn get_timed_weather(&self, address: &str, date: &str) -> anyhow::Result<String>;
}

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
                Box::new(open_weather_map::OpenWeatherMap::new(api_key))
            }
            ProviderName::WeatherApi => Box::new(weather_api::WeatherApi::new(api_key)),
        }
    }

    pub fn get_pretty_name(&self) -> String {
        self.to_string().to_ascii_lowercase().replace('_', "-")
    }
}

pub mod open_weather_map;
pub mod weather_api;
