use std::collections::HashMap;

use anyhow::Context;
use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::provider::{Provider, ProviderName};

static APP_NAME: &str = "elastio_task";

#[derive(Parser, Debug)]
#[command(about = "Forecasts and displays present and past weather.")]
struct Application {
    #[command(subcommand)]
    command: InputSubcommand,
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub enum InputSubcommand {
    #[clap(subcommand)]
    Configure(ProviderName),
    Get(SpaceTimeConfig),
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
        let (provider_name, provider_key) =
            if let Some(key) = available_providers.get(&provider_name) {
                (provider_name.to_owned(), key.to_owned())
            } else if let Some((name, key)) = available_providers.iter().next() {
                (name.to_owned(), key.to_owned())
            } else {
                return Err(anyhow::anyhow!(
                    "Failed to retrieve any provider. Check .env file in the current folder"
                ));
            };

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
        let date_time_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").expect(
            "Failed during regular expression initialization. Contact developers for proceeding.",
        );
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
    use super::*;

    #[test]
    fn test_get_available_providers() {
        let available_providers = PromptAgent::get_available_providers().unwrap();
        assert!(available_providers.contains_key(&ProviderName::default()));
    }
}
