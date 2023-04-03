use std::collections::HashMap;

use anyhow::Context;
use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::provider::{Provider, ProviderName};

static APP_NAME: &str = "ELASTIO_TASK";

#[derive(Parser, Debug)]
#[command(about = "Forecasts and displays present and past weather.")]
struct Application {
    #[command(subcommand)]
    command: InputSubcommand,
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub enum InputSubcommand {
    /// Configures provider for retrieving weather data.
    /// Example: configure weather-api (open-weather-map is default)
    #[clap(subcommand)]
    Configure(ProviderName),
    /// Gets apporpriate weather data, based on address and date(YYYY-MM-DD), if provided, and current weather, if not.
    /// Example: get "L'aquila, Italy" 2023-04-07
    Get(SpaceTimeConfig),
    /// Displays currently used provider    
    CurrentProvider,
}

#[derive(clap::Args, Debug, Clone, Serialize, Deserialize)]
pub struct SpaceTimeConfig {
    pub address: String,
    pub date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct ApplicationConfig {
    provider_name: ProviderName,
}
/// Entity, which is responsible for managing provider's and users communication
pub struct PromptAgent {
    current_provider: Box<dyn Provider>,
    current_provider_name: ProviderName,
}

impl PromptAgent {
    pub fn new() -> anyhow::Result<Self> {
        let config: Result<ApplicationConfig, confy::ConfyError> = confy::load(APP_NAME, None);

        let provider_name = match config {
            Ok(config) => config.provider_name,
            Err(err) => return Err(anyhow::anyhow!("Failed to retrieve config: {}", err)),
        };

        let available_providers = Self::get_available_providers()?;
        let provider_key = available_providers
            .get(&provider_name)
            .expect("Couldn't retrieve required api_key")
            .to_owned();

        let provider: Box<dyn Provider> = provider_name.get_provider_instance(provider_key);

        Ok(PromptAgent {
            current_provider: provider,
            current_provider_name: provider_name,
        })
    }

    pub fn parse_command(&self) -> anyhow::Result<()> {
        let command = Application::parse();
        self.process_command(command)
    }

    fn process_command(&self, command: Application) -> anyhow::Result<()> {
        let date_time_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$")
            .expect("Failed during regular expression initialization");
        match command.command {
            InputSubcommand::Get(space_time_config) => match space_time_config.date {
                Some(ref date) if !date_time_regex.is_match(date) => Err(anyhow::anyhow!(
                    "Entered date should be in the YYYY-MM-DD format"
                )),
                Some(ref date) => {
                    let weather = self
                        .current_provider
                        .get_timed_weather(&space_time_config.address, date)?;

                    println!(
                        "-- Weather for {} on {}: \n{}",
                        &space_time_config.address, date, weather
                    );

                    Ok(())
                }
                None => {
                    let weather = self
                        .current_provider
                        .get_current_weather(&space_time_config.address)?;

                    println!(
                        "-- Current weather for {}: \n{}",
                        &space_time_config.address, weather
                    );

                    Ok(())
                }
            },
            InputSubcommand::Configure(provider_name) => {
                if provider_name == self.current_provider_name {
                    println!(
                        "-- Provider {} is already in use.",
                        self.current_provider_name.get_pretty_name()
                    );
                } else {
                    println!(
                        "-- Changing provider: {} => {}.",
                        self.current_provider_name.get_pretty_name(),
                        provider_name.get_pretty_name()
                    );

                    match confy::store(APP_NAME, None, ApplicationConfig { provider_name }) {
                        Ok(_) => {
                            println!("-- Provider was successfully changed.");
                        }
                        Err(err) => {
                            return Err(anyhow::anyhow!(
                                "There was an issue while updating configuration file: {}",
                                err
                            ));
                        }
                    }
                }

                Ok(())
            }
            InputSubcommand::CurrentProvider => {
                println!(
                    "-- Current provider: {}.",
                    self.current_provider_name.get_pretty_name()
                );

                Ok(())
            }
        }
    }

