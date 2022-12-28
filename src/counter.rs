use chrono::Utc;
use font_kit::font::Font;
use num_enum::TryFromPrimitive;
use raqote::{DrawOptions, DrawTarget, PathBuilder, Point as PointF, Source, StrokeStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    tools::{draw_text, measure_text},
    window::{HEIGHT, WIDTH},
};

const VK_CTRL: u32 = 162;
const VK_ESC: u32 = 27;
const VK_TAB: u32 = 9;
const VK_BACKSPACE: u32 = 8;
// const VK_SHIFT: u32 = 160;
const VK_ALT: u32 = 164;
const VK_DELETE: u32 = 46;
const VK_ENTER: u32 = 13;
const VK_C: u32 = 67;
const VK_X: u32 = 88;
const VK_V: u32 = 86;
const VK_Z: u32 = 90;
const VK_Y: u32 = 89;
const VK_S: u32 = 83;

#[derive(Debug)]
pub enum KeyEvent {
    KeyPress(u32),
    KeyUp(u32),
}

#[derive(Serialize, Default, Deserialize, PartialEq, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(i32)]
pub enum MouseEvent {
    MouseMove = 0x200,
    MouseLeftBUttonDown = 0x201,
    MouseLeftButtonUp = 0x202,
    MouseRightButtonDown = 0x204,
    MouseRightButtonUp = 0x205,
    MouseWheelRouting = 0x20A,
    MouseMiddleButtonDown = 0x2b,
    MouseMiddleButtonUp = 0x20c,
}

#[derive(Debug)]
pub enum Event {
    KeyEvent(KeyEvent),
    MouseEvent((MouseEvent, Point)),
}

const MOUSE_LEFT_CLICK_COUNT: &str = "鼠标左击";
const MOUSE_RIGHT_CLICK_COUNT: &str = "鼠标右击";
const MOUSE_DOUBLE_CLICKS_COUNT: &str = "鼠标双击";
const MOUSE_WHEEL_COUNT: &str = "鼠标滚轮";
const MOUSE_MOVE_COUNT: &str = "鼠标移动";

const KEY_KEYSTROKES: &str = "键盘敲击";
const KEY_CTRL_C: &str = "CTRL + C";
const KEY_CTRL_X: &str = "CTRL + X";
const KEY_CTRL_V: &str = "CTRL + V";
const KEY_CTRL_Z: &str = "CTRL + Z";
const KEY_CTRL_Y: &str = "CTRL + Y";
const KEY_CTRL_S: &str = "CTRL + S";
const KEY_ALT_TAB: &str = "ALT + TAB";
const KEY_BACKSPACE: &str = "Backspace";
const KEY_ENTER: &str = "Enter";
const KEY_ESC: &str = "Esc";
const KEY_DELETE: &str = "Delete";
const KEY_TAB: &str = "Tab";

const KEY_LIST: &[&str] = &[
    MOUSE_LEFT_CLICK_COUNT,
    MOUSE_RIGHT_CLICK_COUNT,
    MOUSE_DOUBLE_CLICKS_COUNT,
    MOUSE_WHEEL_COUNT,
    MOUSE_MOVE_COUNT,
    KEY_KEYSTROKES,
    KEY_CTRL_C,
    KEY_CTRL_X,
    KEY_CTRL_V,
    KEY_CTRL_Z,
    KEY_CTRL_Y,
    KEY_CTRL_S,
    KEY_ALT_TAB,
    KEY_BACKSPACE,
    KEY_ENTER,
    KEY_ESC,
    KEY_DELETE,
    KEY_TAB,
];

pub struct DrawConfig<'a> {
    pub background: Source<'a>,
    pub border_color: Source<'a>,
    pub label_color: Source<'a>,
    pub text_color: Source<'a>,
    pub draw_options: DrawOptions,
    pub stroke_style: StrokeStyle,
    pub lable_font_size: f32,
    pub font_size: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Counter {
    pub timestamp: i64,
    pub maps: HashMap<String, u128>,
    pub ctrl_press: bool,
    pub alt_press: bool,
    pub last_mouse_click_event: (i64, Point),
    pub last_mouse_wheel_time: i64,
    pub last_mouse_move_time: i64,
}

