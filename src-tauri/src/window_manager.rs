#[cfg(windows)]
pub fn focus_window_by_process_name(process_name: &str) -> Result<(), String> {
    use sysinfo::System;
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        BringWindowToTop, EnumWindows, IsIconic, SetForegroundWindow, SetWindowPos, ShowWindow,
        HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SW_RESTORE, SW_SHOW,
    };

    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    // 获取目标进程的所有 PID
    let target_pids: Vec<u32> = sys
        .processes()
        .values()
        .filter(|p| p.name().to_string_lossy() == process_name)
        .map(|p| p.pid().as_u32())
        .collect();

    if target_pids.is_empty() {
        return Err(format!("未找到进程: {}", process_name));
    }

    unsafe {
        let mut hwnd_result: Option<HWND> = None;

        // 枚举所有窗口，查找属于任意一个目标进程的窗口
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut (target_pids, &mut hwnd_result) as *mut _ as isize),
        );

        if let Some(hwnd) = hwnd_result {
            // 多步骤确保窗口被唤起到前台

            // 1. 如果窗口是最小化状态，恢复它；否则只激活不改变大小
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            } else {
                let _ = ShowWindow(hwnd, SW_SHOW);
            }

            // 2. 将窗口带到顶部
            let _ = BringWindowToTop(hwnd);

            // 3. 临时设为最顶层窗口确保它在最上面
            let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);

            // 4. 取消最顶层属性，但保持在其他窗口上面
            let _ = SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);

            // 5. 设置为前台窗口并激活
            let _ = SetForegroundWindow(hwnd);

            Ok(())
        } else {
            Err(format!("未找到 {} 的窗口", process_name))
        }
    }
}

#[cfg(windows)]
unsafe extern "system" fn enum_windows_callback(
    hwnd: windows::Win32::Foundation::HWND,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowTextLengthW, GetWindowThreadProcessId, IsWindowVisible,
    };

    let (target_pids, hwnd_result) = &mut *(lparam.0 as *mut (Vec<u32>, &mut Option<HWND>));

    // 只处理可见窗口
    if IsWindowVisible(hwnd).as_bool() {
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        // 检查是否属于任意一个目标进程
        if target_pids.contains(&process_id) {
            // 检查窗口标题长度
            let title_len = GetWindowTextLengthW(hwnd);

            // 优先选择有标题的窗口（主窗口通常有标题）
            if title_len > 0 {
                **hwnd_result = Some(hwnd);
                return windows::Win32::Foundation::BOOL(0); // 找到有标题的窗口，停止枚举
            } else if hwnd_result.is_none() {
                // 如果还没找到任何窗口，就先记录这个无标题窗口作为备选
                **hwnd_result = Some(hwnd);
            }
        }
    }

    windows::Win32::Foundation::BOOL(1) // 继续枚举
}
