use dotenvy::dotenv;

use elastio_task::prompt_agent::PromptAgent;

fn main() {
    dotenv().ok();
    let agent = match PromptAgent::new() {
        Ok(agent) => agent,
        Err(err) => {
            eprintln!(
                "Error: Command crashed during initalization steps: {}. Contact developers for proceeding.",
                err
            );
            return;
        }
    };

    if let Err(err) = agent.parse_command() {
        eprintln!("Error: {}.", err);
    }
}
