use std::sync::Arc;

use nmos_model::{resource, version::APIVersion, Model};
use nmos_schema::is_04::{v1_0_x, v1_3_x};
use serde_json::json;
use tokio::sync::Mutex;
use tracing::info;

use crate::mdns::NmosMdnsRegistry;

pub struct RegistrationApi;

macro_rules! api_post_request {
    ($resource:expr, $r_type:expr,$json_enum:ident, $request:ident, $variant:ident) => {
        match $resource {
            resource::$json_enum::V1_0(json) => {
                let request = v1_0_x::$request {
                    data: Some(json),
                    type_: Some(String::from($r_type.to_string())),
                };
                json!(v1_0_x::RegistrationapiResourcePostRequest::$variant(
                    request
                ))
            }
            resource::$json_enum::V1_3(json) => {
                let request = v1_3_x::$request {
                    data: Some(json),
                    type_: Some(String::from($r_type.to_string())),
                };
                json!(v1_3_x::RegistrationapiResourcePostRequest::$variant(
                    request
                ))
            }
        }
    };
}

impl RegistrationApi {
    async fn register_node(
        client: &reqwest::Client,
        url: &reqwest::Url,
        node: &resource::Node,
        api_version: &APIVersion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        client
            .post(url.clone())
            .json(&api_post_request!(
                node.to_json(&api_version),
                "node",
                NodeJson,
                RegistrationapiResourcePostRequestHealthVariant0,
                Variant0
            ))
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
            .json(&api_post_request!(
                device.to_json(&api_version),
                "device",
                DeviceJson,
                RegistrationapiResourcePostRequestHealthVariant1,
                Variant1
            ))
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
            .json(&api_post_request!(
                source.to_json(&api_version),
                "source",
                SourceJson,
                RegistrationapiResourcePostRequestHealthVariant4,
                Variant4
            ))
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
            .json(&api_post_request!(
                flow.to_json(&api_version),
                "flow",
                FlowJson,
                RegistrationapiResourcePostRequestHealthVariant5,
                Variant5
            ))
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
            .json(&api_post_request!(
                sender.to_json(&api_version),
                "sender",
                SenderJson,
                RegistrationapiResourcePostRequestHealthVariant2,
                Variant2
            ))
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
            .json(&api_post_request!(
                receiver.to_json(&api_version),
                "receiver",
                ReceiverJson,
                RegistrationapiResourcePostRequestHealthVariant3,
                Variant3
            ))
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

        info!("All received registries: {:?}", registry);

        info!("Attempting to register with {}", base);

        // Resource endpoint
        let resource_url = &base.join("resource").unwrap();

        // Get node
        let nodes = model.nodes().await;
        let node = nodes.iter().next().unwrap().1;

        // Register resources in order
        info!("Registering node...");
        Self::register_node(client, resource_url, node, api_version).await?;
        info!("Registering devices...");
        for (_, device) in model.devices().await.iter() {
            Self::register_device(client, resource_url, device, api_version).await?;
        }
        info!("Registering sources...");
        for (_, source) in model.sources().await.iter() {
            Self::register_source(client, resource_url, source, api_version).await?;
        }
        info!("Registering flows...");
        for (_, flow) in model.flows().await.iter() {
            Self::register_flow(client, resource_url, flow, api_version).await?;
        }
        info!("Registering senders...");
        for (_, sender) in model.senders().await.iter() {
            Self::register_sender(client, resource_url, sender, api_version).await?;
        }
        info!("Registering receivers...");
        for (_, receiver) in model.receivers().await.iter() {
            Self::register_receiver(client, resource_url, receiver, api_version).await?;
        }

        Ok(())
    }
}