    fn get_available_providers() -> anyhow::Result<HashMap<ProviderName, String>> {
        let mut available_providers = HashMap::<ProviderName, String>::new();

        for provider_name in ProviderName::iter() {
            let api_key = std::env::var(provider_name.to_string()).with_context(|| {
                format!(
                    "Failed to get api key for {} provider. Check .env file in the current folder",
                    provider_name
                )
            })?;
            available_providers.insert(provider_name, api_key);
        }

        Ok(available_providers)
    }
}

#[cfg(test)]
mod test {
    use chrono::{Duration, Utc};
    use dotenvy::dotenv;    

    use super::*;

    #[test]
    fn test_get_available_providers() {
        dotenv().ok();
        let available_providers = PromptAgent::get_available_providers().unwrap();
        for provider in ProviderName::iter() {
            assert!(available_providers.contains_key(&provider));
        }
    }

    #[test]
    #[ignore]
    fn test_parse_command_get_current_weather() {
        dotenv().ok();
        let agent = PromptAgent::new().unwrap();
        let space_time_config = SpaceTimeConfig {
            address: String::from("L'aquila, Italy"),
            date: None,
        };

        let result = agent.process_command(Application {
            command: InputSubcommand::Get(space_time_config),
        });
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_process_command_get_timed_tommorow_weather() {
        dotenv().ok();
        let agent = PromptAgent::new().unwrap();

        let now = Utc::now();
        let tomorrow = now + Duration::days(1);
        let formatted_tomorrow = tomorrow.format("%Y-%m-%d");

        let space_time_config = SpaceTimeConfig {
            address: String::from("Palermo, Italy"),
            date: Some(formatted_tomorrow.to_string()),
        };
        let result = agent.process_command(Application {
            command: InputSubcommand::Get(space_time_config),
        });
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_process_command_get_timed_yesterday_weather() {
        dotenv().ok();
        let agent = PromptAgent::new().unwrap();

        let now = Utc::now();
        let yesterday = now - Duration::days(1);
        let formatted_yesterday = yesterday.format("%Y-%m-%d");

        let space_time_config = SpaceTimeConfig {
            address: String::from("Palermo, Italy"),
            date: Some(formatted_yesterday.to_string()),
        };
        let result = agent.process_command(Application {
            command: InputSubcommand::Get(space_time_config),
        });
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_process_invalid_address() {
        dotenv().ok();
        let agent = PromptAgent::new().unwrap();

        let space_time_config = SpaceTimeConfig {
            address: String::from("SO INVALID ADDRESS"),
            date: None,
        };

        let result = agent.process_command(Application {
            command: InputSubcommand::Get(space_time_config),
        });

        assert!(result.is_err());
    }

    #[test]
    #[ignore]
    fn test_process_invalid_date() {
        dotenv().ok();
        let agent = PromptAgent::new().unwrap();

        let space_time_config = SpaceTimeConfig {
            address: String::from("São Paulo"),
            date: Some(String::from("1800-12-12")),
        };

        let result = agent.process_command(Application {
            command: InputSubcommand::Get(space_time_config),
        });

        assert!(result.is_err());
    }

    #[test]    
    fn test_process_command_configure() {
        dotenv().ok();
        let agent = PromptAgent::new().unwrap();
        let current_provider = agent.current_provider_name.clone();

        let result = agent.process_command(Application {
            command: InputSubcommand::Configure(ProviderName::OpenWeatherMap),
        });
        assert!(result.is_ok());        

        let result = agent.process_command(Application {
            command: InputSubcommand::Configure(current_provider),
        });
        assert!(result.is_ok());
    }

    #[test]    
    fn test_process_command_current_provider() {
        dotenv().ok();
        let agent = PromptAgent::new().unwrap();
        let result = agent.process_command(Application {
            command: InputSubcommand::CurrentProvider,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_command_with_invalid_date_format() {
        dotenv().ok();
        let agent = PromptAgent::new().unwrap();

        let space_time_config = SpaceTimeConfig {
            address: String::from("São Paulo"),
            date: Some(String::from("2000-12-32")),
        };

        let result = agent.process_command(Application {
            command: InputSubcommand::Get(space_time_config),
        });

        assert!(result.is_err());
    }

}
