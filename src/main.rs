use font_kit::source::SystemSource;
use minifb::{Key, Scale, Window, WindowOptions, Icon};
use raqote::{DrawTarget, Point, DrawOptions, SolidSource, Source};
use windows::Win32::{UI::WindowsAndMessaging::{SetWindowsHookExW, WINDOWS_HOOK_ID, HOOKPROC}, Foundation::{HINSTANCE, WPARAM, LPARAM, LRESULT}};
use std::{time::Duration, str::FromStr};
use anyhow::Result;
const WIDTH: usize = 640;
const HEIGHT: usize = 480;

fn main() -> Result<()> {
    let mut window = Window::new(
        "按键统计",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            scale: Scale::X1,
            ..WindowOptions::default()
        },
    )?;

    let icon = Icon::from_str("icon.ico").unwrap();
    window.set_icon(icon);

    window.limit_update_rate(Some(Duration::from_millis(100)));

    let font = SystemSource::new().all_fonts()?.get(0).unwrap().load()?;
    
    let size = window.get_size();
    let mut dt = DrawTarget::new(size.0 as i32, size.1 as i32);
    let text_color = Source::Solid(SolidSource::from_unpremultiplied_argb(0xff, 0xFF, 0xFF, 0xFF));
    dt.draw_text(&font, 14., "hello", Point::new(20., 20.)  , &text_color, &DrawOptions::default());

    unsafe{
        /*
        参数1：idHook。表示我们需要钩取哪种类型的事件。数值13表示全局键盘事件，数值14表示全局鼠标事件，其它事件值不在本文讨论范围内，有需要的同学请自行查阅官方文档。
        参数2：lpfn。在C#中就是一个委托类型值，填入要注册的钩子函数名。具体的委托类型会在后面说明。
        参数3：hmod。无须过多理会，表示持有钩子函数的进程号，填0再强转为IntPtr即可。
        参数4：dwThreadId。无须过多理会，直接填0即可。 
         */
        let res = SetWindowsHookExW(WINDOWS_HOOK_ID(13), HOOKPROC::Some(keyboard_hook_proc), HINSTANCE::default(), 0)?;
        println!("keyboard hhook:{:?}", res);

        let res = SetWindowsHookExW(WINDOWS_HOOK_ID(14), HOOKPROC::Some(mouse_hook_proc), HINSTANCE::default(), 0)?;
        println!("mouse hhook:{:?}", res);
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(dt.get_data(), size.0, size.1)?;
    }

    Ok(())
}

/// https://www.cnblogs.com/chorm590/p/14199978.html
/// 参数1：nCode。事件状态码，当值为0时处理按键事件，小于0时最好将事件交由 CallNextHookEx 函数处理。
/// 参数2：wParam。按键事件码，有四个可能值：1、普通键按下：0x100；2、普通键抬起：0x101；3、系统键按下：0x104；4、系统键抬起：0x105。在本文中我们只需关心前两个事件。
/// 参数3：lParam。事件详细信息结构体的地址
unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT{
    println!("键盘事件 code={code} wparam={:?} lparam={:?}", wparam, lparam);
    LRESULT::default()
}

unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT{
    println!("鼠标事件 code={code} wparam={:?} lparam={:?}", wparam, lparam);
    LRESULT::default()
}