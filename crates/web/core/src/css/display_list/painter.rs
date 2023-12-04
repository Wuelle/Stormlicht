use math::Vec2D;
use render::{Composition, Path, Source};

use crate::css::{
    display_list::{
        command::{RectCommand, TextCommand},
        Command,
    },
    layout::Pixels,
    FontMetrics,
};

#[derive(Clone, Debug, Default)]
pub struct Painter {
    commands: Vec<Command>,
    offset: Vec2D<Pixels>,
}

impl Painter {
    #[must_use]
    pub fn offset(&self) -> Vec2D<Pixels> {
        self.offset
    }

    pub fn set_offset(&mut self, offset: Vec2D<Pixels>) {
        self.offset = offset;
    }

    pub fn rect(&mut self, area: math::Rectangle<Pixels>, color: math::Color) {
        let area = math::Rectangle::from_position_and_size(
            area.top_left() + self.offset,
            area.width(),
            area.height(),
        );

        self.commands
            .push(Command::Rect(RectCommand { area, color }))
    }

    pub fn text(
        &mut self,
        text: String,
        position: Vec2D<Pixels>,
        color: math::Color,
        font_metrics: FontMetrics,
    ) {
        let position = position + self.offset;
        let text_command = TextCommand {
            position,
            text,
            font_metrics,
            color,
        };

        self.commands.push(Command::Text(text_command));
    }

    pub fn paint(&self, composition: &mut Composition) {
        for (index, command) in self.commands.iter().enumerate() {
            match command {
                Command::Rect(rect_cmd) => {
                    composition
                        .get_or_insert_layer(index as u16)
                        .with_source(Source::Solid(rect_cmd.color))
                        .with_outline(Path::rect(
                            Vec2D {
                                x: rect_cmd.area.top_left().x.0,
                                y: rect_cmd.area.top_left().y.0,
                            },
                            Vec2D {
                                x: rect_cmd.area.bottom_right().x.0,
                                y: rect_cmd.area.bottom_right().y.0,
                            },
                        ));
                },
                Command::Text(text_command) => {
                    composition
                        .get_or_insert_layer(index as u16)
                        .text(
                            &text_command.text,
                            *text_command.font_metrics.font_face.clone(),
                            text_command.font_metrics.size.into(),
                            Vec2D {
                                x: text_command.position.x.0,
                                y: text_command.position.y.0,
                            },
                        )
                        .with_source(Source::Solid(text_command.color));
                },
            }
        }
    }
}
