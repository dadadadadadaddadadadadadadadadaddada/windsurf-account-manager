# Windsurf Windows 诊断报告

> **诊断时间**: 2026-04-05 22:18 (UTC+8)  
> **操作系统**: Windows  
> **用户名**: 赵勤杰  

---

## 一、环境变量与数据目录

| 环境变量 | 值 |
|---|---|
| `APPDATA` | `C:\Users\赵勤杰\AppData\Roaming` |
| `LOCALAPPDATA` | `C:\Users\赵勤杰\AppData\Local` |

### Windsurf 数据目录存在性

| 路径 | 状态 |
|---|---|
| `C:\Users\赵勤杰\AppData\Roaming\Windsurf` | **存在** ✅ |
| `C:\Users\赵勤杰\AppData\Local\Windsurf` | 不存在 |
| `C:\Users\赵勤杰\AppData\Roaming\Windsurf Next` | 不存在 |
| `C:\Users\赵勤杰\AppData\Local\Windsurf Next` | 不存在 |

**结论**: Windsurf 的数据目录位于 `%APPDATA%\Windsurf`（即 `Roaming` 目录下），不在 `Local` 目录。

---

## 二、Local State 文件（加密密钥来源）

### 文件位置

| 路径 | 状态 |
|---|---|
| `C:\Users\赵勤杰\AppData\Roaming\Windsurf\Local State` | **找到** ✅ |
| 其余 3 个候选路径 | 均不存在 |

### 文件详情

- **文件大小**: 434 bytes
- **顶层 JSON keys**: `['os_crypt']`（仅有一个顶层 key）
- **`os_crypt` 子 keys**: `['audit_enabled', 'encrypted_key']`

### `os_crypt.encrypted_key` 分析

| 属性 | 值 |
|---|---|
| Base64 字符串长度 | 380 字符 |
| Base64 前 40 字符 | `RFBBUEkBAAAA0Iyd3wEV0RGMegDAT8KX6wEAAAAP` |
| Base64 解码后 bytes 长度 | 283 bytes |
| 前 5 bytes (ASCII) | `DPAPI` |
| 前 5 bytes (HEX) | `4450415049` |

**结论**: `encrypted_key` 使用的是 **Windows DPAPI** 加密方案。解密流程如下：

1. Base64 解码 `encrypted_key`
2. 去掉前 5 字节 `DPAPI` 前缀
3. 使用 `win32crypt.CryptUnprotectData()` 解密剩余部分，得到 AES-256 密钥（32 字节）

---

## 三、state.vscdb 数据库文件

### 文件位置

| 路径 | 状态 |
|---|---|
| `C:\Users\赵勤杰\AppData\Roaming\Windsurf\User\globalStorage\state.vscdb` | **找到** ✅ |
| 其余 3 个候选路径 | 均不存在 |

### 文件详情

| 属性 | 值 |
|---|---|
| 文件大小 | 819,200 bytes (800 KB) |
| 最后修改时间 | 2026-04-05 22:18:25 |
| WAL 文件 | 不存在 |
| SHM 文件 | 不存在 |

### 数据库结构

- **表**: 仅有 `ItemTable` 一张表（SQLite 对表名大小写不敏感，`ItemTable` 与 `itemTable` 指向同一张表）
- **总行数**: 86 行

---

## 四、state.vscdb 关键数据详情

### 4.1 与 session/secret/auth/codeium 相关的 key 列表

| Key | 值类型 | 值摘要 |
|---|---|---|
| `chat.ChatSessionStore.index` | 明文 JSON | `{"version":1,"entries":{}}` |
| `codeium.windsurf` | 明文 JSON | 包含 `apiServerUrl`, `apiKey` 等配置 |
| `codeium.windsurf-windsurf_auth` | 明文字符串 | `Melissa Martin`（用户显示名） |
| `codeium.windsurf-windsurf_auth-` | 明文字符串 | `d9a22514-df0b-4ae7-9493-063979ce425f`（用户 ID） |
| `secret://...windsurf_auth.apiServerUrl` | 加密 Buffer JSON | v10 加密，密文 69 bytes |
| `secret://...windsurf_auth.sessions` | 加密 Buffer JSON | v10 加密，密文 269 bytes |
| `telemetry.currentSessionDate` | 明文字符串 | `Sun, 05 Apr 2026 14:15:49 GMT` |
| `telemetry.firstSessionDate` | 明文字符串 | `Mon, 08 Sep 2025 11:24:12 GMT` |
| `telemetry.lastSessionDate` | 明文字符串 | `Sun, 05 Apr 2026 14:14:22 GMT` |
| `windsurfAuthStatus` | 明文 JSON | 包含 `apiKey`, `allowedCommandModelConfigsProtoBinaryBase64`(8项), `userStatusProtoBinaryBase64` |
| `windsurf_auth-Melissa Martin` | 明文 JSON | `[]` |
| `windsurf_auth-Melissa Martin-usages` | 明文 JSON | 使用记录, lastUsed=1775398615547 |

