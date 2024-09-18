use nmos_schema::is_04;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    resource::{Format, Source},
    version::{is_04::V1_0, is_04::V1_3, APIVersion},
};

use super::{ResourceCore, ResourceCoreBuilder};

#[must_use]
pub struct FlowBuilder {
    core: ResourceCoreBuilder,
    format: Format,
    source_id: Uuid,
    parents: Vec<Uuid>,
}

impl FlowBuilder {
    pub fn new<S: Into<String>>(label: S, source: &Source) -> Self {
        FlowBuilder {
            core: ResourceCoreBuilder::new(label),
            format: source.format,
            source_id: source.core.id,
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
            parents: self.parents,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Flow {
    pub core: ResourceCore,
    pub format: Format,
    pub source_id: Uuid,
    pub parents: Vec<Uuid>,
}

impl Flow {
    pub fn builder<S: Into<String>>(label: S, source: &Source) -> FlowBuilder {
        FlowBuilder::new(label, source)
    }

    #[must_use]
    pub fn to_json(&self, api: &APIVersion) -> FlowJson {
        match *api {
            V1_0 => {
                let parents = self.parents.iter().map(ToString::to_string).collect();

                FlowJson::V1_0(is_04::v1_0_x::Flow {
                    id: self.core.id.to_string(),
                    version: self.core.version.to_string(),
                    label: self.core.label.clone(),
                    description: self.core.description.clone(),
                    format: self.format.to_string(),
                    tags: self.core.tags_json(),
                    source_id: self.source_id.to_string(),
                    parents,
                })
            }
            V1_3 => FlowJson::V1_3((*self).clone().into()),
            _ => panic!("Unsupported API"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum FlowJson {
    V1_0(is_04::v1_0_x::Flow),
    V1_3(is_04::v1_3_x::Flow),
}

impl Into<is_04::v1_3_x::Flow> for Flow {
    fn into(self) -> is_04::v1_3_x::Flow {
        let parents = self.parents.iter().map(ToString::to_string).collect();
        let id = self.core.id.to_string();
        let version = self.core.version.to_string();
        let label = self.core.label.clone();
        let description = self.core.description.clone();
        let format = self.format.to_string();
        let tags = self.core.tags_json();
        let source_id = self.source_id.to_string();
        // TODO: implement device_id in flows
        let device_id = "".to_string();
        match self.format {
            Format::Video => {
                json!(is_04::v1_3_x::FlowVideo {
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
                })
            }
            Format::Audio => {
                let sample_rate = nmos_schema::is_04::v1_3_x::FlowAudioSampleRate {
                    numerator: 44000,
                    denominator: None,
                };
                json!(is_04::v1_3_x::FlowAudio {
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
                    grain_rate: None,
                })
            }
            Format::Data => {
                panic!("Data flow not implemented")
            }
        }
    }
}
