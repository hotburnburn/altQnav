use crate::logger::log_error;
use std::process::Command;
use sysinfo::System;

#[cfg(windows)]
use crate::window_manager::focus_window_by_process_name;

/// 启动应用或聚焦已运行的应用
pub fn launch_or_focus_app_impl(
    process_name: String,
    launch_path: String,
) -> Result<String, String> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    // 检查应用是否运行
    let is_running = sys
        .processes()
        .values()
        .any(|p| p.name().to_string_lossy() == process_name);

    if is_running {
        // 应用已运行,尝试切换窗口
        #[cfg(windows)]
        {
            if let Err(e) = focus_window_by_process_name(&process_name) {
                // 可能是托盘那种，尝试启动一下，实际上会叫出来
                if let Ok(_) = launch(process_name.clone(), launch_path.clone()) {
                    return Ok(format!("已启动 {}", process_name));
                }

                let error_msg = format!("切换窗口失败: {}", e);
                log_error(&format!("窗口切换: {}", process_name), &error_msg);
                return Err(error_msg);
            }
            Ok(format!("已切换到 {}", process_name))
        }
        #[cfg(not(windows))]
        {
            Ok(format!("{} 已在运行中", process_name))
        }
    } else {
        // 应用未运行,启动应用
        launch(process_name, launch_path)
    }
}

fn launch(process_name: String, launch_path: String) -> Result<String, String> {
    // 检查是否是协议链接（如 calculator:, ms-settings: 等）
    if launch_path.contains(':') && !launch_path.starts_with('"') && !launch_path.contains('\\') {
        // 这是一个协议链接，使用 cmd /c start 启动
        match Command::new("cmd")
            .args(&["/c", "start", "", &launch_path])
            .spawn()
        {
            Ok(_) => Ok(format!("已启动 {}", process_name)),
            Err(e) => {
                let error_msg = format!("启动失败: {}", e);
                log_error(&format!("应用启动(协议): {}", process_name), &error_msg);
                Err(error_msg)
            }
        }
    } else {
        // 普通可执行文件路径
        match Command::new(&launch_path).spawn() {
            Ok(_) => Ok(format!("已启动 {}", process_name)),
            Err(e) => {
                let error_msg = format!("启动失败: {}", e);
                log_error(
                    &format!("应用启动: {}", process_name),
                    &format!("{}, 路径: {}", error_msg, launch_path),
                );
                Err(error_msg)
            }
        }
    }
}
