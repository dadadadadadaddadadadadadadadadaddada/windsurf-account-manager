use std::process::Command;
use std::thread;
use std::time::Duration;

pub fn is_running() -> bool {
    #[cfg(target_os = "macos")]
    {
        Command::new("pgrep")
            .args(["-x", "Windsurf"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq Windsurf.exe", "/NH"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("Windsurf.exe"))
            .unwrap_or(false)
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("pgrep")
            .args(["-x", "windsurf"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// 检测是否还有任何 Windsurf 相关进程（含 Helper/GPU/Crashpad）
#[cfg(target_os = "macos")]
fn has_any_windsurf_process() -> bool {
    Command::new("sh")
        .args(["-c", "pgrep -f 'Windsurf.app/Contents' 2>/dev/null"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 用 SIGTERM 清理残留 Helper 进程（不会触发崩溃恢复对话框）
#[cfg(target_os = "macos")]
fn cleanup_helper_processes() {
    eprintln!("[cleanup] macOS: SIGTERM 清理残留 Helper...");
    let _ = Command::new("sh").args(["-c", "pkill -15 -f 'Windsurf.app/Contents/Frameworks' 2>/dev/null || true"]).output();
    let _ = Command::new("sh").args(["-c", "pkill -15 -f 'Windsurf Helper' 2>/dev/null || true"]).output();
    let _ = Command::new("sh").args(["-c", "pkill -15 -f 'Windsurf.app/Contents/MacOS' 2>/dev/null || true"]).output();
}

pub fn close_windsurf() -> Result<(), String> {
    if !is_running() {
        eprintln!("[close_windsurf] Windsurf 主进程未运行");
        // 主进程不在但可能有残留 Helper，一并清理
        #[cfg(target_os = "macos")]
        if has_any_windsurf_process() {
            eprintln!("[close_windsurf] 检测到残留 Helper，清理中...");
            cleanup_helper_processes();
            thread::sleep(Duration::from_secs(2));
        }
        return Ok(());
    }
    eprintln!("[close_windsurf] 检测到 Windsurf 正在运行，开始关闭...");

    #[cfg(target_os = "macos")]
    {
        // Step 1: AppleScript 优雅退出（与老项目完全一致，不触发崩溃恢复）
        eprintln!("[close_windsurf] macOS: AppleScript 退出...");
        let _ = Command::new("osascript")
            .args(["-e", r#"tell application "Windsurf" to quit"#])
            .output();

        // 等待主进程退出（最多 5 秒，与老项目一致）
        for i in 0..5 {
            thread::sleep(Duration::from_secs(1));
            if !is_running() {
                eprintln!("[close_windsurf] macOS: 主进程已退出 ({}秒)", i + 1);
                break;
            }
            eprintln!("[close_windsurf] macOS: 等待主进程退出... ({}/5)", i + 1);
        }

        // Step 2: 主进程还在 → SIGTERM（仍然不是 SIGKILL，不触发崩溃）
        if is_running() {
            eprintln!("[close_windsurf] macOS: AppleScript 超时，SIGTERM...");
            let _ = Command::new("sh").args(["-c",
                "pkill -15 -f 'Windsurf.app/Contents/MacOS/Windsurf' 2>/dev/null || true"
            ]).output();
            for i in 0..3 {
                thread::sleep(Duration::from_secs(1));
                if !is_running() {
                    eprintln!("[close_windsurf] macOS: 主进程已退出 (SIGTERM, {}秒)", i + 1);
                    break;
                }
            }
        }

        // Step 3: 主进程已退出，等待 Helper 自然退出
        // 优雅退出后 Helper 会自行关闭，但需要时间
        eprintln!("[close_windsurf] macOS: 等待 Helper 进程自然退出...");
        for i in 0..5 {
            thread::sleep(Duration::from_secs(1));
            if !has_any_windsurf_process() {
                eprintln!("[close_windsurf] macOS: 所有进程已退出 ({}秒)", i + 1);
                return Ok(());
            }
            eprintln!("[close_windsurf] macOS: 仍有 Helper 运行中... ({}/5)", i + 1);
        }

        // Step 4: Helper 超时未退出 → SIGTERM 清理（Helper 不触发崩溃对话框）
        if has_any_windsurf_process() {
            cleanup_helper_processes();
            thread::sleep(Duration::from_secs(2));
        }

        // Step 5: 最后手段 SIGKILL（仅对 Helper，不影响崩溃恢复）
        if has_any_windsurf_process() {
            eprintln!("[close_windsurf] macOS: Helper 仍存活，SIGKILL...");
            let _ = Command::new("sh").args(["-c", "pkill -9 -f 'Windsurf.app/Contents' 2>/dev/null || true"]).output();
            thread::sleep(Duration::from_secs(1));
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 先尝试优雅关闭（最多等 2 秒）
        let _ = Command::new("taskkill").args(["/IM", "Windsurf.exe"]).output();
        for _ in 0..2 {
            thread::sleep(Duration::from_secs(1));
            if !is_running() { break; }
        }
        // 还在运行则强杀整个进程树（/F 强制 + /T 子进程）
        if is_running() {
            let _ = Command::new("taskkill").args(["/F", "/T", "/IM", "Windsurf.exe"]).output();
            thread::sleep(Duration::from_millis(500));
        }
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("pkill").args(["-x", "-15", "windsurf"]).output();
        for _ in 0..5 {
            thread::sleep(Duration::from_secs(1));
            if !is_running() { break; }
        }
        let _ = Command::new("pkill").args(["-x", "-9", "windsurf"]).output();
        thread::sleep(Duration::from_millis(1500));
    }

    if is_running() {
        Err("无法关闭 Windsurf 主进程，请手动关闭后重试".to_string())
    } else {
        eprintln!("[close_windsurf] 关闭完成");
        Ok(())
    }
}

pub fn launch_windsurf() -> Result<(), String> {
    eprintln!("[launch_windsurf] 开始启动 Windsurf...");

    // 启动前确保没有残留进程（否则 macOS open 不会启动新实例）
    #[cfg(target_os = "macos")]
    if has_any_windsurf_process() {
        eprintln!("[launch_windsurf] 检测到残留进程，清理...");
        cleanup_helper_processes();
        thread::sleep(Duration::from_secs(2));
        if has_any_windsurf_process() {
            let _ = Command::new("sh").args(["-c", "pkill -9 -f 'Windsurf.app/Contents' 2>/dev/null || true"]).output();
            thread::sleep(Duration::from_secs(1));
        }
    }

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().unwrap_or_default();
        let possible_paths = vec![
            "/Applications/Windsurf.app".to_string(),
            format!("{}/Applications/Windsurf.app", home.display()),
        ];
        let app_path = possible_paths.iter().find(|p| std::path::Path::new(p).exists());
        match app_path {
            Some(path) => {
                eprintln!("[launch_windsurf] macOS: open {}", path);
                let output = Command::new("sh")
                    .args(["-c", &format!("open \"{}\"", path)])
                    .output()
                    .map_err(|e| format!("启动失败: {}", e))?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("open 失败: {}", stderr));
                }
            }
            None => return Err(format!("未找到 Windsurf.app，已搜索: {}", possible_paths.join(", "))),
        }
    }

    #[cfg(target_os = "windows")]
    {
        let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let program_files = std::env::var("PROGRAMFILES").unwrap_or_default();
        let mut paths = vec![
            format!(r"{}\Programs\Windsurf\Windsurf.exe", local_appdata),
            format!(r"{}\Windsurf\Windsurf.exe", program_files),
        ];
        for drive in &["C", "D", "E", "F"] {
            paths.push(format!(r"{}:\Program Files\Windsurf\Windsurf.exe", drive));
        }
        match paths.iter().find(|p| std::path::Path::new(p).exists()) {
            Some(path) => { Command::new(path).spawn().map_err(|e| format!("启动失败: {}", e))?; }
            None => return Err("未找到 Windsurf.exe".to_string()),
        }
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("windsurf").spawn().map_err(|e| format!("启动失败: {}", e))?;
    }

    let max_wait = if cfg!(target_os = "macos") { 5 } else { 10 };
    for i in 0..max_wait {
        thread::sleep(Duration::from_secs(1));
        if is_running() {
            eprintln!("[launch_windsurf] 启动成功 ({}秒)", i + 1);
            return Ok(());
        }
        eprintln!("[launch_windsurf] 等待启动... ({}/{})", i + 1, max_wait);
    }

    eprintln!("[launch_windsurf] 验证超时，open 已执行");
    Ok(())
}
