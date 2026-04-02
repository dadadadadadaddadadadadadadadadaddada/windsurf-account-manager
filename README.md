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
