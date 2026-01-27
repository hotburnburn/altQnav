// 模块声明
mod app_config;
mod app_launcher;
mod logger;
mod window_manager;
mod window_utils;

// 导入模块
use app_config::{check_app_status, get_app_list_with_status, load_monitored_apps, AppInfo};
use app_launcher::launch_or_focus_app_impl;
use logger::{cleanup_log, log_info};
use window_utils::calculate_window_size;

// Tauri commands
#[tauri::command]
fn get_app_list() -> Vec<AppInfo> {
    get_app_list_with_status()
}

#[tauri::command]
fn launch_or_focus_app(process_name: String, launch_path: String) -> Result<String, String> {
    launch_or_focus_app_impl(process_name, launch_path)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 清理旧日志
    cleanup_log();
    log_info("应用启动", "altQnav 正在启动...");

    tauri::Builder::default()
        .setup(|app| {
            // 启动时检测应用状态
            check_app_status();

            #[cfg(desktop)]
            {
                use tauri::Manager;

                // 根据应用数量动态计算窗口大小
                let app_count = load_monitored_apps().len();
                let (window_width, window_height) = calculate_window_size(app_count);

                let window = app.get_webview_window("main").unwrap();

                // 先设置最小尺寸限制
                let _ = window.set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize {
                    width: 400.0,
                    height: 100.0,
                })));

                // 设置窗口大小
                let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                    width: window_width as f64,
                    height: window_height as f64,
                }));

                // 创建系统托盘
                use tauri::{
                    menu::{Menu, MenuItem},
                    tray::TrayIconBuilder,
                };

                let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                let show_i = MenuItem::with_id(app, "show", "显示/隐藏", true, None::<&str>)?;

                let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

                let _tray = TrayIconBuilder::new()
                    .menu(&menu)
                    .tooltip("altQnav - Alt+Q 快速启动")
                    .icon(app.default_window_icon().unwrap().clone())
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            std::process::exit(0);
                        }
                        "show" => {
                            let window = app.get_webview_window("main").unwrap();
                            if window.is_visible().unwrap() {
                                window.hide().unwrap();
                            } else {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                        _ => {}
                    })
                    .build(app)?;

                // 全局快捷键
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };

                let alt_q_shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyQ);
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |app, shortcut, event| {
                            if shortcut == &alt_q_shortcut
                                && event.state() == ShortcutState::Released
                            {
                                use tauri::Manager;

                                let window = app.get_webview_window("main").unwrap();
                                if window.is_visible().unwrap() {
                                    window.hide().unwrap();
                                } else {
                                    // 先显示窗口，不等待状态检测
                                    window.show().unwrap();
                                    window.set_focus().unwrap();

                                    // 异步触发前端刷新（通过focus事件自动触发）
                                    // 前端会调用get_app_list获取最新状态

                                    // 确保窗口真正获得焦点（在 Windows 上有时需要额外的激活）
                                    #[cfg(windows)]
                                    {
                                        use std::thread;
                                        use std::time::Duration;
                                        let window_clone = window.clone();
                                        thread::spawn(move || {
                                            thread::sleep(Duration::from_millis(50));
                                            let _ = window_clone.set_focus();
                                        });
                                    }
                                }
                            }
                        })
                        .build(),
                )?;

                app.global_shortcut().register(alt_q_shortcut)?;
            }

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_list,
            launch_or_focus_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
