//! This example draws text into a bounding box that can be modified by
//! clicking and dragging on the display.
//!
//! Press spacebar to switch between height modes
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::PrimitiveStyle,
};
use embedded_text::{prelude::*, style::vertical_overdraw::FullRowsOnly};
use sdl2::keyboard::Keycode;
use std::{thread, time::Duration};

enum ProcessedEvent {
    Nothing,
    Quit,
    Next,
    Resize(Point),
}

impl ProcessedEvent {
    pub fn new(event: SimulatorEvent) -> Self {
        unsafe {
            // This is fine for a demo
            static mut MOUSE_DOWN: bool = false;

            match event {
                SimulatorEvent::MouseButtonDown { point, .. } => {
                    println!("MouseButtonDown: {:?}", point);
                    MOUSE_DOWN = true;
                    ProcessedEvent::Resize(point)
                }
                SimulatorEvent::MouseButtonUp { .. } => {
                    println!("MouseButtonUp");
                    MOUSE_DOWN = false;
                    ProcessedEvent::Nothing
                }
                SimulatorEvent::MouseMove { point, .. } => {
                    if MOUSE_DOWN {
                        println!("MouseMove: {:?}", point);
                        ProcessedEvent::Resize(point)
                    } else {
                        ProcessedEvent::Nothing
                    }
                }
                SimulatorEvent::KeyDown { keycode, .. } if keycode == Keycode::Space => {
                    ProcessedEvent::Next
                }
                SimulatorEvent::Quit => ProcessedEvent::Quit,
                _ => ProcessedEvent::Nothing,
            }
        }
    }
}

fn demo_loop<V>(window: &mut Window, bounds: &mut Rectangle, alignment: V) -> bool
where
    V: VerticalTextAlignment + std::fmt::Debug,
    for<'a> &'a StyledTextBox<'a, BinaryColor, Font6x8, LeftAligned, TopAligned, Exact<FullRowsOnly>>:
        Drawable<BinaryColor>,
{
    let text = "Hello, World!\nLorem Ipsum is simply dummy text of the printing and typesetting \
    industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when \
    an unknown printer took a galley of type and scrambled it to make a type specimen book.";
    loop {
        let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(255, 255));

        let textbox_style = TextBoxStyleBuilder::new(Font6x8)
            .vertical_alignment(alignment)
            .text_color(BinaryColor::On)
            .build();

        let tb = TextBox::new(text, *bounds).into_styled(textbox_style);
        tb.draw(&mut display).unwrap();

        tb.text_box
            .bounds
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        let height_text = format!("Vertical Alignment: {:?}", alignment);

        Text::new(&height_text, Point::zero())
            .into_styled(textbox_style.text_style)
            .draw(&mut display)
            .unwrap();

        window.update(&display);
        for event in window.events() {
            match ProcessedEvent::new(event) {
                ProcessedEvent::Resize(bottom_right) => {
                    bounds.bottom_right.x = bottom_right.x.max(bounds.top_left.x);
                    bounds.bottom_right.y = bottom_right.y.max(bounds.top_left.y);
                }
                ProcessedEvent::Quit => return false,
                ProcessedEvent::Next => return true,
                ProcessedEvent::Nothing => {}
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn main() -> Result<(), core::convert::Infallible> {
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    let mut window = Window::new("TextBox demonstration", &output_settings);

    let mut bounds = Rectangle::with_corners(Point::new(0, 8), Point::new(128, 200));

    'running: loop {
        if !demo_loop(&mut window, &mut bounds, TopAligned) {
            break 'running;
        }
        if !demo_loop(&mut window, &mut bounds, CenterAligned) {
            break 'running;
        }
        if !demo_loop(&mut window, &mut bounds, BottomAligned) {
            break 'running;
        }
    }

    Ok(())
}
