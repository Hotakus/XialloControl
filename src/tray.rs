use std::thread;
use std::time::Duration;
use tauri::menu;
use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent, menu::Menu, menu::MenuEvent};
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoopBuilder};

enum UserEvent {
    TrayIconEvent(tray_icon::TrayIconEvent),
    MenuEvent(tray_icon::menu::MenuEvent),
}

fn load_icon(path: &str, size: Option<(u32, u32)>) -> Icon {
    Icon::from_path(path, size).unwrap()
}

fn tray_thread() {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(100));
    });
}

pub fn initialize() {
    log::info!("tray initializing...");

    // use UserEvent
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // event proxy
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::TrayIconEvent(event));
    }));

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    // create tray
    // 定义菜单项
    // let show_settings = CustomMenuItem::new("settings".to_string(), "打开设置");
    // let quit = CustomMenuItem::new("quit".to_string(), "退出");
    let tray_menu = Menu::new();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("system-tray - tray icon library!")
        .with_icon(load_icon("./icons/icon.ico", None))
        .build()
        .unwrap();

    // event loop
    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    event_loop.run(move |event, _event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::TrayIconEvent(tray_event)) => {
                match tray_event {
                    TrayIconEvent::Click { id, position, rect, button, button_state } => {
                        log::info!("🔘 Tray Click: id={id:?}, button={button:?}, pos={position:?}, state={button_state:?}");
                    }
                    TrayIconEvent::DoubleClick { id, position, rect, button } => {
                        log::info!("🖱️ Tray DoubleClick: id={id:?}, button={button:?}");
                    }
                    TrayIconEvent::Enter { id, position, rect } => {
                        // log::info!("➡️ Tray Enter: id={id:?}");
                    }
                    TrayIconEvent::Move { id, position, rect } => {
                        // log::info!("🕹️ Tray Move: id={id:?}, pos={position:?}");
                    }
                    TrayIconEvent::Leave { id, position, rect } => {
                        // log::info!("⬅️ Tray Leave: id={id:?}");
                    }
                    // 👇 这个分支是必须的，因为枚举被标记为 non_exhaustive
                    _ => {
                        log::warn!("⚠️ 未处理的 TrayIconEvent: {:?}", tray_event);
                    }
                }
            }
            _ => {}
        }
    });
}
