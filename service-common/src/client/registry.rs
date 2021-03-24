use reqwest::{Client, Response, StatusCode};
use url::Url;

use drogue_cloud_service_api::auth::ClientError;

use drogue_cloud_service_api::management;

/// An device registry client backed by reqwest.
#[derive(Clone, Debug)]
pub struct RegistryClient {
    client: Client,
    device_registry_url: Url,
}

impl RegistryClient {
    /// Create a new client instance.
    pub fn new(client: Client, url: Url) -> Self {
        Self {
            client,
            device_registry_url: url,
        }
    }

    pub async fn get_device(
        &self,
        application: &str,
        device: &str,
        token: &str,
    ) -> Result<management::Device, ClientError<reqwest::Error>> {
        let req = self.client.get(
            self.device_registry_url
                .join(&format!("/api/v1/apps/{}/devices/{}", application, device))?,
        );
        let req = req.bearer_auth(token);

        let response: Response = req.send().await.map_err(|err| {
            log::warn!("Error while accessing registry: {}", err);
            Box::new(err)
        })?;

        match response.status() {
            StatusCode::OK => match response.json::<management::Device>().await {
                Ok(result) => Ok(result),
                Err(err) => {
                    log::debug!(
                        "Registry lookup failed for {:?}/{:?}. Result: {:?}",
                        application,
                        device,
                        err
                    );

                    Err(ClientError::Request(format!(
                        "Failed to decode service response: {}",
                        err
                    )))
                }
            },
            StatusCode::NOT_FOUND => Err(ClientError::Request("Device Not Found".to_string())),
            code => Err(ClientError::Request(format!("Unexpected code {:?}", code))),
        }
    }
}
