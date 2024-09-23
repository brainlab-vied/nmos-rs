use nmos_schema::is_04::{v1_0_x, v1_3_x};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    resource::{Format, Source},
    version::{is_04::V1_0, is_04::V1_3, APIVersion},
};

use super::{Device, Registerable, ResourceCore, ResourceCoreBuilder};

macro_rules! registration_request {
    ($value:expr, $version:ident) => {
        json!($version::RegistrationapiResourcePostRequest::Variant5(
            $version::RegistrationapiResourcePostRequestHealthVariant5 {
                data: Some($value),
                type_: Some(String::from("flow")),
            }
        ))
    };
}

#[must_use]
pub struct FlowBuilder {
    core: ResourceCoreBuilder,
    format: Format,
    source_id: Uuid,
    device_id: Uuid,
    parents: Vec<Uuid>,
}

impl FlowBuilder {
    pub fn new<S: Into<String>>(label: S, source: &Source, device: &Device) -> Self {
        FlowBuilder {
            core: ResourceCoreBuilder::new(label),
            format: source.format,
            source_id: source.core.id,
            device_id: device.core.id,
            parents: Vec::new(),
        }
    }

    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.core = self.core.description(description);
        self
    }

    pub fn tag<S, V>(mut self, key: S, values: V) -> Self
    where
        S: Into<String>,
        V: IntoIterator<Item = S>,
    {
        self.core = self.core.tag(key, values);
        self
    }

    #[must_use]
    pub fn build(self) -> Flow {
        Flow {
            core: self.core.build(),
            format: self.format,
            source_id: self.source_id,
            device_id: self.device_id,
            parents: self.parents,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Flow {
    pub core: ResourceCore,
    pub format: Format,
    pub source_id: Uuid,
    pub device_id: Uuid,
    pub parents: Vec<Uuid>,
}

impl Flow {
    pub fn builder<S: Into<String>>(label: S, source: &Source, device: &Device) -> FlowBuilder {
        FlowBuilder::new(label, source, device)
    }

    #[must_use]
    pub fn to_json(&self, api: &APIVersion) -> FlowJson {
        match *api {
            V1_0 => FlowJson::V1_0((*self).clone().into()),
            V1_3 => FlowJson::V1_3((*self).clone().into()),
            _ => panic!("Unsupported API"),
        }
    }
}

impl Registerable for Flow {
    fn registry_path(&self) -> String {
        format!("flows/{}", self.core.id)
    }

    fn registration_request(&self, api: &APIVersion) -> serde_json::Value {
        match self.to_json(api) {
            FlowJson::V1_0(json) => registration_request!(json, v1_0_x),
            FlowJson::V1_3(json) => registration_request!(json, v1_3_x),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum FlowJson {
    V1_0(v1_0_x::Flow),
    V1_3(v1_3_x::Flow),
}

impl Into<v1_0_x::Flow> for Flow {
    fn into(self) -> v1_0_x::Flow {
        let parents = self.parents.iter().map(ToString::to_string).collect();

        v1_0_x::Flow {
            id: self.core.id.to_string(),
            version: self.core.version.to_string(),
            label: self.core.label.clone(),
            description: self.core.description.clone(),
            format: self.format.to_string(),
            tags: self.core.tags_json(),
            source_id: self.source_id.to_string(),
            parents,
        }
    }
}

impl Into<v1_3_x::Flow> for Flow {
    fn into(self) -> v1_3_x::Flow {
        let parents = self.parents.iter().map(ToString::to_string).collect();
        let id = self.core.id.to_string();
        let version = self.core.version.to_string();
        let label = self.core.label.clone();
        let description = self.core.description.clone();
        let format = self.format.to_string();
        let tags = self.core.tags_json();
        let source_id = self.source_id.to_string();
        let device_id = self.device_id.to_string();
        match self.format {
            Format::Video => {
                json!(v1_3_x::FlowVideoCoded {
                    id,
                    version,
                    label,
                    description,
                    format,
                    tags,
                    source_id,
                    parents,
                    device_id,
                    grain_rate: None,
                    colorspace: "RGB".into(),
                    frame_height: 640,
                    frame_width: 480,
                    interlace_mode: None,
                    transfer_characteristic: None,
                    media_type: "video/h264".into()
                })
            }
            Format::Audio => {
                let sample_rate = nmos_schema::is_04::v1_3_x::FlowAudioCodedSampleRate {
                    numerator: 44000,
                    denominator: None,
                };
                json!(v1_3_x::FlowAudioCoded {
                    id,
                    version,
                    label,
                    description,
                    format,
                    tags,
                    source_id,
                    parents,
                    device_id,
                    sample_rate,
                    media_type: "audio/ogg".to_string(),
                    grain_rate: None,
                })
            }
            Format::Data => {
                panic!("Data flow not implemented")
            }
        }
    }
}