impl Counter {
    /// 清空数据
    pub fn clear(&mut self) {
        self.maps.clear();
    }
    pub fn recv(&mut self, event: Event) {
        match event {
            Event::KeyEvent(KeyEvent::KeyPress(code)) => {
                self.add_count(KEY_KEYSTROKES);
                // println!("code={code}");
                if code == VK_CTRL {
                    self.ctrl_press = true;
                }
                if code == VK_ALT {
                    self.alt_press = true;
                }
                if code == VK_BACKSPACE {
                    self.add_count(KEY_BACKSPACE);
                }
                if code == VK_ENTER {
                    self.add_count(KEY_ENTER);
                }
                if code == VK_DELETE {
                    self.add_count(KEY_DELETE);
                }
                if code == VK_ESC {
                    self.add_count(KEY_ESC);
                }
                if code == VK_TAB {
                    self.add_count(KEY_TAB);
                    if self.alt_press {
                        self.add_count(KEY_ALT_TAB);
                    }
                }

                if self.ctrl_press {
                    match code {
                        VK_C => self.add_count(KEY_CTRL_C),
                        VK_S => self.add_count(KEY_CTRL_S),
                        VK_X => self.add_count(KEY_CTRL_X),
                        VK_Z => self.add_count(KEY_CTRL_Z),
                        VK_Y => self.add_count(KEY_CTRL_Y),
                        VK_V => self.add_count(KEY_CTRL_V),
                        _ => (),
                    }
                };
            }
            Event::KeyEvent(KeyEvent::KeyUp(code)) => {
                if code == VK_CTRL {
                    self.ctrl_press = false;
                } else if code == VK_ALT {
                    self.alt_press = false;
                }
            }
            Event::MouseEvent((MouseEvent::MouseLeftBUttonDown, pt)) => {
                self.add_count(MOUSE_LEFT_CLICK_COUNT);
                let now = Utc::now().timestamp_millis();
                //判断双击
                let (last_time, last_pt) = &self.last_mouse_click_event;
                if now - *last_time < 500
                    && (last_pt.x - pt.x).abs() < 4
                    && (last_pt.y - pt.y).abs() < 4
                {
                    self.add_count(MOUSE_DOUBLE_CLICKS_COUNT);
                }
                self.last_mouse_click_event = (now, pt);
            }
            Event::MouseEvent((MouseEvent::MouseRightButtonDown, _)) => {
                self.add_count(MOUSE_RIGHT_CLICK_COUNT);
            }
            Event::MouseEvent((MouseEvent::MouseWheelRouting, _)) => {
                let now = Utc::now().timestamp_millis();
                if now - self.last_mouse_wheel_time > 800 {
                    self.last_mouse_wheel_time = now;
                    self.add_count(MOUSE_WHEEL_COUNT);
                }
            }
            Event::MouseEvent((MouseEvent::MouseMove, _)) => {
                let now = Utc::now().timestamp_millis();
                if now - self.last_mouse_move_time > 800 {
                    self.last_mouse_move_time = now;
                    self.add_count(MOUSE_MOVE_COUNT);
                }
            }
            _ => (),
        }
    }

    pub fn add_count(&mut self, name: &str) {
        if let Some(val) = self.maps.get_mut(name) {
            *val += 1;
        } else {
            self.maps.insert(name.to_string(), 1);
        }
    }

    pub fn draw(&self, dt: &mut DrawTarget, font: &Font, draw_config: &DrawConfig) {
        // 清空
        dt.fill_rect(
            0.,
            0.,
            dt.width() as f32,
            dt.height() as f32,
            &draw_config.background,
            &draw_config.draw_options,
        );
        let box_margin = 10.;
        let box_width = (WIDTH as f32 - box_margin) / 6. - box_margin;
        let box_height = (HEIGHT as f32 - box_margin * 4.) / 3.;
        let corner = 6.;
        let start_x = 0.;
        let starty_y = 0.;

        let mut cursor_x = start_x;
        let mut cursor_y = starty_y + box_margin;
        for (index, key) in KEY_LIST.iter().enumerate() {
            let val = self.maps.get(*key).unwrap_or(&0);
            if index > 0 && index % 6 == 0 {
                cursor_y += box_height + box_margin;
                cursor_x = start_x;
            }
            cursor_x += box_margin;
            draw_box(
                cursor_x,
                cursor_y,
                box_width,
                box_height,
                corner,
                &key,
                &format!("{val}"),
                dt,
                font,
                draw_config,
            );
            cursor_x += box_width;
        }
    }
}

fn draw_box(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    corner: f32,
    label: &str,
    text: &str,
    dt: &mut DrawTarget,
    font: &Font,
    draw_config: &DrawConfig,
) {
    let mut pb = PathBuilder::new();
    let half_corner = corner / 2.;
    let x0 = x;
    let y0 = y;
    let x1 = x + width;
    let y1 = y;
    let x2 = x + width;
    let y2 = y + height;
    let x3 = x;
    let y3 = y + height;

    pb.move_to(x0 + corner, y0);
    pb.line_to(x1 - corner, y1);
    pb.cubic_to(x1 - half_corner, y1, x1, y + half_corner, x1, y1 + corner);
    pb.line_to(x2, y2 - corner);
    pb.cubic_to(x2, y2 - half_corner, x2 - half_corner, y2, x2 - corner, y2);
    pb.line_to(x3 + corner, y3);
    pb.cubic_to(x3 + half_corner, y3, x3, y3 - half_corner, x3, y3 - corner);
    pb.line_to(x0, y0 + corner);
    pb.cubic_to(x0, y0 + half_corner, x0 + half_corner, y0, x0 + corner, y0);
    let path = pb.finish();
    dt.stroke(
        &path,
        &draw_config.border_color,
        &draw_config.stroke_style,
        &draw_config.draw_options,
    );
    let point_size = draw_config.lable_font_size;
    let measure_size = measure_text(font, point_size, label);
    let start_x = x + width / 2. - measure_size.x / 2.;
    let start = PointF::new(start_x, y + height - point_size / 2.);

    draw_text(
        dt,
        font,
        point_size,
        label,
        start,
        &draw_config.label_color,
        &draw_config.draw_options,
    );

    // 绘制数量
    let point_size = draw_config.font_size;
    let measure_size = measure_text(font, point_size, text);
    let start_x = x + width / 2. - measure_size.x / 2.;
    let start = PointF::new(start_x, y + height / 2.);

    draw_text(
        dt,
        font,
        point_size,
        text,
        start,
        &draw_config.text_color,
        &draw_config.draw_options,
    );
}
