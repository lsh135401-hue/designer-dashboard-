use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, PhysicalPosition, WindowEvent,
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
            let dashboard_item = MenuItem::with_id(app, "dashboard", "팝오버 열기/닫기", true, None::<&str>)?;
            let fullview_item = MenuItem::with_id(app, "fullview", "전체 대시보드 (웹)", true, None::<&str>)?;
            let sketches_item = MenuItem::with_id(app, "sketches", "레이아웃 비교 (웹)", true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let about_item = MenuItem::with_id(app, "about", "GitHub Repo", true, None::<&str>)?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Designer Dashboard 종료", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[
                    &dashboard_item,
                    &fullview_item,
                    &sketches_item,
                    &sep1,
                    &about_item,
                    &sep2,
                    &quit_item,
                ],
            )?;

            // 메인 윈도우 — 포커스 잃으면 자동 숨김 (팝오버 동작)
            if let Some(window) = app.get_webview_window("main") {
                let win_clone = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Focused(false) = event {
                        let _ = win_clone.hide();
                    }
                });
            }

            // 트레이 아이콘
            let _tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("Designer Dashboard")
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(true) // macOS 메뉴바 라이트/다크 모드 자동 색상
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "dashboard" => toggle_main_window(app, None),
                    "fullview" => {
                        let _ = app.opener().open_url(
                            "https://lsh135401-hue.github.io/designer-dashboard-/.planning/sketches/v0-dashboard.html#main",
                            None::<&str>,
                        );
                    }
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
                        position,
                        ..
                    } = event
                    {
                        toggle_main_window(tray.app_handle(), Some(position));
                    }
                })
                .build(app)?;

            // 첫 실행 — 윈도우 숨김
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn toggle_main_window(app: &tauri::AppHandle, tray_pos: Option<tauri::PhysicalPosition<f64>>) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            _ => {
                if let Some(pos) = tray_pos {
                    position_window_under_tray(&window, pos);
                }
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
}

fn position_window_under_tray(window: &tauri::WebviewWindow, tray_pos: tauri::PhysicalPosition<f64>) {
    // 트레이 아이콘 바로 아래 + 살짝 좌측으로 정렬해서 화면 안에 들어오게
    if let Ok(size) = window.outer_size() {
        let scale = window.scale_factor().unwrap_or(1.0);
        let win_w = size.width as f64;
        // 트레이 아이콘 X를 중심으로 윈도우 가로 절반만큼 왼쪽으로 시프트
        let mut x = tray_pos.x - (win_w / 2.0);
        // 화면 우측 가장자리 안 넘게 보정 (메인 모니터 width 가져오기)
        if let Ok(Some(monitor)) = window.current_monitor() {
            let m_size = monitor.size();
            let m_pos = monitor.position();
            let max_x = (m_pos.x as f64) + (m_size.width as f64) - win_w - 10.0;
            let min_x = (m_pos.x as f64) + 10.0;
            if x > max_x { x = max_x; }
            if x < min_x { x = min_x; }
        }
        // 트레이 아래 4px 오프셋
        let y = tray_pos.y + 4.0;
        let _ = window.set_position(PhysicalPosition::new(x as i32, y as i32));
        let _ = scale; // unused warning 회피
    }
}
