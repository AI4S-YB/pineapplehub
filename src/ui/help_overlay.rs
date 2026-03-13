//! Diablo 2 Resurrected-style help overlay.
//!
//! **Lines & dots** are drawn on a Canvas; **labels** are iced `text`
//! widgets positioned via standard layout combinators, because Canvas
//! `fill_text` has rendering issues under iced's WebGL backend with
//! multiple text calls per frame.

use crate::theme;
use crate::Message;
use iced::widget::canvas::{self, Geometry, Path, Stroke, Text as CanvasText};
use iced::widget::{button, canvas as canvas_widget, column, container, row, space, stack, text};
use iced::{Color, Element, Length, Padding, Point, Rectangle, Size, Theme, mouse};

// ────────────────────────  Annotation  ────────────────────────

struct Annotation {
    /// Pixel x of the target.
    tx: f32,
    /// Pixel y of the target.
    ty: f32,
    /// Proportional y of the label text (0–1 of viewport height).
    label_frac_y: f32,
}

// ────────────────────────  Layout constants  ────────────────────────

const TB_PAD_Y: f32 = 14.0;
const TB_PAD_X: f32 = 20.0;
const TB_SPACING: f32 = 8.0;

const ICON_SZ: f32 = 20.0;
const ICON_BTN_PAD: f32 = 6.0;
const ICON_BTN_W: f32 = ICON_SZ + ICON_BTN_PAD * 2.0;

const LOGO_H: f32 = 34.0;
const LOGO_PAD_X: f32 = 8.0;
const LOGO_FAVICON_SZ: f32 = 26.0;

const TB_H: f32 = TB_PAD_Y + LOGO_H + TB_PAD_Y;
const TB_CENTER_Y: f32 = TB_H / 2.0;

const SEP_H: f32 = 2.0;
const CONTENT_TOP: f32 = TB_H + SEP_H;

// ────────────────────────  Canvas (lines & dots only)  ────────────

struct LinesCanvas {
    build: fn(f32, f32) -> Vec<Annotation>,
}

impl canvas::Program<Message> for LinesCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;

        // ── Scrim ──
        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(w, h),
            Color { a: 0.82, ..theme::BG_DEEP },
        );

        let line_color = Color::from_rgba(1.0, 0.85, 0.3, 0.50);
        let dot_r = 4.0_f32;

        let annotations = (self.build)(w, h);

        for ann in &annotations {
            let px = ann.tx;
            let target_py = ann.ty;
            let label_py = ann.label_frac_y * h;

            // Vertical line from target toward label
            let (la, lb) = if label_py > target_py {
                (target_py, label_py + 4.0)
            } else {
                (target_py, label_py + 18.0)
            };
            if (lb - la).abs() > 4.0 {
                frame.stroke(
                    &Path::line(Point::new(px, la), Point::new(px, lb)),
                    Stroke::default().with_color(line_color).with_width(1.5),
                );
            }

            // Target dot
            frame.fill(&Path::circle(Point::new(px, target_py), dot_r), theme::ACCENT);
        }

        // Close hint — single Canvas text call (works reliably)
        let hint = "Press F1 / ESC to close     ·     Click anywhere to dismiss";
        frame.fill_text(CanvasText {
            content: hint.to_string(),
            position: Point::new(w * 0.17, h - 30.0),
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.45),
            size: iced::Pixels(13.0),
            ..CanvasText::default()
        });

        vec![frame.into_geometry()]
    }
}

// ────────────────────────  Annotation data  ────────────────────────

fn analysis_annotations(w: f32, _h: f32) -> Vec<Annotation> {
    let github_cx = w - TB_PAD_X - ICON_BTN_W / 2.0;
    let history_cx = github_cx - TB_SPACING - ICON_BTN_W;
    let logo_cx = TB_PAD_X + LOGO_PAD_X + LOGO_FAVICON_SZ / 2.0;

    vec![
        Annotation { tx: logo_cx, ty: TB_CENTER_Y, label_frac_y: 0.20 },
        Annotation { tx: history_cx, ty: TB_CENTER_Y, label_frac_y: 0.30 },
        Annotation { tx: github_cx, ty: TB_CENTER_Y, label_frac_y: 0.42 },
    ]
}