### 4.2 加密 sessions 值详细分析

**Key**: `secret://{"extensionId":"codeium.windsurf","key":"windsurf_auth.sessions"}`

| 属性 | 值 |
|---|---|
| 值类型 | `str`（字符串） |
| 字符串长度 | 982 字符 |
| JSON 格式 | `{"type":"Buffer","data":[...]}` |
| data bytes 总长度 | 269 bytes |
| 前 3 bytes (ASCII) | `v10` |
| 前 16 bytes (HEX) | `7631304e8a9242b8b9e8b000cfdfb3c0` |
| Nonce (12 bytes, HEX) | `4e8a9242b8b9e8b000cfdfb3` |
| Ciphertext 长度 | 238 bytes |
| GCM Tag (16 bytes, HEX) | `1d71b75c2abbb610245b72bcf6cd2156` |

### 4.3 加密 apiServerUrl 值详细分析

**Key**: `secret://{"extensionId":"codeium.windsurf","key":"windsurf_auth.apiServerUrl"}`

| 属性 | 值 |
|---|---|
| 值类型 | `str`（字符串） |
| 字符串长度 | 279 字符 |
| JSON 格式 | `{"type":"Buffer","data":[...]}` |
| data bytes 总长度 | 69 bytes |
| 前 3 bytes (ASCII) | `v10` |
| 前 16 bytes (HEX) | `76313023f09d7682d077b9cff2917073` |
| Nonce (12 bytes, HEX) | `23f09d7682d077b9cff29170` |
| Ciphertext 长度 | 38 bytes |
| GCM Tag (16 bytes, HEX) | `95cf78902a628d8543400c991ccf87f6` |

### 4.4 明文存储的认证信息

| Key | 值 |
|---|---|
| 用户显示名 (`codeium.windsurf-windsurf_auth`) | `Melissa Martin` |
| 用户 ID (`codeium.windsurf-windsurf_auth-`) | `d9a22514-df0b-4ae7-9493-063979ce425f` |
| API Key (在 `windsurfAuthStatus` 中) | `sk-ws-01-vms5KyBGa-UMO9bYirdDSJzKflYmtHiPByPOscLSIngHZ6eLKW53_qHD4qokbIuzIKtWv0AYIp4Rvs2nRkAeC6nkZhMknw` |
| `windsurfAuthStatus` 其他字段 | `allowedCommandModelConfigsProtoBinaryBase64` (list, 8 项), `userStatusProtoBinaryBase64` (string, 77548 字符) |

---

## 五、Windsurf 安装信息

| 属性 | 值 |
|---|---|
| 安装路径 | `C:\Users\赵勤杰\AppData\Local\Programs\Windsurf\Windsurf.exe` |
| 文件版本 | `1.9577.43` |
| 产品名 | `Windsurf` |

---

## 六、加密体系总结

Windsurf 在 Windows 上使用的加密体系与 **Chromium 浏览器完全一致**：

### 解密流程图

```
Local State (JSON)
  └─ os_crypt.encrypted_key (Base64)
       │
       ▼
  Base64 解码 → 去掉前 5 字节 "DPAPI" 前缀
       │
       ▼
  Windows DPAPI (CryptUnprotectData) 解密
       │
       ▼
  得到 AES-256 密钥 (32 bytes)
       │
       ▼
state.vscdb → ItemTable
  └─ secret://... key 的 value
       │
       ▼
  JSON 解析 {"type":"Buffer","data":[...]}
       │
       ▼
  bytes = data 数组转 bytes
       │
       ▼
  解析 v10 结构:
    ├─ [0:3]   = "v10" (固定前缀, 跳过)
    ├─ [3:15]  = Nonce (12 bytes)
    ├─ [15:-16] = Ciphertext
    └─ [-16:]  = GCM Authentication Tag (16 bytes)
       │
       ▼
  AES-256-GCM 解密 (key=AES密钥, nonce=12bytes, data=ciphertext+tag)
       │
       ▼
  得到明文 (sessions JSON / apiServerUrl 字符串)
```

### 关键路径汇总

| 用途 | 完整路径 |
|---|---|
| DPAPI 加密的 AES 密钥 | `C:\Users\赵勤杰\AppData\Roaming\Windsurf\Local State` → `os_crypt.encrypted_key` |
| 加密的 sessions 数据 | `C:\Users\赵勤杰\AppData\Roaming\Windsurf\User\globalStorage\state.vscdb` → `ItemTable` → key=`secret://{"extensionId":"codeium.windsurf","key":"windsurf_auth.sessions"}` |
| 加密的 API Server URL | 同上数据库 → key=`secret://{"extensionId":"codeium.windsurf","key":"windsurf_auth.apiServerUrl"}` |

### 依赖库

解密需要以下 Python 库：

- `sqlite3` (标准库)
- `json` (标准库)
- `base64` (标准库)
- `win32crypt` (来自 `pywin32`，用于 DPAPI 解密)
- `cryptography` (用于 AES-256-GCM 解密)
