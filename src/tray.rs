// use std::thread;
// use std::time::Duration;
// use tauri::menu;
// use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent, menu::Menu, menu::MenuEvent};
// use winit::event::Event;
// use winit::event_loop::{ControlFlow, EventLoopBuilder};
//
// enum UserEvent {
//     TrayIconEvent(tray_icon::TrayIconEvent),
//     MenuEvent(tray_icon::menu::MenuEvent),
// }
//
// fn load_icon(path: &str, size: Option<(u32, u32)>) -> Icon {
//     Icon::from_path(path, size).unwrap()
// }
//
// fn tray_thread() {
//     thread::spawn(move || {
//         thread::sleep(Duration::from_secs(100));
//     });
// }
//
// pub fn initialize() {
//     log::info!("tray initializing...");
//
//     // use UserEvent
//     let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
//
//     // event proxy
//     let proxy = event_loop.create_proxy();
//     TrayIconEvent::set_event_handler(Some(move |event| {
//         let _ = proxy.send_event(UserEvent::TrayIconEvent(event));
//     }));
//
//     let proxy = event_loop.create_proxy();
//     MenuEvent::set_event_handler(Some(move |event| {
//         let _ = proxy.send_event(UserEvent::MenuEvent(event));
//     }));
//
//     // create tray
//     // 定义菜单项
//     // let show_settings = CustomMenuItem::new("settings".to_string(), "打开设置");
//     // let quit = CustomMenuItem::new("quit".to_string(), "退出");
//     let tray_menu = Menu::new();
//
//     let tray_icon = TrayIconBuilder::new()
//         .with_menu(Box::new(tray_menu))
//         .with_tooltip("system-tray - tray icon library!")
//         .with_icon(load_icon("./icons/icon.ico", None))
//         // .with_menu_on_left_click(true)
//         .build()
//         .unwrap();
//
//     // event loop
//     let menu_channel = MenuEvent::receiver();
//     let tray_channel = TrayIconEvent::receiver();
//     event_loop.run(move |event, _event_loop, control_flow| {
//         *control_flow = ControlFlow::Wait;
//
//         match event {
//             Event::UserEvent(UserEvent::TrayIconEvent(tray_event)) => {
//                 match tray_event {
//                     TrayIconEvent::Click { id, position, rect, button, button_state } => {
//                         log::info!("🔘 Tray Click: id={id:?}, button={button:?}, pos={position:?}, state={button_state:?}");
//                     }
//                     TrayIconEvent::DoubleClick { id, position, rect, button } => {
//                         log::info!("🖱️ Tray DoubleClick: id={id:?}, button={button:?}");
//                     }
//                     TrayIconEvent::Enter { id, position, rect } => {
//                         // log::info!("➡️ Tray Enter: id={id:?}");
//                     }
//                     TrayIconEvent::Move { id, position, rect } => {
//                         // log::info!("🕹️ Tray Move: id={id:?}, pos={position:?}");
//                     }
//                     TrayIconEvent::Leave { id, position, rect } => {
//                         // log::info!("⬅️ Tray Leave: id={id:?}");
//                     }
//                     // 👇 这个分支是必须的，因为枚举被标记为 non_exhaustive
//                     _ => {
//                         log::warn!("⚠️ 未处理的 TrayIconEvent: {:?}", tray_event);
//                     }
//                 }
//             }
//             _ => {}
//         }
//     });
// }

use std::error::Error;
use tauri::{App, AppHandle, Manager};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};

pub fn initialize(app: AppHandle) -> Result<(), Box<dyn Error>> {
    let quit_i = MenuItem::with_id(&app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(&app, &[&quit_i])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                log::debug!("quit menu item was clicked");
                app.exit(0);
            }
            _ => {
                log::debug!("menu item {:?} not handled", event.id);
            }
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                // button_state: MouseButtonState::Up,
                ..
            } => {
                log::debug!("double click on tray icon");
                // in this example, let's show and focus the main window when the tray is clicked
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {
                // println!("unhandled event {event:?}");
            }
        })
        .build(&app)?;
    Ok(())
}
