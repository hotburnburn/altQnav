use chrono::Local;
use notify_rust::Notification;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// 获取日志文件路径
fn get_log_path() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        PathBuf::from("altqnav.log")
    }

    #[cfg(not(debug_assertions))]
    {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.join("altqnav.log");
            }
        }
        PathBuf::from("altqnav.log")
    }
}

/// 写入错误日志
pub fn log_error(context: &str, error: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] ERROR - {}: {}\n", timestamp, context, error);

    // 写入日志文件
    if let Ok(log_path) = get_log_path()
        .canonicalize()
        .or_else(|_| Ok::<PathBuf, std::io::Error>(get_log_path()))
    {
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .and_then(|mut file| file.write_all(log_entry.as_bytes()));

        // 显示Windows通知
        let log_path_str = log_path.display().to_string();
        let _ = Notification::new()
            .summary("altQnav 错误")
            .body(&format!(
                "发生错误: {}\n详情请查看日志: {}",
                context, log_path_str
            ))
            .icon("dialog-error")
            .timeout(5000)
            .show();
    }
}

/// 写入警告日志
pub fn log_warning(context: &str, message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] WARN - {}: {}\n", timestamp, context, message);

    if let Ok(log_path) = get_log_path()
        .canonicalize()
        .or_else(|_| Ok::<PathBuf, std::io::Error>(get_log_path()))
    {
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .and_then(|mut file| file.write_all(log_entry.as_bytes()));
    }
}

/// 写入信息日志
pub fn log_info(context: &str, message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!("[{}] INFO - {}: {}\n", timestamp, context, message);

    if let Ok(log_path) = get_log_path()
        .canonicalize()
        .or_else(|_| Ok::<PathBuf, std::io::Error>(get_log_path()))
    {
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .and_then(|mut file| file.write_all(log_entry.as_bytes()));
    }
}

/// 清理旧日志（保留最近1MB）
pub fn cleanup_log() {
    let log_path = get_log_path();

    if let Ok(metadata) = fs::metadata(&log_path) {
        // 如果日志文件超过1MB，清空它
        if metadata.len() > 1_048_576 {
            let _ = fs::write(&log_path, "");
        }
    }
}
