use anyhow::Result;
use font_kit::{family_name::FamilyName, properties::Properties, source::SystemSource};
use minifb::{Window, WindowOptions};
use raqote::{DrawOptions, DrawTarget, SolidSource, Source, StrokeStyle};
use std::thread::JoinHandle;
use tray_icon::{menu::Menu, tray_event_receiver, ClickEvent, TrayEvent, TrayIconBuilder};
use wfd::DialogParams;
use windows::{
    w,
    Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK},
};

use crate::{
    alert,
    counter::DrawConfig,
    tools::{
        hide_window, is_app_registered_for_startup, load_icon_from_memory,
        load_tray_icon_from_memory, register_app_for_startup, remove_app_for_startup,
        remove_keyboard_hook, remove_mouse_hook, save_storage, set_window_icon, show_window,
    },
    COUNTER,
};

const ICON: &[u8] = include_bytes!("../icon.rgba.bzip2");
const ICON_SIZE: u32 = 128;

pub const WIDTH: usize = 750;
pub const HEIGHT: usize = 250;

pub fn open(first_run: bool) -> JoinHandle<Result<()>> {
    std::thread::spawn(move || run(first_run))
}

pub fn run(mut first_run: bool) -> Result<()> {
    let app_name = "按键统计";

    let mut window = Window::new(
        app_name,
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )?;

    let mut menu = minifb::Menu::new("设置")?;
    menu.add_item("开机启动", 0).build();
    menu.add_item("保存图片", 1).build();
    window.add_menu(&menu);

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let properties = Properties::default();

    let font = SystemSource::new()
        .select_best_match(
            &vec![
                FamilyName::Title("微软雅黑".to_string()),
                FamilyName::Title("Tahoma".to_string()),
            ],
            &properties,
        )?
        .load()?;

    let draw_config = DrawConfig {
        background: Source::Solid(SolidSource::from_unpremultiplied_argb(
            0xFF, 0x1a, 0x21, 0x2c,
        )),
        border_color: Source::Solid(SolidSource::from_unpremultiplied_argb(
            0xFF, 0x70, 0x70, 0x70,
        )),
        label_color: Source::Solid(SolidSource::from_unpremultiplied_argb(
            0xFF, 0xad, 0xad, 0xad,
        )),
        text_color: Source::Solid(SolidSource::from_unpremultiplied_argb(
            0xFF, 0xF1, 0xF1, 0xF1,
        )),
        stroke_style: StrokeStyle {
            width: 1.,
            ..Default::default()
        },
        draw_options: DrawOptions::default(),
        lable_font_size: 20.,
        font_size: 24.,
    };

    let size = window.get_size();
    let mut dt = DrawTarget::new(size.0 as i32, size.1 as i32);

    let tray_menu = Menu::new();
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip(app_name)
        .with_icon(load_tray_icon_from_memory(
            ICON.to_vec(),
            ICON_SIZE,
            ICON_SIZE,
        )?)
        .build()?;

    let mut active = true;

    set_window_icon(
        &window,
        load_icon_from_memory(ICON.to_vec(), ICON_SIZE, ICON_SIZE)?,
    )?;

    while window.is_open() {
        if active {
            //渲染
            let counter = COUNTER.read().unwrap();
            counter.draw(&mut dt, &font, &draw_config);
        }

        if !first_run {
            hide_window(&window);
            first_run = true;
        }

        if active && !window.is_active() {
            active = false;
            let (width, height) = window.get_size();
            if width == 0 && height == 0 {
                hide_window(&window);
            }
        }
        if !active && window.is_active() {
            active = true;
        }

        if let Some(menu_id) = window.is_menu_pressed() {
            match menu_id {
                0 => {
                    if is_app_registered_for_startup(app_name)? {
                        remove_app_for_startup(app_name)?;
                        alert!("已关闭开机启动！");
                    } else {
                        register_app_for_startup(app_name)?;
                        alert!("已设置开机启动！");
                    }
                }
                1 => {
                    let date = chrono::Local::now();
                    let file_name = format!("{}-{}", app_name, date.format("%Y-%m-%d_%H-%M-%S"));
                    let params = DialogParams {
                        title: "保存图片",
                        file_types: vec![("PNG", "*.png")],
                        default_extension: "png",
                        file_name: &file_name,
                        ..Default::default()
                    };
                    let dialog_result = wfd::save_dialog(params).unwrap();
                    let save_path = dialog_result
                        .selected_file_path
                        .to_str()
                        .unwrap_or(&file_name);
                    dt.write_png(save_path)?;
                }
                _ => (),
            }
        }

        window.update_with_buffer(dt.get_data(), size.0, size.1)?;

        //处理托盘事件
        match tray_event_receiver().try_recv() {
            Ok(TrayEvent { event, .. }) => match event {
                ClickEvent::Left | ClickEvent::Double => {
                    if !active {
                        show_window(&window);
                    }
                }
                _ => (),
            },
            Err(_) => (),
        };
    }

    remove_keyboard_hook();
    remove_mouse_hook();
    //存盘
    save_storage(&COUNTER.read().unwrap())?;
    //退出
    std::process::exit(0);
}
