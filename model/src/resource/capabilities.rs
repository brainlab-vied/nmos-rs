use nmos_schema::is_04::v1_3_x::{
    FlowAudioCodedGrainRate, FlowAudioCodedSampleRate, FlowVideoCodedGrainRate,
};

#[derive(Debug, Clone)]
pub struct GrainRate {
    pub denominator: Option<i64>,
    pub numerator: i64,
}

impl Into<FlowVideoCodedGrainRate> for GrainRate {
    fn into(self) -> FlowVideoCodedGrainRate {
        FlowVideoCodedGrainRate {
            denominator: self.denominator,
            numerator: self.numerator,
        }
    }
}

impl Into<FlowAudioCodedGrainRate> for GrainRate {
    fn into(self) -> FlowAudioCodedGrainRate {
        FlowAudioCodedGrainRate {
            denominator: self.denominator,
            numerator: self.numerator,
        }
    }
}

impl Into<FlowAudioCodedSampleRate> for GrainRate {
    fn into(self) -> FlowAudioCodedSampleRate {
        FlowAudioCodedSampleRate {
            denominator: self.denominator,
            numerator: self.numerator,
        }
    }
}
