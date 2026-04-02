# Windsurf GetPlanStatus 接口文档

## 基本信息

| 项目 | 值 |
|------|-----|
| **接口名称** | GetPlanStatus |
| **完整URL** | `https://windsurf.com/_backend/exa.seat_management_pb.SeatManagementService/GetPlanStatus` |
| **请求方法** | POST |
| **协议** | HTTP/2 (Connect Protocol v1) |
| **Content-Type** | `application/proto` (Protobuf 二进制) |
| **响应格式** | `application/proto` (Protobuf 二进制) |

---

## 请求头 (Headers)

```http
:authority: windsurf.com
:method: POST
:path: /_backend/exa.seat_management_pb.SeatManagementService/GetPlanStatus
:scheme: https
accept: */*
accept-encoding: gzip, deflate, br, zstd
accept-language: zh-CN,zh;q=0.9
connect-protocol-version: 1
content-type: application/proto
origin: https://windsurf.com
referer: https://windsurf.com/profile
user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36
x-auth-token: <Firebase_JWT_Token>
```

### 关键请求头说明

| Header | 说明 |
|--------|------|
| `connect-protocol-version: 1` | 使用 Connect Protocol v1（基于 gRPC-Web 的变体） |
| `content-type: application/proto` | 请求体为 Protobuf 二进制格式 |
| `x-auth-token` | **必须**，Firebase JWT 认证令牌 |

---

## 认证方式

### 1. 获取 JWT Token

通过 Firebase Identity Toolkit 登录获取：

```
POST https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key=AIzaSyDsOl-1XpT5err0Tcnx8FFod1H8gVGIycY
Content-Type: application/json

{
  "email": "<你的邮箱>",
  "password": "<你的密码>",
  "returnSecureToken": true
}
```

登录成功后返回的 `idToken` 字段即为 JWT Token。

### 2. JWT Token 结构（解码后 Payload）

```json
{
  "name": "Joseph Moore",
  "iss": "https://securetoken.google.com/exa2-fb170",
  "aud": "exa2-fb170",
  "auth_time": 1775092908,
  "user_id": "AOy0LHI9eVV9iQ0xFkLUNWGywGb2",
  "sub": "AOy0LHI9eVV9iQ0xFkLUNWGywGb2",
  "iat": 1775092908,
  "exp": 1775096508,
  "email": "zft001-nghjr65754@yahoo.com",
  "email_verified": true
}
```

> **注意**: JWT Token 有效期约 1 小时（`exp - iat = 3600s`），过期后需重新登录获取。

---

## 请求体 (Request Body)

请求体为 Protobuf 二进制编码，包含两个字段：

| Protobuf Field | 类型 | 值 | 说明 |
|----------------|------|-----|------|
| field 1 | string | `<JWT Token>` | 与 x-auth-token 相同的 Firebase JWT |
| field 2 | varint | `1` | 固定值，可能表示请求类型 |

### 用 Python 构造请求体示例

```python
import struct

def encode_varint(value):
    result = b''
    while value > 127:
        result += bytes([value & 0x7F | 0x80])
        value >>= 7
    result += bytes([value & 0x7F])
    return result

def build_request_body(jwt_token):
    token_bytes = jwt_token.encode('utf-8')
    # field 1, wire type 2 (length-delimited) = tag 0x0A
    body = b'\x0a' + encode_varint(len(token_bytes)) + token_bytes
    # field 2, wire type 0 (varint) = tag 0x10, value 1
    body += b'\x10\x01'
    return body
```

---

## 响应体解码

响应为 Protobuf 二进制（可能经过 gzip 压缩），解码后结构如下：

### 核心字段映射

```
1 {                              // PlanStatus 主体
  1 {                            // PlanConfig 套餐配置
    1: 9                         // plan_id
    2: "Trial"                   // plan_name (套餐名称)
    3: 1                         // ?
    4: 1                         // ?
    7: 16384                     // ?
    8: 600                       // ?
    12: 10000                    // daily_quota_total (日配额总量)
    13: 20000                    // weekly_quota_total (周配额总量)
    21 { ... }                   // 各模型的 cost multiplier 配置
  }
  2 {
    1: <timestamp>               // billing_cycle_start (账单周期开始时间戳)
  }
  3 {
    1: <timestamp>               // billing_cycle_end (账单周期结束时间戳)
  }
  8: 10000                       // daily_quota_total (重复)
  9: 20000                       // weekly_quota_total (重复)
  10 {
    1: 4                         // ?
  }
  12: 1                          // ?
  14: 100                        // ★ daily_quota_remaining_percent (日配额剩余百分比)
  15: 50                         // ★ weekly_quota_remaining_percent (周配额剩余百分比)
  17: 1775116800                 // daily_reset_timestamp (日配额重置时间)
  18: 1775376000                 // weekly_reset_timestamp (周配额重置时间)
}
```

