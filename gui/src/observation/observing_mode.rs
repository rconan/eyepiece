use std::fmt::Display;

use eyepiece::{
    AdaptiveOptics, Builder, DiffractionLimited, Field, FieldBuilder, FieldImage, SeeingBuilder,
    SeeingLimited,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub enum ObservingMode {
    #[default]
    DiffractionLimited,
    SeeingLimited(SeeingBuilder),
    AdaptiveOptics {
        seeing: SeeingBuilder,
        strehl_ratio: f64,
    },
}

impl Display for ObservingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObservingMode::DiffractionLimited => write!(f, "DiffractionLimited"),
            ObservingMode::SeeingLimited(_) => write!(f, "SeeingLimited"),
            ObservingMode::AdaptiveOptics { strehl_ratio, .. } => {
                write!(f, "AdaptiveOptics{strehl_ratio}")
            }
        }
    }
}

impl ObservingMode {
    pub fn build<T: eyepiece::Observer>(&self, builder: FieldBuilder<T>) -> FieldImage {
        match self {
            ObservingMode::DiffractionLimited => {
                <FieldBuilder<T> as Builder<Field<T, DiffractionLimited>>>::build(builder).into()
            }
            ObservingMode::SeeingLimited(seeing) => {
                <FieldBuilder<T> as Builder<Field<T, SeeingLimited>>>::build(
                    builder.seeing_limited(seeing.clone()),
                )
                .into()
            }
            ObservingMode::AdaptiveOptics {
                seeing,
                strehl_ratio,
            } => <FieldBuilder<T> as Builder<Field<T, AdaptiveOptics>>>::build(
                builder.seeing_limited(seeing.clone().ngao(*strehl_ratio, None)),
            )
            .into(),
        }
    }
}
