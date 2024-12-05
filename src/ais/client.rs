use anyhow::{anyhow, Result};
use async_openai::config::OpenAIConfig;
use async_openai::Client;

pub type OaClient = Client<OpenAIConfig>;
const ENV_API_KEY: &str = "GROQ_API_KEY";

pub fn new_oa_client() -> Result<OaClient> {
    if std::env::var(ENV_API_KEY).is_ok() {
        let config = OpenAIConfig::new()
            .with_api_key(std::env::var(ENV_API_KEY).unwrap())
            .with_api_base("https://api.groq.com/openai/v1");
        Ok(Client::with_config(config))
    } else {
        println!("No {ENV_API_KEY} env variable. Please set it.");

        Err(anyhow!("No api key in env"))
    }
}
