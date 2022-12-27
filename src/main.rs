#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use chrono::Utc;
use counter::{Counter, Event, KeyEvent, MouseEvent, Point};
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::RwLock};
use tools::{
    read_storage, save_storage_async, set_keyboard_hook, set_mouse_hook, KEYBOARD_HOOK, MOUSE_HOOK,
};
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, TranslateMessage, KBDLLHOOKSTRUCT, MSG,
        MSLLHOOKSTRUCT,
    },
};

mod counter;
mod tools;
mod window;

pub static COUNTER: Lazy<RwLock<Counter>> = Lazy::new(|| {
    RwLock::new(read_storage().unwrap_or(Counter {
        timestamp: Utc::now().timestamp_millis(),
        maps: HashMap::new(),
        ctrl_press: false,
        last_mouse_click_event: (0, Point::default()),
        last_mouse_wheel_time: 0,
        last_mouse_move_time: 0,
    }))
});

fn main() -> Result<()> {
    window::open(COUNTER.read().unwrap().maps.len() == 0);

    let mut last_save_time = Utc::now().timestamp_millis();
    set_keyboard_hook(keyboard_hook_proc)?;
    set_mouse_hook(mouse_hook_proc)?;

    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).0 > 0 {
            //每隔1分钟存盘
            let now = Utc::now().timestamp_millis();
            if now - last_save_time > 60 * 1000 {
                let _ = save_storage_async(&COUNTER.read().unwrap());
                last_save_time = now;
            }

            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}

unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mut counter = COUNTER.write().unwrap();
    let data = lparam.0 as *const KBDLLHOOKSTRUCT;
    if !data.is_null() {
        let data: &KBDLLHOOKSTRUCT = &*data;

        match wparam {
            WPARAM(0x100) => {
                //普通键按下
                counter.recv(Event::KeyEvent(KeyEvent::KeyPress(data.vkCode)));
            }
            WPARAM(0x101) => {
                //普通键抬起
                counter.recv(Event::KeyEvent(KeyEvent::KeyUp(data.vkCode)));
            }
            WPARAM(0x104) => {
                //系统键按下
                counter.recv(Event::KeyEvent(KeyEvent::KeyPress(data.vkCode)));
            }
            WPARAM(0x105) => {
                //系统键抬起
                counter.recv(Event::KeyEvent(KeyEvent::KeyUp(data.vkCode)));
            }
            _ => (),
        };
    }
    CallNextHookEx(KEYBOARD_HOOK, code, wparam, lparam)
}

unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mut counter = COUNTER.write().unwrap();

    let data = lparam.0 as *const MSLLHOOKSTRUCT;
    if !data.is_null() {
        if let Ok(mouse_event) = MouseEvent::try_from(wparam.0 as i32) {
            let point = Point {
                x: (*data).pt.x,
                y: (*data).pt.y,
            };
            let _ = counter.recv(Event::MouseEvent((mouse_event, point)));
        }
    }
    CallNextHookEx(MOUSE_HOOK, code, wparam, lparam)
}
