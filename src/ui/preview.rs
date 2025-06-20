pub(super) mod thumbnail;

use ::image::DynamicImage;
use iced::{
    Animation, animation,
    time::{Instant, milliseconds},
    widget::image,
};
use thumbnail::Thumbnail;

use crate::EncodedImage;

#[derive(Clone, Debug)]
pub(crate) struct Blurhash {
    pub(crate) handle: image::Handle,
    pub(crate) fade_in: Animation<bool>,
}

#[derive(Clone, Debug)]
pub(crate) enum Preview {
    Loading {
        blurhash: Blurhash,
    },
    Ready {
        blurhash: Option<Blurhash>,
        thumbnail: Thumbnail,
    },
}

impl Preview {
    const WIDTH: u32 = 320;
    const HEIGHT: u32 = 410;

    pub(crate) fn loading(img: EncodedImage, now: Instant) -> Self {
        Self::Loading {
            blurhash: Blurhash {
                fade_in: Animation::new(false)
                    .duration(milliseconds(700))
                    .easing(animation::Easing::EaseIn)
                    .go(true, now),
                handle: image::Handle::from_bytes(img),
            },
        }
    }

    pub(crate) fn ready(img: DynamicImage, now: Instant) -> Self {
        Self::Ready {
            blurhash: None,
            thumbnail: Thumbnail::new(img, now),
        }
    }

    fn load(self, img: DynamicImage, now: Instant) -> Self {
        let Self::Loading { blurhash } = self else {
            return self;
        };

        Self::Ready {
            blurhash: Some(blurhash),
            thumbnail: Thumbnail::new(img, now),
        }
    }

    pub(crate) fn toggle_zoom(&mut self, enabled: bool, now: Instant) {
        if let Self::Ready { thumbnail, .. } = self {
            thumbnail.zoom.go_mut(enabled, now);
        }
    }

    pub(crate) fn is_animating(&self, now: Instant) -> bool {
        match &self {
            Self::Loading { blurhash } => blurhash.fade_in.is_animating(now),
            Self::Ready { thumbnail, .. } => {
                thumbnail.fade_in.is_animating(now) || thumbnail.zoom.is_animating(now)
            }
        }
    }

    pub(crate) fn blurhash(&self, now: Instant) -> Option<&Blurhash> {
        match self {
            Self::Loading { blurhash, .. } => Some(blurhash),
            Self::Ready {
                blurhash: Some(blurhash),
                thumbnail,
                ..
            } if thumbnail.fade_in.is_animating(now) => Some(blurhash),
            Self::Ready { .. } => None,
        }
    }
}
