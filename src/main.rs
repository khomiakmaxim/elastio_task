use clap::error::ErrorKind;
use dotenvy::dotenv;

use elastio_task::prompt_agent::PromptAgent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let mut agent = PromptAgent::init()?;

    loop {
        let command = PromptAgent::parse_command();

        match command {
            Ok(command) => agent.process_command(command)?,
            Err(err) => match err.kind() {
                ErrorKind::DisplayHelp => {
                    eprintln!("{}", err)
                }
                _ => {
                    eprintln!("Error: {}. Please, see help and try again!", err);
                }
            },
        }
    }
}
