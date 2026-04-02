# Windsurf Account Manager

多账号管理与一键切换工具，支持 Windsurf 编辑器的账号快速切换、配额监控。

## 功能特性

- **多账号管理** — 添加、导入、导出、批量删除账号
- **一键切换** — 自动关闭 Windsurf → 重置机器 ID → 写入认证数据 → 重启编辑器
- **配额监控** — 实时查看日/周剩余额度百分比及刷新时间
- **批量操作** — 批量获取 Token、批量刷新配额
- **跨平台** — 支持 macOS / Windows / Linux

## 截图

| 功能 | 说明 |
|------|------|
| 账号列表 | 展示邮箱、类型、日/周额度进度条 |
| 一键切换 | 进度弹窗实时显示切换步骤 |
| 配额刷新 | 批量查询所有账号剩余额度 |

## 技术栈

- **前端**: Vue 3 + TailwindCSS
- **后端**: Rust + Tauri 2
- **数据库**: SQLite (本地存储)
- **认证**: Firebase Authentication
- **构建**: GitHub Actions 多平台自动打包

## 安装

前往 [Releases](../../releases) 页面下载对应平台安装包：

| 平台 | 文件 |
|------|------|
| macOS (Apple Silicon) | `*.aarch64.dmg` |
| macOS (Intel) | `*.x64.dmg` |
| Windows | `*.msi` / `*.exe` |
| Linux | `*.deb` / `*.AppImage` |

## 安装注意事项

### macOS

由于应用未经 Apple 签名，首次打开会提示「无法验证开发者」。解决方法：

**方法一（推荐）：**
1. 双击 `.dmg` 安装后，先尝试打开应用（会弹出警告）
2. 打开 **系统设置 → 隐私与安全性**
3. 滚动到底部，找到「已阻止使用 Windsurf Account Manager」提示
4. 点击 **仍要打开** → 输入密码确认

**方法二（终端命令）：**
```bash
sudo xattr -rd com.apple.quarantine /Applications/Windsurf\ Account\ Manager.app
```

### Windows

首次运行 `.exe` 或 `.msi` 可能弹出 **SmartScreen** 警告「Windows 已保护你的电脑」：
1. 点击 **更多信息**
2. 点击 **仍要运行**

### Linux

**`.AppImage` 文件需要添加执行权限：**
```bash
chmod +x Windsurf.Account.Manager_1.0.0_amd64.AppImage
./Windsurf.Account.Manager_1.0.0_amd64.AppImage
```

**`.deb` 安装：**
```bash
sudo dpkg -i Windsurf.Account.Manager_1.0.0_amd64.deb
```

**`.rpm` 安装：**
```bash
sudo rpm -i Windsurf.Account.Manager-1.0.0-1.x86_64.rpm
```

## 本地开发

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build
```

**环境要求**: Node.js 20+, Rust 1.70+

## 数据安全

- 所有账号数据仅存储在本地 SQLite 数据库
- 无任何数据上传至远程服务器
- 网络请求仅用于 Firebase 认证和 Windsurf 官方 API

## License

MIT
