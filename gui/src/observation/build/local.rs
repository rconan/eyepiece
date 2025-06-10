use std::{
    env,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use eyepiece::{FieldBuilder, FieldImage, Gmt, Hst, Jwst, SEED};
use std::time::SystemTime;

use crate::observation::{Based, Builder, Observation, Telescope};

impl Observation {
    pub fn build(&mut self, field: Arc<Mutex<FieldImage>>, flag: Arc<AtomicBool>) {
        self.stars.update_fov(self.camera.field_of_view);
        let this = self.clone();
        thread::spawn(move || {
            if this.stars.seed {
                unsafe {
                    env::remove_var("SEED");
                }
                SEED.store(
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    Ordering::Relaxed,
                );
            }
            let image = match this.telescope {
                Based::Ground(Telescope::GMT) => FieldBuilder::new(Gmt::new())
                    .camera(&this.camera)
                    .objects(&this.stars)
                    .observation(&this.mode),
                Based::Ground(Telescope::Telescope(telescope)) => FieldBuilder::new(telescope)
                    .camera(&this.camera)
                    .objects(&this.stars)
                    .observation(&this.mode),
                Based::Space(Telescope::JWST) => FieldBuilder::new(Jwst::new())
                    .camera(&this.camera)
                    .objects(&this.stars)
                    .observation(&this.mode),
                Based::Space(Telescope::HST) => FieldBuilder::new(Hst::new())
                    .camera(&this.camera)
                    .objects(&this.stars)
                    .observation(&this.mode),
                _ => unimplemented!(),
            };
            *field.lock().unwrap() = image;
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        });
    }
}
