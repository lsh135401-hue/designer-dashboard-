use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_opener::OpenerExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // macOS: 메뉴바 액세서리 앱 — Dock에 아이콘 표시 안 함
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // 트레이 메뉴 (오른쪽 클릭 시)
            let dashboard_item = MenuItem::with_id(app, "dashboard", "대시보드 열기", true, None::<&str>)?;
            let sketches_item = MenuItem::with_id(app, "sketches", "전체 목업 (웹) 열기", true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let about_item = MenuItem::with_id(app, "about", "Designer Dashboard 정보", true, None::<&str>)?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Designer Dashboard 종료", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&dashboard_item, &sketches_item, &sep1, &about_item, &sep2, &quit_item],
            )?;

            // 트레이 아이콘
            let _tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("Designer Dashboard")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false) // 왼쪽 클릭 = 윈도우 토글, 오른쪽 클릭 = 메뉴
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "dashboard" => show_main_window(app),
                    "sketches" => {
                        let _ = app.opener().open_url(
                            "https://lsh135401-hue.github.io/designer-dashboard-/.planning/sketches/index.html",
                            None::<&str>,
                        );
                    }
                    "about" => {
                        let _ = app.opener().open_url(
                            "https://github.com/lsh135401-hue/designer-dashboard-",
                            None::<&str>,
                        );
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // 첫 실행 — 사용자가 트레이 아이콘 클릭하기 전까진 윈도우 숨김
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            _ => {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }
    }
}
