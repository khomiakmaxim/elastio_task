use std::collections::HashMap;

use anyhow::Context;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::provider::{Provider, ProviderName};

#[derive(clap::Args, Debug, Clone, Serialize, Deserialize)]
pub struct SpaceTimeConfig {
    pub address: String,
    pub date: Option<String>,
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
pub enum InputCommand {
    #[clap(subcommand)]
    Configure(ProviderName),
    Get(SpaceTimeConfig),
}

pub struct PromptAgent {
    pub current_provider: Box<dyn Provider>,
    pub available_providers: HashMap<ProviderName, String>,
    date_time_regex: Regex,
}

impl PromptAgent {
    pub fn init() -> anyhow::Result<Self> {
        let mut api_pairs = HashMap::<ProviderName, String>::new();

        for provider_name in ProviderName::iter() {
            let api_key = std::env::var(provider_name.to_string()).with_context(|| {
                format!("Failed to get api key for {provider_name} provider. Check .env file in the current folder or contact the developers.")
            })?;
            api_pairs.insert(provider_name, api_key);
        }

        let default_provider_name = ProviderName::default();
        let (provider_name, provider_key) = if let Some(key) = api_pairs.get(&default_provider_name)
        {
            (default_provider_name.to_owned(), key.to_owned())
        } else {
            let (key, value) = api_pairs.iter().next().ok_or_else( ||
                anyhow::anyhow!(
                    "Provider's configuration issue"
                )).with_context(|| "There was an issue while retrieving default provider. Please, contact the developers for proceeding.")?;
            (key.to_owned(), value.to_owned())
        };

        let provider: Box<dyn Provider> = provider_name.get_provider_instance(provider_key);

        let date_time_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();

        Ok(PromptAgent {
            current_provider: provider,
            available_providers: api_pairs,
            date_time_regex,
        })
    }

    pub fn parse_command(&self) -> clap::error::Result<InputCommand> {
        let mut line = String::default();
        let stdin = std::io::stdin();
        let _ = stdin.read_line(&mut line)?;
        let splitted_line = line.split_whitespace();
        InputCommand::try_parse_from(splitted_line)
    }

    // This should return some concrete error here
    pub fn process_command(&mut self, command: InputCommand) -> anyhow::Result<()> {
        match command {
            InputCommand::Get(space_time_config) => {
                let timestamp = if let Some(ref date) = space_time_config.date {
                    if self.date_time_regex.is_match(date) {
                        let date = NaiveDate::parse_from_str(date, "%Y-%m-%d");
                        match date {
                            Ok(date) => {
                                let midday_datetime = NaiveDateTime::new(
                                    date,
                                    NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                                );
                                Some(midday_datetime.timestamp())
                            }
                            Err(err) => {
                                eprintln!("Error: {}\nEntered date should be in the YYYY-MM-DD format. Forecast for the current time is retrieved", err);
                                None
                            }
                        }
                    } else {
                        eprintln!("Entered date should be in the YYYY-MM-DD format. Forecast for the current time is retrieved");
                        None
                    }
                } else {
                    None
                };

                let weather = self
                    .current_provider
                    .get_weather(timestamp, space_time_config.address)?;

                println!("{}", serde_json::to_string_pretty(&weather)?);

                Ok(())
            }
            InputCommand::Configure(provider_name) => {
                let api_key = self.available_providers.get(&provider_name).ok_or_else(|| anyhow::anyhow!("Provider's configuration issue")).with_context(|| format!("There was an issue with {} configuration. Please, contact the developers for proceding.", provider_name))?;
                self.current_provider = provider_name.get_provider_instance(api_key.to_owned());
                Ok(())
            }
        }
    }
}
