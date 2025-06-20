use ::image::DynamicImage;
use iced::{Animation, animation, time::Instant, widget::image};

#[derive(Clone, Debug)]
pub(crate) struct Thumbnail {
    pub(crate) handle: image::Handle,
    pub(crate) fade_in: Animation<bool>,
    pub(crate) zoom: Animation<bool>,
}

impl Thumbnail {
    pub fn new(img: DynamicImage, now: Instant) -> Self {
        Self {
            handle: image::Handle::from_rgba(img.width(), img.height(), img.to_rgba8().into_raw()),
            fade_in: Animation::new(false).slow().go(true, now),
            zoom: Animation::new(false)
                .quick()
                .easing(animation::Easing::EaseInOut),
        }
    }
}
