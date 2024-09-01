use anyhow::{anyhow, Result};
use async_openai::config::OpenAIConfig;
use async_openai::Client;

pub type OaClient = Client<OpenAIConfig>;

const GROQ_API_KEY: &str = "GROQ_API_KEY";
const OPENAI_API_KEY: &str = "OPENAI_API_KEY";
const ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";

#[derive(Debug)]
pub enum ApiEndpoint {
    OpenAI,
    Anthropic,
    Groq,
    Local,
}

pub struct ClientFactory;

impl ClientFactory {
    pub fn create_client(endpoint: &ApiEndpoint) -> Result<OaClient> {
        match endpoint {
            ApiEndpoint::Groq => Self::create_groq_client(),
            ApiEndpoint::OpenAI => Self::create_openai_client(),
            ApiEndpoint::Anthropic => Self::create_anthropic_client(),
            ApiEndpoint::Local => Self::create_local_client(),
        }
    }

    fn create_groq_client() -> Result<OaClient> {
        Self::create_client_with_config(GROQ_API_KEY, "https://api.groq.com/openai/v1")
    }

    fn create_openai_client() -> Result<OaClient> {
        Self::create_client_with_config(OPENAI_API_KEY, "https://api.openai.com/v1")
    }

    fn create_anthropic_client() -> Result<OaClient> {
        Self::create_client_with_config(ANTHROPIC_API_KEY, "https://api.anthropic.com")
    }

    fn create_local_client() -> Result<OaClient> {
        Self::create_client_with_config("", "http://localhost:11434/v1")
    }

    fn create_client_with_config(env_key: &str, api_base: &str) -> Result<OaClient> {
        let api_key =
            std::env::var(env_key).map_err(|_| anyhow!("No {} env variable set", env_key))?;

        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base);

        Ok(Client::with_config(config))
    }
}
