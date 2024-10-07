use std::sync::Arc;

use nmos_model::{
    resource::{self},
    version::APIVersion,
    Model,
};
use reqwest::StatusCode;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::mdns::NmosMdnsRegistry;

pub struct RegistrationApi;

impl RegistrationApi {
    pub async fn post_resource(
        client: &reqwest::Client,
        url: &reqwest::Url,
        resource: &dyn resource::Registerable,
        api_version: &APIVersion,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let request = resource.registration_request(api_version);

        Ok(client
            .post(url.clone())
            .json(&request)
            .send()
            .await?
            .error_for_status()?)
    }

    pub async fn delete_resource(
        client: &reqwest::Client,
        url: &reqwest::Url,
        resource: &dyn resource::Registerable,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let delete_url = url
            .clone()
            .join(format!("resource/{}", resource.registry_path()).as_str())
            .unwrap();

        Ok(client.delete(delete_url).send().await?.error_for_status()?)
    }

    pub async fn register_resource(
        client: &reqwest::Client,
        url: &reqwest::Url,
        resource: &dyn resource::Registerable,
        api_version: &APIVersion,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let res = Self::post_resource(client, url, resource, api_version).await?;

        if res.status() == StatusCode::OK {
            warn!(
                "Resource already present in API deleting: {}",
                resource.registry_path()
            );

            let res = Self::delete_resource(client, url, resource).await?;

            if res.status() == StatusCode::OK {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to register resource after deleting and re-registering",
                )));
            } else {
                return Ok(res);
            }
        }

        Ok(res)
    }

    pub async fn register_resources(
        client: &reqwest::Client,
        model: Arc<Model>,
        registry: Arc<Mutex<Option<NmosMdnsRegistry>>>,
        api_version: &APIVersion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let registry = registry.lock().await.clone().unwrap();

        let base = &registry
            .url
            .join(format!("{}/", api_version).as_str())
            .unwrap();

        let resource_url = &base.join("resource").unwrap();

        info!("Attempting to register with {}", resource_url);

        // Register resources in order
        debug!("Registering nodes...");
        for node in model.nodes().await.values() {
            Self::register_resource(client, resource_url, node, api_version).await?;
        }
        debug!("Registering devices...");
        for device in model.devices().await.values() {
            Self::register_resource(client, resource_url, device, api_version).await?;
        }
        debug!("Registering sources...");
        for source in model.sources().await.values() {
            Self::register_resource(client, resource_url, source, api_version).await?;
        }
        debug!("Registering flows...");
        for flow in model.flows().await.values() {
            Self::register_resource(client, resource_url, flow, api_version).await?;
        }
        debug!("Registering senders...");
        for sender in model.senders().await.values() {
            Self::register_resource(client, resource_url, sender, api_version).await?;
        }
        debug!("Registering receivers...");
        for receiver in model.receivers().await.values() {
            Self::register_resource(client, resource_url, receiver, api_version).await?;
        }

        Ok(())
    }
}
