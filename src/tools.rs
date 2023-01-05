use std::{
    fs::File,
    io::{Read, Write},
    mem,
    path::{Path, PathBuf},
    thread::JoinHandle,
};

use anyhow::{anyhow, Result};
use bzip2::write::BzDecoder;
use directories::ProjectDirs;
use font_kit::font::Font;
use minifb::Window;
use pathfinder_geometry::vector::vec2f;
use raqote::{DrawOptions, DrawTarget, Point, Source};
use windows::Win32::{
    Foundation::{BOOL, HINSTANCE, HWND, LPARAM, LRESULT, MAX_PATH, WPARAM},
    UI::{
        Shell::{SHGetSpecialFolderPathW, CSIDL_STARTUP},
        WindowsAndMessaging::{
            CreateIcon, GetDesktopWindow, SendMessageW, SetForegroundWindow, SetWindowsHookExW,
            ShowWindow, UnhookWindowsHookEx, HHOOK, HICON, HOOKPROC, ICON_BIG, ICON_SMALL, SW_HIDE,
            SW_SHOWNORMAL, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_SETICON,
        },
    },
};

use crate::counter::Counter;

type HookFn = unsafe extern "system" fn(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;

const PIXEL_SIZE: usize = 4;

pub static mut MOUSE_HOOK: HHOOK = HHOOK(0);
pub static mut KEYBOARD_HOOK: HHOOK = HHOOK(0);

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct Pixel {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) a: u8,
}

impl Pixel {
    fn convert_to_bgra(&mut self) {
        mem::swap(&mut self.r, &mut self.b);
    }
}

pub fn load_tray_icon_from_memory(
    rgba: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<tray_icon::icon::Icon> {
    let rgba = decompress(&rgba)?;
    Ok(tray_icon::icon::Icon::from_rgba(rgba, width, height)?)
}

pub fn load_icon_from_memory(rgba: Vec<u8>, width: u32, height: u32) -> Result<HICON> {
    let rgba = decompress(&rgba)?;

    let pixel_count = rgba.len() / PIXEL_SIZE;
    let mut and_mask = Vec::with_capacity(pixel_count);
    let pixels =
        unsafe { std::slice::from_raw_parts_mut(rgba.as_ptr() as *mut Pixel, pixel_count) };
    for pixel in pixels {
        and_mask.push(pixel.a.wrapping_sub(std::u8::MAX)); // invert alpha channel
        pixel.convert_to_bgra();
    }
    assert_eq!(and_mask.len(), pixel_count);
    let handle = unsafe {
        CreateIcon(
            HINSTANCE(0),
            width as i32,
            height as i32,
            1,
            (PIXEL_SIZE * 8) as u8,
            and_mask.as_ptr(),
            rgba.as_ptr(),
        )?
    };

    Ok(handle)
}

pub fn set_window_icon(window: &Window, icon: HICON) -> Result<()> {
    let handle = HWND(window.get_window_handle() as isize);
    unsafe {
        SendMessageW(
            handle,
            WM_SETICON,
            WPARAM(ICON_BIG.try_into()?),
            LPARAM(icon.0),
        );
        SendMessageW(
            handle,
            WM_SETICON,
            WPARAM(ICON_SMALL.try_into()?),
            LPARAM(icon.0),
        );
    }
    Ok(())
}

pub fn hide_window(window: &Window) -> BOOL {
    let handle = HWND(window.get_window_handle() as isize);
    unsafe { ShowWindow(handle, SW_HIDE) }
}

pub fn show_window(window: &Window) {
    let handle = HWND(window.get_window_handle() as isize);
    //显示窗口
    unsafe {
        ShowWindow(handle, SW_SHOWNORMAL);
        SetForegroundWindow(handle);
    }
}

pub fn set_keyboard_hook(f: HookFn) -> Result<()> {
    unsafe {
        KEYBOARD_HOOK =
            SetWindowsHookExW(WH_KEYBOARD_LL, HOOKPROC::Some(f), HINSTANCE::default(), 0)?
    };
    Ok(())
}

pub fn set_mouse_hook(f: HookFn) -> Result<()> {
    unsafe {
        MOUSE_HOOK = SetWindowsHookExW(WH_MOUSE_LL, HOOKPROC::Some(f), HINSTANCE::default(), 0)?
    };
    Ok(())
}

pub fn remove_keyboard_hook() {
    unsafe {
        let _ = UnhookWindowsHookEx(KEYBOARD_HOOK);
    }
}

pub fn remove_mouse_hook() {
    unsafe {
        let _ = UnhookWindowsHookEx(MOUSE_HOOK);
    }
}

///解压缩到字节
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decompressor = BzDecoder::new(vec![]);
    decompressor.write_all(data)?;
    Ok(decompressor.finish()?)
}

