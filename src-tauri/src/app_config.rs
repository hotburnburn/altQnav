use crate::logger::{log_error, log_warning};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::fs;
use std::path::Path;
use sysinfo::System;

#[cfg(not(debug_assertions))]
use std::env;

#[derive(Serialize, Clone)]
pub struct AppInfo {
    pub process_name: String,
    pub display_name: String,
    pub launch_path: String,
    pub is_running: bool,
    pub icon_path: Option<String>,
}

// 应用启动时读取配置文件，缓存起来
static APP_CONFIG: Lazy<Vec<(String, String, String)>> = Lazy::new(|| load_monitored_apps_impl());

/// 从缓存获取应用列表
pub fn load_monitored_apps() -> Vec<(String, String, String)> {
    APP_CONFIG.clone()
}

/// 从 target_app/target_app.txt 读取应用列表 (三段式格式) - 内部实现
fn load_monitored_apps_impl() -> Vec<(String, String, String)> {
    let file_path = Path::new("target_app/target_app.txt");

    match fs::read_to_string(file_path) {
        Ok(content) => content
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return None;
                }

                // 解析 CSV 格式,支持带引号的路径
                let mut parts = Vec::new();
                let mut current = String::new();
                let mut in_quotes = false;

                for ch in line.chars() {
                    match ch {
                        '"' => in_quotes = !in_quotes,
                        ',' if !in_quotes => {
                            parts.push(current.trim().to_string());
                            current.clear();
                        }
                        _ => current.push(ch),
                    }
                }
                parts.push(current.trim().to_string());

                if parts.len() == 3 {
                    Some((parts[0].clone(), parts[1].clone(), parts[2].clone()))
                } else {
                    log_warning("配置解析", &format!("配置行格式错误: {}", line));
                    None
                }
            })
            .collect(),
        Err(e) => {
            log_error(
                "配置文件加载",
                &format!("无法读取 target_app/target_app.txt: {}", e),
            );
            vec![(
                "Code.exe".to_string(),
                "VSCode".to_string(),
                "code".to_string(),
            )]
        }
    }
}

/// 检测应用运行状态
pub fn check_app_status() {
    let _monitored_apps = load_monitored_apps();
    let mut _sys = System::new();
    _sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    // 状态检测逻辑已移除打印，仅保留函数供兼容性调用
}

/// 获取应用列表及运行状态
pub fn get_app_list_with_status() -> Vec<AppInfo> {
    let monitored_apps = load_monitored_apps();
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    monitored_apps
        .into_iter()
        .map(|(process_name, display_name, launch_path)| {
            let is_running = sys
                .processes()
                .values()
                .any(|p| p.name().to_string_lossy() == process_name);

            // 检查图标是否存在
            let icon_path = check_icon_exists(&display_name);

            AppInfo {
                process_name,
                display_name,
                launch_path,
                is_running,
                icon_path,
            }
        })
        .collect()
}

/// 检查应用图标是否存在，返回绝对路径
fn check_icon_exists(display_name: &str) -> Option<String> {
    // 开发环境：从当前工作目录查找
    #[cfg(debug_assertions)]
    {
        let icon_path = Path::new("target_app").join(format!("{}.png", display_name));
        if icon_path.exists() {
            let abs_path = icon_path.canonicalize().ok()?;
            let mut path_str = abs_path.to_str()?.to_string();

            // Windows长路径前缀处理：去掉 \\?\ 前缀
            if path_str.starts_with(r"\\?\") {
                path_str = path_str.replacen(r"\\?\", "", 1);
            }

            return Some(path_str);
        }
        return None;
    }

    // 生产环境：从可执行文件所在目录查找
    #[cfg(not(debug_assertions))]
    {
        let exe_dir = env::current_exe().ok()?.parent()?.to_path_buf();

        let icon_path = exe_dir
            .join("target_app")
            .join(format!("{}.png", display_name));

        if icon_path.exists() {
            icon_path.to_str().map(|s: &str| s.to_string())
        } else {
            None
        }
    }
}
