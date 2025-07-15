mod error;
mod intermediate;
mod ui;
mod upload;

use crate::{
    intermediate::{EncodedImage, Intermediate, Step},
    ui::{preview::Preview, viewer::Viewer},
    upload::{State, Update, Upload},
};

use ::image::{DynamicImage, EncodableLayout, imageops};
use iced::{
    Element, Length, Subscription, Task,
    advanced::image::Bytes,
    time::Instant,
    widget::{button, column, container, grid, horizontal_space, image, row, scrollable, stack},
    window,
};

#[non_exhaustive]
#[derive(Debug, Clone)]
enum Message {
    Upload,
    UploadUpdated(Update),
    Process,
    ThumbnailHovered(Step, bool),
    Open(Step),
    Close,
    Animate,
}
struct Img {
    upload: Upload,
    origin: Option<Intermediate>,
    now: Instant,
    viewer: Viewer,
    intermediates: Vec<Intermediate>,
}

impl Img {
    fn new() -> Self {
        Self {
            upload: Upload::new(),
            origin: None,
            now: Instant::now(),
            viewer: Viewer::new(),
            intermediates: Vec::new(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let is_animating = self
            .intermediates
            .iter()
            .any(|i| i.preview.is_animating(self.now))
            || self.viewer.is_animating(self.now);

        if is_animating {
            window::frames().map(|_| Message::Animate)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
        self.now = now;

        match message {
            Message::Upload => {
                let task = self.upload.start();

                task.map(Message::UploadUpdated)
            }
            Message::UploadUpdated(update) => {
                self.upload.update(update);

                Task::none()
            }
            Message::Process => {
                if let State::Finished(image) = &self.upload.state {
                    let image = image.image.clone().unwrap();
                    let preview = Preview::loading(
                        blurhash::decode(
                            &blurhash::encode(
                                4,
                                3,
                                image.width(),
                                image.height(),
                                image.to_rgba8().as_bytes(),
                            )
                            .unwrap(),
                            50,
                            50,
                            1.0,
                        )
                        .unwrap(),
                        now,
                    );
                    self.intermediates.push(Intermediate {
                        current_step: Step::Gray,
                        preview,
                        image: None,
                    });
                    let res = DynamicImage::ImageLuma8(image.to_luma8());

                    let thumbnail = res.resize(res.width(), res.height(), imageops::Lanczos3);

                    let preview = Preview::ready(thumbnail, now);

                    if let Some(last) = self.intermediates.last_mut() {
                        *last = Intermediate {
                            current_step: Step::Gray,
                            preview,
                            image: Some(res),
                        }
                    }
                }

                Task::none()
            }
            Message::ThumbnailHovered(step, is_hovered) => {
                if let Some(i) = self
                    .intermediates
                    .iter_mut()
                    .find(|i| i.current_step == step)
                {
                    i.preview.toggle_zoom(is_hovered, self.now);
                }

                Task::none()
            }
            Message::Open(step) => {
                if let Some(intermediate) = self
                    .intermediates
                    .iter()
                    .find(|i| i.current_step == step)
                    .cloned()
                {
                    self.viewer.show(intermediate.image.unwrap(), self.now);
                }

                Task::none()
            }
            Message::Close => {
                self.viewer.close(self.now);

                Task::none()
            }
            Message::Animate => Task::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        let content = container(row![
            column![
                button("Choose the image").on_press(Message::Upload),
                self.upload.view(),
                if let Some(img) = &self.origin {
                    let img = img.image.as_ref().unwrap().to_rgba8();
                    container(image(image::Handle::from_rgba(
                        img.width(),
                        img.height(),
                        Bytes::from(img.into_raw()),
                    )))
                } else {
                    container(horizontal_space()).style(container::dark)
                },
                button("Do it!").on_press(Message::Process),
            ]
            .spacing(10)
            .width(Length::FillPortion(2)),
            scrollable(column![grid(
                self.intermediates.iter().map(|i| i.card(self.now))
            )])
            .width(Length::FillPortion(8))
        ]);

        let viewer = self.viewer.view(self.now);

        stack![content, viewer].into()
    }
}

fn main() -> iced::Result {
    console_log::init().expect("Initialize logger");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    iced::application::timed(Img::new, Img::update, Img::subscription, Img::view)
        .centered()
        .run()
}
