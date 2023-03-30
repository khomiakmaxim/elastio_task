use anyhow::Context;
use clap::error::ErrorKind;
use dotenvy::dotenv;

use elastio_task::prompt_agent::PromptAgent;

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let mut agent = PromptAgent::new().context(
        "There was an issue during elastio_task initialization. Contact developers for proceeding.",
    )?;

    loop {
        let command = agent.parse_command();

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
