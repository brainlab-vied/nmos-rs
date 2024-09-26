use std::{collections::BinaryHeap, net::SocketAddr, sync::Arc, thread, time::Duration};

use axum::{http::Method, Server};
use mdns::MdnsContext;
use nmos_model::{
    resource::{Registerable, ResourceBundle},
    Model,
};
use reqwest::StatusCode;
use tokio::{
    runtime::Runtime,
    sync::{mpsc, Mutex},
};
use tower::{make::Shared, ServiceBuilder};
use tower_http::cors::{self, CorsLayer};
use tracing::{debug, error, info, warn};

use nmos_model::version::is_04::V1_3;
use nmos_model::version::APIVersion;

mod api;
mod error;
mod mdns;

pub use error::Error as NmosError;

use api::{NodeApi, RegistrationApi};
use mdns::{NmosMdnsConfig, NmosMdnsEvent, NmosMdnsRegistry};

#[must_use]
pub struct NodeBuilder {
    model: Arc<Model>,
    address: SocketAddr,
    api_version: APIVersion,
    event_channel: mpsc::UnboundedReceiver<ResourceUpdate>,
    heartbeat_interval: u64,
    registry_timeout: u64,
}

impl NodeBuilder {
    pub fn new(model: Model) -> Self {
        Self {
            model: Arc::new(model),
            address: ([0, 0, 0, 0], 3000).into(),
            api_version: V1_3,
            event_channel: mpsc::unbounded_channel::<ResourceUpdate>().1,
            heartbeat_interval: 5,
            registry_timeout: 5,
        }
    }

    pub fn from_resources(resource_bundle: ResourceBundle) -> Self {
        Self {
            model: Arc::new(Model::from_resources(resource_bundle)),
            address: ([0, 0, 0, 0], 3000).into(),
            api_version: V1_3,
            event_channel: mpsc::unbounded_channel::<ResourceUpdate>().1,
            heartbeat_interval: 5,
            registry_timeout: 5,
        }
    }

    pub fn with_addr(mut self, address: SocketAddr) -> Self {
        self.address = address;
        self
    }

    pub fn with_api_version(mut self, api_version: APIVersion) -> Self {
        self.api_version = api_version;
        self
    }

    pub fn with_registration_timeout(mut self, timeout: u64) -> Self {
        self.registry_timeout = timeout;
        self
    }

    pub fn with_heartbeat_interval(mut self, interval: u64) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    pub fn with_event_channel(mut self, channel: mpsc::UnboundedReceiver<ResourceUpdate>) -> Self {
        self.event_channel = channel;
        self
    }

    pub fn build(self) -> Node {
        // Make service
        let service = NodeApi::new(self.model.clone());

        Node {
            model: self.model,
            service,
            address: self.address,
            api_version: self.api_version,
            current_registry: Arc::new(Mutex::new(None)),
            event_channel: self.event_channel,
            registry_timeout: self.registry_timeout,
            heartbeat_interval: self.heartbeat_interval,
        }
    }
}

pub enum ResourceUpdate {
    Update(Box<dyn Registerable>),
    Added(Box<dyn Registerable>),
    Removed(Box<dyn Registerable>),
}

pub struct Node {
    model: Arc<Model>,
    service: NodeApi,
    address: SocketAddr,
    api_version: APIVersion,
    current_registry: Arc<Mutex<Option<NmosMdnsRegistry>>>,
    event_channel: mpsc::UnboundedReceiver<ResourceUpdate>,
    heartbeat_interval: u64,
    registry_timeout: u64,
}

impl Node {
    pub fn builder(model: Model) -> NodeBuilder {
        NodeBuilder::new(model)
    }

    pub fn builder_from_resources(resource_bundle: ResourceBundle) -> NodeBuilder {
        NodeBuilder::from_resources(resource_bundle)
    }

    #[must_use]
    pub fn model(&self) -> Arc<Model> {
        self.model.clone()
    }