### ★ 目标字段

| 字段路径 | 含义 | 示例值 | 对应页面显示 |
|----------|------|--------|-------------|
| `1.14` | 日配额剩余百分比 | `100` | "Your daily quota: **100.00%** remaining" |
| `1.15` | 周配额剩余百分比 | `50` | "Your weekly quota: **50.00%** remaining" |
| `1.17` | 日配额重置时间(Unix) | `1775116800` | "Resets 4月2日 GMT+8 16:00" |
| `1.18` | 周配额重置时间(Unix) | `1775376000` | "Resets 4月5日 GMT+8 16:00" |

---

## 完整调用示例 (Python)

```python
import requests
import struct

def encode_varint(value):
    result = b''
    while value > 127:
        result += bytes([value & 0x7F | 0x80])
        value >>= 7
    result += bytes([value & 0x7F])
    return result

def get_plan_status(jwt_token):
    url = "https://windsurf.com/_backend/exa.seat_management_pb.SeatManagementService/GetPlanStatus"

    # 构造 Protobuf 请求体
    token_bytes = jwt_token.encode('utf-8')
    body = b'\x0a' + encode_varint(len(token_bytes)) + token_bytes + b'\x10\x01'

    headers = {
        "Content-Type": "application/proto",
        "Connect-Protocol-Version": "1",
        "x-auth-token": jwt_token,
        "Origin": "https://windsurf.com",
        "Referer": "https://windsurf.com/profile",
        "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36"
    }

    resp = requests.post(url, data=body, headers=headers)
    return resp.content  # Protobuf 二进制，需用 protoc --decode_raw 解码

def login_and_get_token(email, password):
    url = "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword"
    params = {"key": "AIzaSyDsOl-1XpT5err0Tcnx8FFod1H8gVGIycY"}
    payload = {
        "email": email,
        "password": password,
        "returnSecureToken": True
    }
    resp = requests.post(url, params=params, json=payload)
    return resp.json()["idToken"]

# 使用方式
# token = login_and_get_token("your@email.com", "your_password")
# raw_response = get_plan_status(token)
# 然后用 protoc --decode_raw 或 Python protobuf 库解析 raw_response
```

### 用 curl 调用

```bash
# 1. 先登录获取 token
TOKEN=$(curl -s 'https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key=AIzaSyDsOl-1XpT5err0Tcnx8FFod1H8gVGIycY' \
  -H 'Content-Type: application/json' \
  -d '{"email":"your@email.com","password":"your_password","returnSecureToken":true}' \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['idToken'])")

# 2. 用 Python 生成请求体并调用接口
python3 -c "
import requests, struct, subprocess

token = '${TOKEN}'

def encode_varint(value):
    result = b''
    while value > 127:
        result += bytes([value & 0x7F | 0x80])
        value >>= 7
    result += bytes([value & 0x7F])
    return result

token_bytes = token.encode('utf-8')
body = b'\x0a' + encode_varint(len(token_bytes)) + token_bytes + b'\x10\x01'

resp = requests.post(
    'https://windsurf.com/_backend/exa.seat_management_pb.SeatManagementService/GetPlanStatus',
    data=body,
    headers={
        'Content-Type': 'application/proto',
        'Connect-Protocol-Version': '1',
        'x-auth-token': token,
        'Origin': 'https://windsurf.com',
    }
)

# 保存并用 protoc 解码
with open('/tmp/resp.bin', 'wb') as f:
    f.write(resp.content)

subprocess.run(['protoc', '--decode_raw'], input=resp.content)
"
```

---

## 响应解码工具

```bash
# 解码已保存的响应
protoc --decode_raw < response.bin
```

---

## 注意事项

1. **Token 有效期**: Firebase JWT 有效期约 1 小时，过期需重新登录
2. **请求体编码**: 请求体是 Protobuf 二进制，不是 JSON，不能用普通 HTTP 工具直接构造
3. **Connect Protocol**: 该接口使用 Connect Protocol v1，是 gRPC-Web 的一种实现
4. **响应解压**: 响应可能经过 gzip/br 压缩，HTTP 客户端库通常会自动处理
