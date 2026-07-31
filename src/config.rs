use std::env;

use reqwest::Url;

use crate::{Error, Result};

pub const DEFAULT_RUNPOD_API_BASE: &str = "https://rest.runpod.io/v1/";

#[derive(Clone)]
pub struct Config {
    runpod_api_key: String,
    runpod_api_base: Url,
    credential_source: &'static str,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let account_key = env::var("RUNPOD_ACCOUNT_API_KEY").ok();
        let pod_or_local_key = env::var("RUNPOD_API_KEY").ok();
        let api_base = env::var("FERN_RUNPOD_API_BASE").ok();

        Self::from_values(account_key, pod_or_local_key, api_base)
    }

    pub fn from_values(
        account_key: Option<String>,
        pod_or_local_key: Option<String>,
        api_base: Option<String>,
    ) -> Result<Self> {
        let (runpod_api_key, credential_source) =
            match account_key.filter(|value| !value.trim().is_empty()) {
                Some(value) => (value, "RUNPOD_ACCOUNT_API_KEY"),
                None => match pod_or_local_key.filter(|value| !value.trim().is_empty()) {
                    Some(value) => (value, "RUNPOD_API_KEY"),
                    None => {
                        return Err(Error::Config(
                            "set RUNPOD_ACCOUNT_API_KEY or RUNPOD_API_KEY".into(),
                        ));
                    }
                },
            };

        let mut api_base = api_base.unwrap_or_else(|| DEFAULT_RUNPOD_API_BASE.to_owned());
        if !api_base.ends_with('/') {
            api_base.push('/');
        }

        let runpod_api_base = Url::parse(&api_base)?;
        if runpod_api_base.scheme() != "https" && !is_local_test_url(&runpod_api_base) {
            return Err(Error::Config(
                "FERN_RUNPOD_API_BASE must use HTTPS (HTTP is allowed only for localhost tests)"
                    .into(),
            ));
        }

        Ok(Self {
            runpod_api_key,
            runpod_api_base,
            credential_source,
        })
    }

    pub fn runpod_api_key(&self) -> &str {
        &self.runpod_api_key
    }

    pub fn runpod_api_base(&self) -> &Url {
        &self.runpod_api_base
    }

    pub fn credential_source(&self) -> &'static str {
        self.credential_source
    }
}

fn is_local_test_url(url: &Url) -> bool {
    matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_key_has_precedence() {
        let config =
            Config::from_values(Some("account-key".into()), Some("pod-key".into()), None).unwrap();

        assert_eq!(config.runpod_api_key(), "account-key");
        assert_eq!(config.credential_source(), "RUNPOD_ACCOUNT_API_KEY");
    }

    #[test]
    fn pod_or_local_key_is_the_fallback() {
        let config = Config::from_values(None, Some("local-key".into()), None).unwrap();

        assert_eq!(config.runpod_api_key(), "local-key");
        assert_eq!(config.credential_source(), "RUNPOD_API_KEY");
    }

    #[test]
    fn missing_key_is_rejected() {
        let error = Config::from_values(None, None, None).err().unwrap();
        assert!(error.to_string().contains("RUNPOD_API_KEY"));
    }

    #[test]
    fn base_url_is_normalized() {
        let config = Config::from_values(
            Some("key".into()),
            None,
            Some("http://localhost:1234/v1".into()),
        )
        .unwrap();

        assert_eq!(
            config.runpod_api_base().as_str(),
            "http://localhost:1234/v1/"
        );
    }

    #[test]
    fn insecure_remote_url_is_rejected() {
        let error = Config::from_values(
            Some("key".into()),
            None,
            Some("http://example.com/v1".into()),
        )
        .err()
        .unwrap();

        assert!(error.to_string().contains("must use HTTPS"));
    }
}