    pub async fn start(mut self) -> error::Result<()> {
        info!("Starting nmos-rs node");

        // Channel for receiving MDNS events
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Keep discovered registries in a priority queue
        let registries = Arc::new(Mutex::new(BinaryHeap::new()));

        // MDNS must run on its own thread
        // Events are sent back to the Tokio runtime
        thread::spawn(move || {
            // Create context
            let mut context = MdnsContext::new(&NmosMdnsConfig {}, tx.clone());
            let poller = context.start();

            loop {
                // Check event channel is still valid
                if tx.is_closed() {
                    break;
                }

                // Poll every 100 ms
                poller.poll();
                thread::sleep(Duration::from_millis(100));
            }
        });

        // Receive MDNS events in "main thread"
        let mdns_receiver = async {
            let registries = registries.clone();

            while let Some(event) = rx.recv().await {
                if let NmosMdnsEvent::Discovery(_, Ok(discovery)) = event {
                    if let Some(registry) = NmosMdnsRegistry::parse(&discovery, &self.api_version) {
                        let mut registries = registries.lock().await;
                        debug!(
                            "Discovered registry url: {} version: {:?} priority: {}",
                            registry.url,
                            registry
                                .api_ver
                                .iter()
                                .map(APIVersion::to_string)
                                .collect::<Vec<_>>(),
                            registry.pri
                        );
                        registries.push(registry);
                    }
                }
            }
            error!("mDNS discovery unexpectedly finished when it should not.");
        };

        // Create HTTP service
        let app = ServiceBuilder::new()
            .layer(
                CorsLayer::new()
                    .allow_methods([Method::GET, Method::POST])
                    .allow_origin(cors::Any),
            )
            .service(self.service);

        let http_server = Server::bind(&self.address).serve(Shared::new(app));

        // Registry connection thread
        let registration = async {
            // Create http client
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(self.registry_timeout))
                .build()
                .unwrap();

            loop {
                // Wait for registry discovery
                tokio::time::sleep(Duration::from_secs(5)).await;

                {
                    let mut registry = self.current_registry.lock().await;

                    // Try and get highest priority registry
                    *registry = {
                        let mut registries = registries.lock().await;
                        registries.pop()
                    };

                    // Don't register and heartbeat if no registry is available
                    if registry.is_none() {
                        continue;
                    }
                }

                // Attempt to register
                match RegistrationApi::register_resources(
                    &client,
                    self.model.clone(),
                    self.current_registry.clone(),
                    &self.api_version,
                )
                .await
                {
                    Ok(_) => info!("Registration successful"),
                    Err(err) => {
                        error!("Failed to register with registry: {}", err);
                        continue;
                    }
                }

                // Get heartbeat endpoint from node id
                let heartbeat_url = {
                    let nodes = self.model.nodes().await;
                    let node_id = *nodes.iter().next().unwrap().0;
                    let registry = self.current_registry.lock().await.clone().unwrap();

                    registry
                        .url
                        .join(&format!("{}/", self.api_version)) // Ensure it ends with a '/'
                        .unwrap()
                        .join(&format!("health/nodes/{}", node_id))
                        .unwrap()
                };

                let mut first_attempt = true;
                // Send heartbeat every 5 seconds
                loop {
                    debug!("Heart-beating to {}", heartbeat_url);
                    match client.post(heartbeat_url.clone()).send().await {
                        Ok(res) => {
                            if !res.status().is_success() {
                                if res.status() == StatusCode::NOT_FOUND && first_attempt {
                                    match RegistrationApi::register_resources(
                                        &client,
                                        self.model.clone(),
                                        self.current_registry.clone(),
                                        &self.api_version,
                                    )
                                    .await
                                    {
                                        Ok(_) => {
                                            first_attempt = false;
                                            continue;
                                        }
                                        Err(_) => break,
                                    }
                                }
                                error!("Heartbeat error {}", res.status());
                                break;
                            }
                            info!("Heartbeat successful!");
                        }
                        Err(err) => {
                            error!("Failed to send heartbeat: {}", err);
                            break;
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(self.heartbeat_interval)).await;
                }
            }
        };

        let update = async {
            // Create http client
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(self.registry_timeout))
                .build()
                .unwrap();

            while let Some(event) = self.event_channel.recv().await {
                if let Some(reg) = self.current_registry.lock().await.clone() {
                    let base = &reg
                        .url
                        .join(format!("{}/", self.api_version.to_string()).as_str())
                        .unwrap();

                    let url = &base.join("resource").unwrap();

                    let res: Result<reqwest::Response, Box<dyn std::error::Error>> = match event {
                        ResourceUpdate::Update(resource) => {
                            RegistrationApi::post_resource(
                                &client,
                                &url,
                                resource.as_ref(),
                                &self.api_version,
                            )
                            .await
                        }
                        ResourceUpdate::Added(resource) => {
                            RegistrationApi::register_resource(
                                &client,
                                &url,
                                resource.as_ref(),
                                &self.api_version,
                            )
                            .await
                        }
                        ResourceUpdate::Removed(resource) => {
                            RegistrationApi::delete_resource(&client, &url, resource.as_ref()).await
                        }
                    };
                    match res {
                        Ok(response) => debug!("{:?}", response),
                        Err(err) => warn!("{}", err),
                    }
                } else {
                    warn!("No registry available!");
                }
            }
        };

        tokio::select! {
            _ = mdns_receiver => {}
            _ = http_server => {}
            _ = registration => {}
            _ = update => {}
        };

        error!("Program shouldn't reach this part!");

        Ok(())
    }

    pub fn start_blocking(self) -> error::Result<()> {
        let rt = Runtime::new().expect("Unable to create Tokio runtime");
        rt.block_on(self.start())
    }
}
