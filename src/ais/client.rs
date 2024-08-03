use anyhow::{anyhow, Result};
use async_openai::config::OpenAIConfig;
use async_openai::Client;

pub type OaClient = Client<OpenAIConfig>;
const ENV_OPENAI_API_KEY: &str = "OPENAI_API_KEY";

pub fn new_oa_client() -> Result<OaClient> {
    if std::env::var(ENV_OPENAI_API_KEY).is_ok() {
        Ok(Client::new())
    } else {
        println!("No {ENV_OPENAI_API_KEY} env variable. Please set it.");

        Err(anyhow!("No openai api key in env"))
    }
}
