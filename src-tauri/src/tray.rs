use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri::LogicalSize;

use crate::state::SharedScale;

const BASE_W: f64 = 720.0;
const BASE_H: f64 = 520.0;

pub fn setup(app: &tauri::App, scale: SharedScale) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "显示/隐藏", true, None::<&str>)?;
    let scale_033x = MenuItem::with_id(app, "scale_033x", "缩放 1/3x", true, None::<&str>)?;
    let scale_05x = MenuItem::with_id(app, "scale_05x", "缩放 0.5x", true, None::<&str>)?;
    let scale_075x = MenuItem::with_id(app, "scale_075x", "缩放 0.75x", true, None::<&str>)?;
    let scale_1x = MenuItem::with_id(app, "scale_1x", "缩放 1x", true, None::<&str>)?;
    let about_item = MenuItem::with_id(app, "about", "关于", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &scale_033x,
            &scale_05x,
            &scale_075x,
            &scale_1x,
            &about_item,
            &quit_item,
        ],
    )?;

    let _tray = TrayIconBuilder::with_id("tianyi-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("luotianyi") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                "scale_033x" => {
                    *scale.lock().unwrap() = 0.333;
                    if let Some(window) = app.get_webview_window("luotianyi") {
                        let _ = window.set_size(LogicalSize::new((BASE_W * 0.333).round(), (BASE_H * 0.333).round()));
                    }
                }
                "scale_05x" => {
                    *scale.lock().unwrap() = 0.5;
                    if let Some(window) = app.get_webview_window("luotianyi") {
                        let _ = window.set_size(LogicalSize::new((BASE_W * 0.5).round(), (BASE_H * 0.5).round()));
                    }
                }
                "scale_075x" => {
                    *scale.lock().unwrap() = 0.75;
                    if let Some(window) = app.get_webview_window("luotianyi") {
                        let _ = window.set_size(LogicalSize::new((BASE_W * 0.75).round(), (BASE_H * 0.75).round()));
                    }
                }
                "scale_1x" => {
                    *scale.lock().unwrap() = 1.0;
                    if let Some(window) = app.get_webview_window("luotianyi") {
                        let _ = window.set_size(LogicalSize::new(BASE_W.round(), BASE_H.round()));
                    }
                }
                "about" => {
                    let _ = open::that("https://blog.starryfirefly.top/");
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("luotianyi") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
