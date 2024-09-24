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
    async fn register_resource(
        client: &reqwest::Client,
        url: &reqwest::Url,
        resource: &impl resource::Registerable,
        api_version: &APIVersion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = resource.registration_request(api_version);

        let res = client
            .post(url.clone())
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        if res.status() == StatusCode::OK {
            let mut delete_url = url.clone();
            delete_url
                .path_segments_mut()
                .unwrap()
                .push(resource.registry_path().as_str());

            warn!("Resource already present in API deleting: {}", delete_url);

            client.delete(delete_url).send().await?.error_for_status()?;

            let res = client
                .post(url.clone())
                .json(&request)
                .send()
                .await?
                .error_for_status()?;

            if res.status() == StatusCode::OK {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to register resource after deleting and re-registering",
                )));
            }
        }

        Ok(())
    }

    pub async fn register_resources(
        client: &reqwest::Client,
        model: Arc<Mutex<Model>>,
        registry: Arc<Mutex<Option<NmosMdnsRegistry>>>,
        api_version: &APIVersion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let model = model.lock().await;
        let registry = registry.lock().await.clone().unwrap();

        let base = &registry
            .url
            .join(format!("{}/", api_version.to_string()).as_str())
            .unwrap();

        let resource_url = &base.join("resource").unwrap();

        info!("Attempting to register with {}", resource_url);

        // Register resources in order
        debug!("Registering nodes...");
        for (_, node) in model.nodes().await.iter() {
            Self::register_resource(client, resource_url, node, api_version).await?;
        }
        debug!("Registering devices...");
        for (_, device) in model.devices().await.iter() {
            Self::register_resource(client, resource_url, device, api_version).await?;
        }
        debug!("Registering sources...");
        for (_, source) in model.sources().await.iter() {
            Self::register_resource(client, resource_url, source, api_version).await?;
        }
        debug!("Registering flows...");
        for (_, flow) in model.flows().await.iter() {
            Self::register_resource(client, resource_url, flow, api_version).await?;
        }
        debug!("Registering senders...");
        for (_, sender) in model.senders().await.iter() {
            Self::register_resource(client, resource_url, sender, api_version).await?;
        }
        debug!("Registering receivers...");
        for (_, receiver) in model.receivers().await.iter() {
            Self::register_resource(client, resource_url, receiver, api_version).await?;
        }

        Ok(())
    }
}
