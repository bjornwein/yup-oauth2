use crate::error::Error;
use crate::types::TokenInfo;

/// Provide options for the Application Default Credential Flow, mostly used for testing
pub struct ApplicationDefaultCredentialsFlowOpts {
    /// Used as base to build the url during token request from GCP metadata server
    pub metadata_url: Option<String>,
}
impl Default for ApplicationDefaultCredentialsFlowOpts {
    fn default() -> Self {
        Self { metadata_url: None }
    }
}

pub struct ApplicationDefaultCredentialsFlow {
    metadata_url: String,
}

impl ApplicationDefaultCredentialsFlow {
    pub(crate) fn new(opts: ApplicationDefaultCredentialsFlowOpts) -> Self {
        let metadata_url = opts.metadata_url.unwrap_or_else(|| "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token".to_string());
        ApplicationDefaultCredentialsFlow { metadata_url }
    }

    pub(crate) async fn token<C, T>(
        &self,
        hyper_client: &hyper::Client<C>,
        scopes: &[T],
    ) -> Result<TokenInfo, Error>
    where
        T: AsRef<str>,
        C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        let scope = crate::helper::join(scopes, ",");
        let token_uri = format!("{}?scopes={}", self.metadata_url, scope);
        let request = hyper::Request::get(token_uri)
            .header("Metadata-Flavor", "Google")
            .body(hyper::Body::from(String::new())) // why body is needed?
            .unwrap();
        log::debug!("requesting token from metadata server: {:?}", request);
        let (head, body) = hyper_client.request(request).await?.into_parts();
        let body = hyper::body::to_bytes(body).await?;
        log::debug!("received response; head: {:?}, body: {:?}", head, body);
        TokenInfo::from_json(&body)
    }
}

// eof
