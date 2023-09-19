use std::sync::Arc;

use nmos_model::{resource, Model};
use tracing::info;

use crate::mdns::NmosMdnsRegistry;

pub struct RegistrationApi;

impl RegistrationApi {
    async fn register_node(
        client: &reqwest::Client,
        url: &reqwest::Url,
        node: &resource::Node,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use nmos_model::version::is_04::V1_0;
        use nmos_schema::is_04::v1_0_x::{
            RegistrationapiResourcePostRequest, RegistrationapiResourcePostRequestHealthVariant0,
        };

        // TODO: Must find better way of representing multiple API
        // version in JSON. For now this will look like a mess.
        let resource::NodeJson::V1_0(node_json) = node.to_json(&V1_0);

        // Construct POST request
        let node_post_request = RegistrationapiResourcePostRequestHealthVariant0 {
            data: Some(node_json),
            type_: Some(String::from("node")),
        };
        let post_request = RegistrationapiResourcePostRequest::Variant0(node_post_request);

        client.post(url.clone()).json(&post_request).send().await?.error_for_status()?;

        Ok(())
    }

    async fn register_device(
        client: &reqwest::Client,
        url: &reqwest::Url,
        device: &resource::Device,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use nmos_model::version::is_04::V1_0;
        use nmos_schema::is_04::v1_0_x::{
            RegistrationapiResourcePostRequest, RegistrationapiResourcePostRequestHealthVariant1,
        };

        let resource::DeviceJson::V1_0(device_json) = device.to_json(&V1_0);
        let device_post_request = RegistrationapiResourcePostRequestHealthVariant1 {
            data: Some(device_json),
            type_: Some(String::from("device")),
        };
        let post_request = RegistrationapiResourcePostRequest::Variant1(device_post_request);

        client.post(url.clone()).json(&post_request).send().await?.error_for_status()?;

        Ok(())
    }

    async fn register_source(
        client: &reqwest::Client,
        url: &reqwest::Url,
        source: &resource::Source,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use nmos_model::version::is_04::V1_0;
        use nmos_schema::is_04::v1_0_x::{
            RegistrationapiResourcePostRequest, RegistrationapiResourcePostRequestHealthVariant4,
        };

        let resource::SourceJson::V1_0(source_json) = source.to_json(&V1_0);
        let source_post_request = RegistrationapiResourcePostRequestHealthVariant4 {
            data: Some(source_json),
            type_: Some(String::from("source")),
        };
        let post_request = RegistrationapiResourcePostRequest::Variant4(source_post_request);

        client.post(url.clone()).json(&post_request).send().await?.error_for_status()?;

        Ok(())
    }

    async fn register_flow(
        client: &reqwest::Client,
        url: &reqwest::Url,
        flow: &resource::Flow,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use nmos_model::version::is_04::V1_0;
        use nmos_schema::is_04::v1_0_x::{
            RegistrationapiResourcePostRequest, RegistrationapiResourcePostRequestHealthVariant5,
        };

        let resource::FlowJson::V1_0(flow_json) = flow.to_json(&V1_0);
        let flow_post_request = RegistrationapiResourcePostRequestHealthVariant5 {
            data: Some(flow_json),
            type_: Some(String::from("flow")),
        };
        let post_request = RegistrationapiResourcePostRequest::Variant5(flow_post_request);

        client.post(url.clone()).json(&post_request).send().await?.error_for_status()?;

        Ok(())
    }

    async fn register_sender(
        client: &reqwest::Client,
        url: &reqwest::Url,
        sender: &resource::Sender,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use nmos_model::version::is_04::V1_0;
        use nmos_schema::is_04::v1_0_x::{
            RegistrationapiResourcePostRequest, RegistrationapiResourcePostRequestHealthVariant2,
        };

        let resource::SenderJson::V1_0(sender_json) = sender.to_json(&V1_0);
        let sender_post_request = RegistrationapiResourcePostRequestHealthVariant2 {
            data: Some(sender_json),
            type_: Some(String::from("sender")),
        };
        let post_request = RegistrationapiResourcePostRequest::Variant2(sender_post_request);

        client.post(url.clone()).json(&post_request).send().await?.error_for_status()?;

        Ok(())
    }

    async fn register_receiver(
        client: &reqwest::Client,
        url: &reqwest::Url,
        receiver: &resource::Receiver,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use nmos_model::version::is_04::V1_0;
        use nmos_schema::is_04::v1_0_x::{
            RegistrationapiResourcePostRequest, RegistrationapiResourcePostRequestHealthVariant3,
        };

        let resource::ReceiverJson::V1_0(receiver_json) = receiver.to_json(&V1_0);
        let receiver_post_request = RegistrationapiResourcePostRequestHealthVariant3 {
            data: Some(receiver_json),
            type_: Some(String::from("receiver")),
        };
        let post_request = RegistrationapiResourcePostRequest::Variant3(receiver_post_request);

        client.post(url.clone()).json(&post_request).send().await?.error_for_status()?;

        Ok(())
    }

    pub async fn register_resources(
        client: &reqwest::Client,
        model: Arc<Model>,
        registry: &NmosMdnsRegistry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let base = &registry.url.join("v1.0/").unwrap();

        info!("Attempting to register with {}", base);

        // Resource endpoint
        let resource_url = &base.join("resource").unwrap();

        // Get node
        let nodes = model.nodes().await;
        let node = nodes.iter().next().unwrap().1;

        // Register resources in order
        Self::register_node(client, resource_url, node).await?;
        for (_, device) in model.devices().await.iter() {
            Self::register_device(client, resource_url, device).await?;
        }
        for (_, source) in model.sources().await.iter() {
            Self::register_source(client, resource_url, source).await?;
        }
        for (_, flow) in model.flows().await.iter() {
            Self::register_flow(client, resource_url, flow).await?;
        }
        for (_, sender) in model.senders().await.iter() {
            Self::register_sender(client, resource_url, sender).await?;
        }
        for (_, receiver) in model.receivers().await.iter() {
            Self::register_receiver(client, resource_url, receiver).await?;
        }

        Ok(())
    }
    //
    // pub async fn delete_resource(
    //     client: &reqwest::Client,
    //     resource: &resource::ResourceCore,
    //     registry: &NmosMdnsRegistry)
    //     -> Result<(), Box<dyn std::error::Error>>
    // {
    //     let resource_id = resource.id;
    //
    //     Ok(())
    // }
}
