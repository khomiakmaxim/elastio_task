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
    current_provider: Box<dyn Provider>,
    current_provider_name: ProviderName,
    available_providers: HashMap<ProviderName, String>,
    date_time_regex: Regex,
}

impl PromptAgent {
    pub fn new() -> anyhow::Result<Self> {
        let mut api_pairs = HashMap::<ProviderName, String>::new();

        for provider_name in ProviderName::iter() {
            let api_key = std::env::var(provider_name.to_string()).with_context(|| {
                format!("Failed to get api key for {} provider. Check .env file in the current folder or contact the developers.", provider_name)
            })?;
            api_pairs.insert(provider_name, api_key);
        }

        let default_provider_name = ProviderName::default();
        let (provider_name, provider_key) = api_pairs
            .get(&default_provider_name)
            .map(|key| (default_provider_name.to_owned(), key.to_owned()))
            .unwrap_or_else(|| {
                api_pairs
                    .iter()
                    .next()
                    .map(|(name, key)| (name.to_owned(), key.to_owned()))
                    .unwrap()
            });

        let provider: Box<dyn Provider> = provider_name.get_provider_instance(provider_key);

        // Will match YYYY-MM-DD. Ex: 2023-03-31
        let date_time_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap(); // TODO: think of 2023-3-31

        println!(
            "Provider {} will be used.",
            provider_name.to_string().to_lowercase().replace('_', "-")
        );

        Ok(PromptAgent {
            current_provider: provider,
            current_provider_name: provider_name,
            available_providers: api_pairs,
            date_time_regex,
        })
    }

    pub fn parse_command(&self) -> Result<InputCommand, clap::Error> {
        let mut line = String::default();
        let stdin = std::io::stdin();
        stdin.read_line(&mut line)?;
        let tokens: Vec<&str> = line.split_whitespace().collect();
        InputCommand::try_parse_from(tokens)
    }

    pub fn process_command(&mut self, command: InputCommand) -> anyhow::Result<()> {
        match command {
            InputCommand::Get(space_time_config) => {
                let timestamp = match space_time_config.date {
                    Some(ref date) if !self.date_time_regex.is_match(date) => {
                        eprintln!("Entered date should be in the YYYY-MM-DD format. Forecast for the current time is retrieved");
                        None
                    }
                    Some(ref date) => {
                        let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|err| {
                            eprintln!("Error: {}\nEntered date should be in the YYYY-MM-DD format. Forecast for the current time is retrieved", err);
                            err
                        })?;
                        let midday_datetime =
                            NaiveDateTime::new(date, NaiveTime::from_hms_opt(12, 0, 0).unwrap());
                        Some(midday_datetime.timestamp())
                    }
                    None => None,
                };

                let weather = self
                    .current_provider
                    .get_weather(timestamp, space_time_config.address)?;
                println!("{}", serde_json::to_string_pretty(&weather)?);
                Ok(())
            }
            InputCommand::Configure(provider_name) => {
                if provider_name == self.current_provider_name {
                    println!(
                        "Provider {} is already in use.",
                        self.current_provider_name
                            .to_string()
                            .to_lowercase()
                            .replace('_', "-")
                    );
                } else {
                    let api_key = self.available_providers.get(&provider_name).expect("Couldn't retrieve api key for current provider. Contact developers for proceeding.");
                    println!(
                        "Changing provider: {} => {}.",
                        self.current_provider_name
                            .to_string()
                            .to_lowercase()
                            .replace('_', "-"),
                        provider_name.to_string().to_lowercase().replace('_', "-")
                    );
                    self.current_provider = provider_name.get_provider_instance(api_key.to_owned());
                }

                Ok(())
            }
        }
    }
}
