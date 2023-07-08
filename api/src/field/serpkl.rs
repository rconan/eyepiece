use std::{fs::File, path::Path};

use crate::{
    AdaptiveOptics, DiffractionLimited, Field, Observer, Observing, ObservingModes, SeeingLimited,
};
use image::ImageError;
use serde::ser::{Serialize, SerializeStruct, Serializer};

impl Serialize for Observing<DiffractionLimited> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let tag = "diffraction limited";
        tag.serialize(serializer)
    }
}
impl Serialize for Observing<SeeingLimited> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.seeing
            .clone()
            .as_mut()
            .map(|seeing| {
                seeing.adaptive_optics = None;
                seeing
            })
            .serialize(serializer)
    }
}
impl Serialize for Observing<AdaptiveOptics> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.seeing.serialize(serializer)
    }
}

impl<T> Serialize for Field<T, DiffractionLimited>
where
    T: Observer,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Field", 8)?;
        s.serialize_field("pixel_scale", &self.pixel_scale)?;
        s.serialize_field("field_of_view", &self.field_of_view)?;
        s.serialize_field("photometry", &self.photometry)?;
        s.serialize_field("objects", &self.objects)?;
        s.serialize_field("exposure", &self.exposure)?;
        s.serialize_field("poisson_noise", &self.poisson_noise)?;
        s.serialize_field("observer", &self.observer)?;
        s.serialize_field("observing_mode", &self.observing_mode)?;
        s.end()
    }
}
impl<T> Serialize for Field<T, SeeingLimited>
where
    T: Observer,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Field", 8)?;
        s.serialize_field("pixel_scale", &self.pixel_scale)?;
        s.serialize_field("field_of_view", &self.field_of_view)?;
        s.serialize_field("photometry", &self.photometry)?;
        s.serialize_field("objects", &self.objects)?;
        s.serialize_field("exposure", &self.exposure)?;
        s.serialize_field("poisson_noise", &self.poisson_noise)?;
        s.serialize_field("observer", &self.observer)?;
        s.serialize_field("observing_mode", &self.observing_mode)?;
        s.end()
    }
}
impl<T> Serialize for Field<T, AdaptiveOptics>
where
    T: Observer,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Field", 8)?;
        s.serialize_field("pixel_scale", &self.pixel_scale)?;
        s.serialize_field("field_of_view", &self.field_of_view)?;
        s.serialize_field("photometry", &self.photometry)?;
        s.serialize_field("objects", &self.objects)?;
        s.serialize_field("exposure", &self.exposure)?;
        s.serialize_field("poisson_noise", &self.poisson_noise)?;
        s.serialize_field("observer", &self.observer)?;
        s.serialize_field("observing_mode", &self.observing_mode)?;
        s.end()
    }
}
struct Data<'a, T, Mode>
where
    T: Observer + Sync + Send,
    Mode: ObservingModes + Send,
{
    field: &'a Field<T, Mode>,
    intensity: Vec<f64>,
}
impl<'a, T> Serialize for Data<'a, T, DiffractionLimited>
where
    T: Observer + Send + Sync,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Data", 2)?;
        s.serialize_field("field", self.field)?;
        s.serialize_field("intensity", &self.intensity)?;
        s.end()
    }
}
impl<'a, T> Serialize for Data<'a, T, SeeingLimited>
where
    T: Observer + Send + Sync,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Data", 2)?;
        s.serialize_field("field", self.field)?;
        s.serialize_field("intensity", &self.intensity)?;
        s.end()
    }
}
impl<'a, T> Serialize for Data<'a, T, AdaptiveOptics>
where
    T: Observer + Send + Sync,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Data", 2)?;
        s.serialize_field("field", self.field)?;
        s.serialize_field("intensity", &self.intensity)?;
        s.end()
    }
}

impl<T> Field<T, DiffractionLimited>
where
    T: Observer + Send + Sync,
{
    pub fn dump<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ImageError> {
        let intensity = self.intensity(Default::default());
        let data = Data {
            field: self,
            intensity,
        };
        serde_pickle::to_writer(&mut File::create(path.as_ref())?, &data, Default::default())
            .expect(&format!(
                "failed to write field intensity into pickle file {:?}",
                path.as_ref()
            ));
        Ok(())
    }
}

impl<T> Field<T, SeeingLimited>
where
    T: Observer + Send + Sync,
{
    pub fn dump<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ImageError> {
        let intensity = self.intensity(Default::default());
        let data = Data {
            field: self,
            intensity,
        };
        serde_pickle::to_writer(&mut File::create(path.as_ref())?, &data, Default::default())
            .expect(&format!(
                "failed to write field intensity into pickle file {:?}",
                path.as_ref()
            ));
        Ok(())
    }
}

impl<T> Field<T, AdaptiveOptics>
where
    T: Observer + Send + Sync,
{
    pub fn dump<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ImageError> {
        let intensity = self.intensity(Default::default());
        let data = Data {
            field: self,
            intensity,
        };
        serde_pickle::to_writer(&mut File::create(path.as_ref())?, &data, Default::default())
            .expect(&format!(
                "failed to write field intensity into pickle file {:?}",
                path.as_ref()
            ));
        Ok(())
    }
}
