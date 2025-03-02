use std::convert::TryFrom;

use bencher_json::{Jwt, Url};
use serde::{de::DeserializeOwned, Serialize};

use crate::parser::CliBackend;

pub const BENCHER_HOST: &str = "BENCHER_HOST";
pub const BENCHER_API_TOKEN: &str = "BENCHER_API_TOKEN";

#[derive(Debug, Clone)]
pub struct Backend {
    client: bencher_client::BencherClient,
}

#[derive(thiserror::Error, Debug)]
pub enum BackendError {
    #[error("Failed to parse host URL: {0}")]
    ParseHost(url::ParseError),
    #[error("Failed to parse API token: {0}")]
    ParseToken(bencher_json::ValidError),
    #[error("{0}")]
    Client(#[from] bencher_client::ClientError),
}

impl TryFrom<CliBackend> for Backend {
    type Error = BackendError;

    fn try_from(backend: CliBackend) -> Result<Self, Self::Error> {
        let CliBackend {
            host,
            token,
            attempts,
            retry_after,
            strict,
        } = backend;
        let host = map_host(host)?;
        let token = map_token(token)?;
        let client = bencher_client::BencherClient::new(
            host,
            token,
            attempts,
            retry_after,
            Some(strict),
            Some(true),
        );
        Ok(Self { client })
    }
}

fn map_host(host: Option<Url>) -> Result<Option<url::Url>, BackendError> {
    host.map(Into::into)
        .or_else(|| std::env::var(BENCHER_HOST).ok())
        .as_deref()
        .map(std::str::FromStr::from_str)
        .transpose()
        .map_err(BackendError::ParseHost)
}

fn map_token(token: Option<Jwt>) -> Result<Option<Jwt>, BackendError> {
    Ok(if let Some(token) = token {
        Some(token)
    } else if let Ok(env_token) = std::env::var(BENCHER_API_TOKEN) {
        Some(env_token.parse().map_err(BackendError::ParseToken)?)
    } else {
        None
    })
}

impl Backend {
    pub fn log(mut self, log: bool) -> Self {
        self.client.log = log;
        self
    }

    pub async fn send<F, R, T, E>(&self, sender: F) -> Result<serde_json::Value, BackendError>
    where
        F: Fn(bencher_client::Client) -> R,
        R: std::future::Future<
            Output = Result<
                progenitor_client::ResponseValue<T>,
                bencher_client::Error<bencher_client::types::Error>,
            >,
        >,
        T: Serialize,
        E: std::error::Error + Send + Sync + 'static,
        bencher_client::JsonValue: TryFrom<T, Error = E>,
    {
        self.client.send(sender).await.map_err(Into::into)
    }

    pub async fn send_with<F, R, T, Json, E>(&self, sender: F) -> Result<Json, BackendError>
    where
        F: Fn(bencher_client::Client) -> R,
        R: std::future::Future<
            Output = Result<
                progenitor_client::ResponseValue<T>,
                bencher_client::Error<bencher_client::types::Error>,
            >,
        >,
        T: Serialize,
        Json: DeserializeOwned + Serialize + TryFrom<T, Error = E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.client.send_with(sender).await.map_err(Into::into)
    }
}
