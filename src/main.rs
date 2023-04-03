use dotenvy::dotenv;

use elastio_task::prompt_agent::PromptAgent;

fn main() {
    dotenv().ok();
    let agent = match PromptAgent::new() {
        Ok(agent) => agent,
        Err(err) => {
            eprintln!(
                "Error: Command crashed during initialization steps: {}. Contact developers for proceeding.",
                err
            );
            std::process::exit(1);
        }
    };

    if let Err(err) = agent.parse_command() {
        eprintln!("Error: {}.", err);
        std::process::exit(1);
    }
}
