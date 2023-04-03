//! Module for performing specific API requests. Scales for new providers.
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// General provider trait, used in dynamic dispatch
pub trait Provider {
    /// Trait method for retrieving weather, which is currently at the 'address', which is specified    
    fn get_current_weather(&self, address: &str) -> anyhow::Result<String>;
    /// Trait method for retrieving weather, which was\will be at the 'address', which is specified and on the 'date', which is also specified    
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
/// Enumeration which represents a set of possible providers and which also provides functionality for creating dynamically dispatched providers.
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
    /// Returns a dynamically dispatched instance of a provider that implements the `Provider` trait, based on the `ProviderName` variant and the respective `api_key`.
    pub fn get_provider_instance(&self, api_key: String) -> Box<dyn Provider> {
        match *self {
            ProviderName::OpenWeatherMap => {
                Box::new(open_weather_map::OpenWeatherMap::new(api_key))
            }
            ProviderName::WeatherApi => Box::new(weather_api::WeatherApi::new(api_key)),
        }
    }
    
    /// Returns a pretty name of encoded 'ProviderName' in .env file.
    /// 
    /// # Examples
    /// ```
    /// use elastio_task::provider::ProviderName;
    /// 
    /// let provider_name = ProviderName::OpenWeatherMap;
    /// assert_eq!(provider_name.to_string(), "OPEN_WEATHER_MAP");
    /// assert_eq!(provider_name.get_pretty_name(), "open-weather-map");
    /// ```
    pub fn get_pretty_name(&self) -> String {
        self.to_string().to_ascii_lowercase().replace('_', "-")
    }
}

pub mod open_weather_map;
pub mod weather_api;