pub fn get_app_dir() -> Result<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("planet", "planet", "keyboard-counter") {
        let path = proj_dirs.config_dir();
        //创建目录
        std::fs::create_dir_all(path)?;
        Ok(path.to_path_buf())
    } else {
        Ok(PathBuf::from("./").to_path_buf())
    }
}

pub fn get_storage_path() -> String {
    let mut app_dir = get_app_dir().unwrap_or(PathBuf::from("./"));
    app_dir.push("keyboard-counter.bin");
    app_dir.to_str().unwrap().to_string()
}

pub fn save_storage_async(data: &Counter) -> Result<JoinHandle<()>> {
    let encoded: Vec<u8> = bincode::serialize(data)?;
    Ok(std::thread::spawn(move || {
        if let Ok(mut cfg_file) = File::create(&get_storage_path()) {
            let res = cfg_file.write_all(&encoded);
            println!("写入了配置文件:{:?}", res);
        }
    }))
}

pub fn save_storage(data: &Counter) -> Result<()> {
    let mut cfg_file = File::create(&get_storage_path())?;
    let encoded: Vec<u8> = bincode::serialize(data)?;
    cfg_file.write_all(&encoded)?;
    Ok(())
}

pub fn read_storage() -> Result<Counter> {
    let mut cfg_file = File::open(&get_storage_path())?;
    let mut encoded = vec![];
    cfg_file.read_to_end(&mut encoded)?;
    let decoded: Counter = bincode::deserialize(&encoded[..])?;
    // println!("读取到counter:{:?}", decoded);
    Ok(decoded)
}

static TEMPLATE: &str = r"[InternetShortcut]
URL=--
IconIndex=0
IconFile=--
";

pub fn register_app_for_startup(app_name: &str) -> Result<()> {
    let hwnd = unsafe { GetDesktopWindow() };
    let mut path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    unsafe { SHGetSpecialFolderPathW(hwnd, &mut path, CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    let url_file = format!("{}\\{}.url", path, app_name);
    //写入url文件
    let mut file = std::fs::File::create(url_file)?;
    let exe_path = ::std::env::current_exe()?;
    if let Some(exe_path) = exe_path.to_str() {
        file.write_all(TEMPLATE.replace("--", exe_path).as_bytes())?;
        Ok(())
    } else {
        Err(anyhow!("exe路径读取失败!"))
    }
}

pub fn is_app_registered_for_startup(app_name: &str) -> Result<bool> {
    let hwnd = unsafe { GetDesktopWindow() };
    let mut path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    unsafe { SHGetSpecialFolderPathW(hwnd, &mut path, CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    Ok(Path::new(&format!("{}\\{}.url", path, app_name)).exists())
}

pub fn remove_app_for_startup(app_name: &str) -> Result<()> {
    let hwnd = unsafe { GetDesktopWindow() };
    let mut path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    unsafe { SHGetSpecialFolderPathW(hwnd, &mut path, CSIDL_STARTUP as i32, false) };
    let path = String::from_utf16(&path)?.replace("\u{0}", "");
    std::fs::remove_file(format!("{}\\{}.url", path, app_name))?;
    Ok(())
}

#[macro_export]
macro_rules! alert {
    ($s:literal) => {{
        unsafe { MessageBoxW(None, w!($s), w!("温馨提示"), MB_OK) };
    }};
}

pub fn draw_text(
    dt: &mut DrawTarget,
    font: &Font,
    point_size: f32,
    text: &str,
    start: Point,
    src: &Source,
    options: &DrawOptions,
) {
    let mut start = vec2f(start.x, start.y);
    let mut ids = Vec::new();
    let mut positions = Vec::new();
    for c in text.chars() {
        let id = font.glyph_for_char(c).unwrap();
        ids.push(id);
        positions.push(Point::new(start.x(), start.y()));
        start += font.advance(id).unwrap() * point_size / point_size / 96.;
    }
    dt.draw_glyphs(font, point_size, &ids, &positions, src, options);
}

pub fn measure_text(font: &Font, point_size: f32, text: &str) -> Point {
    let mut start = vec2f(0., 0.);
    let mut ids = Vec::new();
    let mut positions = Vec::new();
    for c in text.chars() {
        let id = font.glyph_for_char(c).unwrap();
        ids.push(id);
        positions.push(Point::new(start.x(), start.y()));
        start += font.advance(id).unwrap() * point_size / point_size / 96.;
    }
    Point::new(start.x(), start.y())
}
