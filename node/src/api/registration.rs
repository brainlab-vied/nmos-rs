use std::sync::Arc;

use nmos_model::{resource, version::APIVersion, Model};
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::mdns::NmosMdnsRegistry;

pub struct RegistrationApi;

impl RegistrationApi {
    async fn register_node(
        client: &reqwest::Client,
        url: &reqwest::Url,
        node: &resource::Node,
        api_version: &APIVersion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        client
            .post(url.clone())
            .json(&node.registration_request(api_version))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn register_device(
        client: &reqwest::Client,
        url: &reqwest::Url,
        device: &resource::Device,
        api_version: &APIVersion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        client
            .post(url.clone())
            .json(&device.registration_request(api_version))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn register_source(
        client: &reqwest::Client,
        url: &reqwest::Url,
        source: &resource::Source,
        api_version: &APIVersion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        client
            .post(url.clone())
            .json(&source.registration_request(api_version))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn register_flow(
        client: &reqwest::Client,
        url: &reqwest::Url,
        flow: &resource::Flow,
        api_version: &APIVersion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        client
            .post(url.clone())
            .json(&flow.registration_request(api_version))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn register_sender(
        client: &reqwest::Client,
        url: &reqwest::Url,
        sender: &resource::Sender,
        api_version: &APIVersion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        client
            .post(url.clone())
            .json(&sender.registration_request(api_version))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn register_receiver(
        client: &reqwest::Client,
        url: &reqwest::Url,
        receiver: &resource::Receiver,
        api_version: &APIVersion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        client
            .post(url.clone())
            .json(&receiver.registration_request(api_version))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn register_resources(
        client: &reqwest::Client,
        model: Arc<Mutex<Model>>,
        registry: &NmosMdnsRegistry,
        api_version: &APIVersion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let model = model.lock().await;

        let base = &registry
            .url
            .join(format!("{}/", api_version.to_string()).as_str())
            .unwrap();

        info!("Attempting to register with {}", base);

        // Resource endpoint
        let resource_url = &base.join("resource").unwrap();

        // Get node
        let nodes = model.nodes().await;
        let node = nodes.iter().next().unwrap().1;

        // Register resources in order
        debug!("Registering node...");
        Self::register_node(client, resource_url, node, api_version).await?;
        debug!("Registering devices...");
        for (_, device) in model.devices().await.iter() {
            Self::register_device(client, resource_url, device, api_version).await?;
        }
        debug!("Registering sources...");
        for (_, source) in model.sources().await.iter() {
            Self::register_source(client, resource_url, source, api_version).await?;
        }
        debug!("Registering flows...");
        for (_, flow) in model.flows().await.iter() {
            Self::register_flow(client, resource_url, flow, api_version).await?;
        }
        debug!("Registering senders...");
        for (_, sender) in model.senders().await.iter() {
            Self::register_sender(client, resource_url, sender, api_version).await?;
        }
        debug!("Registering receivers...");
        for (_, receiver) in model.receivers().await.iter() {
            Self::register_receiver(client, resource_url, receiver, api_version).await?;
        }

        Ok(())
    }
}