fn history_annotations(w: f32, h: f32) -> Vec<Annotation> {
    let sidebar_w = w * 0.18;
    let sidebar_cx = sidebar_w / 2.0;
    let main_x = sidebar_w;
    let main_w = w - sidebar_w;
    let tab_y = CONTENT_TOP + 15.0;
    let session_list_y = CONTENT_TOP + 40.0;
    let cleanup_y = h - 40.0;

    vec![
        Annotation { tx: sidebar_cx, ty: session_list_y, label_frac_y: 0.45 },
        Annotation { tx: sidebar_cx * 0.4, ty: session_list_y + 30.0, label_frac_y: 0.58 },
        Annotation { tx: sidebar_cx * 1.6, ty: session_list_y + 30.0, label_frac_y: 0.58 },
        Annotation { tx: sidebar_cx, ty: session_list_y + 100.0, label_frac_y: 0.70 },
        Annotation { tx: sidebar_cx, ty: cleanup_y, label_frac_y: 0.85 },
        Annotation { tx: main_x + main_w * 0.15, ty: tab_y, label_frac_y: 0.30 },
        Annotation { tx: main_x + main_w * 0.35, ty: tab_y, label_frac_y: 0.30 },
        Annotation { tx: main_x + main_w * 0.52, ty: tab_y, label_frac_y: 0.30 },
        Annotation { tx: main_x + main_w * 0.40, ty: CONTENT_TOP + 60.0, label_frac_y: 0.50 },
        Annotation { tx: main_x + main_w * 0.80, ty: CONTENT_TOP + 60.0, label_frac_y: 0.50 },
        Annotation { tx: main_x + main_w * 0.90, ty: CONTENT_TOP + 150.0, label_frac_y: 0.65 },
        Annotation { tx: main_x + main_w * 0.45, ty: CONTENT_TOP + 200.0, label_frac_y: 0.78 },
    ]
}

// ────────────────────────  Styled label  ────────────────────────

fn lbl<'a>(s: &str) -> Element<'a, Message> {
    text(s.to_string())
        .size(15)
        .color(theme::ACCENT)
        .into()
}

// ────────────────────────  Public API  ────────────────────────

pub(crate) fn view_analysis_help<'a>() -> Element<'a, Message> {
    let lines = canvas_widget(LinesCanvas { build: analysis_annotations })
        .width(Length::Fill)
        .height(Length::Fill);

    // Labels use FillPortion spacers to match Canvas label_frac_y exactly:
    // Logo=0.20, History=0.30, GitHub=0.42
    let labels: Element<'_, Message> = column![
        // 0 → 0.20
        space::vertical().height(Length::FillPortion(20)),
        container(lbl("Logo — Back to Analysis"))
            .padding(Padding::from([0, 0]).left(10)),
        // 0.20 → 0.30
        space::vertical().height(Length::FillPortion(10)),
        // History (right-aligned)
        container(
            row![
                space::horizontal().width(Length::Fill),
                lbl("History"),
                space::horizontal().width(Length::Fixed(85.0)),
            ]
        ).width(Length::Fill),
        // 0.30 → 0.42
        space::vertical().height(Length::FillPortion(12)),
        // GitHub (right-aligned)
        container(
            row![
                space::horizontal().width(Length::Fill),
                lbl("GitHub Repository"),
                space::horizontal().width(Length::Fixed(15.0)),
            ]
        ).width(Length::Fill),
        // 0.42 → 1.0
        space::vertical().height(Length::FillPortion(58)),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    let overlay = stack![lines, labels]
        .width(Length::Fill)
        .height(Length::Fill);

    button(overlay)
        .on_press(Message::ToggleHelp)
        .style(theme::text_button_style)
        .padding(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub(crate) fn view_history_help<'a>() -> Element<'a, Message> {
    let lines = canvas_widget(LinesCanvas { build: history_annotations })
        .width(Length::Fill)
        .height(Length::Fill);

    // Labels laid out with iced widgets
    let labels: Element<'_, Message> = column![
        space::vertical().height(Length::Fixed(100.0)),
        // Sidebar + Main row for top labels
        row![
            container(lbl("Session List"))
                .width(Length::FillPortion(1))
                .center_x(Length::Fill),
            container(
                column![
                    lbl("Records    Statistics    Charts"),
                    space::vertical().height(Length::Fixed(50.0)),
                    lbl("Click Header to Sort"),
                ]
            ).width(Length::FillPortion(4))
            .padding(Padding::from([0, 0]).left(30)),
        ],
        space::vertical().height(Length::Fixed(30.0)),
        // Middle section
        row![
            container(
                column![
                    lbl("☑ Check to Load"),
                    space::vertical().height(Length::Fixed(10.0)),
                    lbl("★ Star = Protected"),
                    space::vertical().height(Length::Fixed(20.0)),
                    lbl("Double-click to Rename"),
                ]
            ).width(Length::FillPortion(1))
            .padding(Padding::from([0, 0]).left(5)),
            container(
                column![
                    row![
                        space::horizontal().width(Length::Fill),
                        lbl("Search & Filters"),
                        space::horizontal().width(Length::Fixed(20.0)),
                    ],
                    space::vertical().height(Length::Fixed(30.0)),
                    row![
                        space::horizontal().width(Length::Fill),
                        lbl("✏ Edit Note / Metric"),
                        space::horizontal().width(Length::Fixed(10.0)),
                    ],
                    space::vertical().height(Length::Fixed(30.0)),
                    lbl("Orange Row = Outlier"),
                ]
            ).width(Length::FillPortion(4)),
        ],
        space::vertical().height(Length::Fill),
        container(lbl("Cleanup / Clear All"))
            .padding(Padding::from([0, 0]).left(10)),
        space::vertical().height(Length::Fixed(20.0)),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    let overlay = stack![lines, labels]
        .width(Length::Fill)
        .height(Length::Fill);

    button(overlay)
        .on_press(Message::ToggleHelp)
        .style(theme::text_button_style)
        .padding(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
