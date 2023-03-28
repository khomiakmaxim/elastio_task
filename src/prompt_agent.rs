use std::collections::HashMap;

use anyhow::Context;
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
            let api_pair = std::env::var(provider_name.to_string()).with_context(|| {
                format!("Failed to get api key for {provider_name} provider. Check .env file")
            })?;
            api_pairs.insert(provider_name, api_pair);
        }

        let default_provider_name = ProviderName::default();
        let (provider_name, provider_key) = match api_pairs.get(&default_provider_name) {
            Some(key) => (default_provider_name, key.to_owned()),
            None => {
                let ref_pair = api_pairs.iter().next().unwrap(); // TODO: remove unwrap
                (ref_pair.0.to_owned(), ref_pair.1.to_owned()) // TODO: rewrite this somehow better
            }
        };

        let provider: Box<dyn Provider> = provider_name.get_provider_instance(provider_key);

        let date_time_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}$").unwrap();

        Ok(PromptAgent {
            current_provider: provider,
            available_providers: api_pairs,
            date_time_regex,
        })
    }

    pub fn parse_command() -> clap::error::Result<InputCommand> {
        let mut line = String::default();
        let stdin = std::io::stdin();
        let _ = stdin.read_line(&mut line)?;
        let splitted_line = line.split_whitespace();
        InputCommand::try_parse_from(splitted_line)
    }

    pub fn process_command(&mut self, command: InputCommand) -> anyhow::Result<()> {        
        // TODO: datetime is parsed horribly here
        match command {
            InputCommand::Get(space_time_config) => {
                let timestamp = if let Some(date) = space_time_config.date.clone() {
                    if self.date_time_regex.is_match(&date) {
                        let date =
                            chrono::NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S");
                        match date {
                            Ok(date) => Some(date.timestamp()),
                            Err(err) => {
                                eprintln!("Error: {}\nPlease, enter date in the format yyyy-mm-ddThh:mm:ss", err);
                                None
                            }
                        }
                    } else {
                        eprintln!("Please, enter date in the format yyyy-mm-ddThh:mm:ss");
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
                // TODO: Think of error handling here
                let api_key = self.available_providers.get(&provider_name).unwrap();
                self.current_provider = provider_name.get_provider_instance(api_key.to_owned());
                Ok(())
            }
        }
    }
}
