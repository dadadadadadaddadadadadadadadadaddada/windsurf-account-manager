# Windsurf Windows 诊断检查文档

请在 Windows 电脑上逐步执行以下检查，将每一步的输出结果完整复制给我。

---

## 1. 检查环境变量和基础路径

在 PowerShell 中执行：

```powershell
Write-Host "=== 环境变量 ==="
Write-Host "APPDATA: $env:APPDATA"
Write-Host "LOCALAPPDATA: $env:LOCALAPPDATA"

Write-Host "`n=== 检查 Windsurf 数据目录是否存在 ==="
$paths = @(
    "$env:APPDATA\Windsurf",
    "$env:LOCALAPPDATA\Windsurf",
    "$env:APPDATA\Windsurf Next",
    "$env:LOCALAPPDATA\Windsurf Next"
)
foreach ($p in $paths) {
    if (Test-Path $p) {
        Write-Host "[存在] $p"
    } else {
        Write-Host "[不存在] $p"
    }
}
```

---

## 2. 检查 Local State 文件（加密密钥来源）

```powershell
Write-Host "=== 查找 Local State 文件 ==="
$localStateLocations = @(
    "$env:APPDATA\Windsurf\Local State",
    "$env:LOCALAPPDATA\Windsurf\Local State",
    "$env:APPDATA\Windsurf Next\Local State",
    "$env:LOCALAPPDATA\Windsurf Next\Local State"
)
foreach ($f in $localStateLocations) {
    if (Test-Path $f) {
        Write-Host "[找到] $f"
        Write-Host "文件大小: $((Get-Item $f).Length) bytes"
        $json = Get-Content $f -Raw | ConvertFrom-Json
        if ($json.os_crypt -and $json.os_crypt.encrypted_key) {
            $keyLen = $json.os_crypt.encrypted_key.Length
            $keyPrefix = $json.os_crypt.encrypted_key.Substring(0, [Math]::Min(20, $keyLen))
            Write-Host "os_crypt.encrypted_key 存在, 长度=$keyLen, 前20字符=$keyPrefix..."
        } else {
            Write-Host "os_crypt.encrypted_key 不存在!"
            Write-Host "Local State 顶层 key: $($json.PSObject.Properties.Name -join ', ')"
        }
    } else {
        Write-Host "[不存在] $f"
    }
}
```

---

## 3. 检查 state.vscdb 文件位置

```powershell
Write-Host "=== 查找 state.vscdb 文件 ==="
$vscdbLocations = @(
    "$env:APPDATA\Windsurf\User\globalStorage\state.vscdb",
    "$env:LOCALAPPDATA\Windsurf\User\globalStorage\state.vscdb",
    "$env:APPDATA\Windsurf Next\User\globalStorage\state.vscdb",
    "$env:LOCALAPPDATA\Windsurf Next\User\globalStorage\state.vscdb"
)
foreach ($f in $vscdbLocations) {
    if (Test-Path $f) {
        Write-Host "[找到] $f"
        Write-Host "文件大小: $((Get-Item $f).Length) bytes"
        Write-Host "最后修改: $((Get-Item $f).LastWriteTime)"
        # 检查是否有 WAL 和 SHM 文件
        $wal = "$f-wal"
        $shm = "$f-shm"
        Write-Host "WAL文件: $(if (Test-Path $wal) { '存在, ' + (Get-Item $wal).Length + ' bytes' } else { '不存在' })"
        Write-Host "SHM文件: $(if (Test-Path $shm) { '存在, ' + (Get-Item $shm).Length + ' bytes' } else { '不存在' })"
    } else {
        Write-Host "[不存在] $f"
    }
}
```

---

## 4. 读取 state.vscdb 中的关键数据（需要安装 sqlite3 或用 Python）

用 Python 执行（Windows 通常自带 Python）：

```powershell
python -c @"
import sqlite3, json, os, sys

appdata = os.environ.get('APPDATA', '')
localappdata = os.environ.get('LOCALAPPDATA', '')

candidates = [
    os.path.join(appdata, 'Windsurf', 'User', 'globalStorage', 'state.vscdb'),
    os.path.join(localappdata, 'Windsurf', 'User', 'globalStorage', 'state.vscdb'),
    os.path.join(appdata, 'Windsurf Next', 'User', 'globalStorage', 'state.vscdb'),
    os.path.join(localappdata, 'Windsurf Next', 'User', 'globalStorage', 'state.vscdb'),
]

db_path = None
for c in candidates:
    if os.path.exists(c):
        db_path = c
        break

if not db_path:
    print('ERROR: state.vscdb not found in any location!')
    sys.exit(1)

print(f'Using DB: {db_path}')
conn = sqlite3.connect(db_path)
cur = conn.cursor()

print('\n=== 所有表 ===')
cur.execute("SELECT name FROM sqlite_master WHERE type='table'")
for row in cur.fetchall():
    print(f'  {row[0]}')

print('\n=== 查找 sessions 相关的 key ===')
for table in ['ItemTable', 'itemTable', 'item_table']:
    try:
        cur.execute(f"SELECT key FROM [{table}] WHERE key LIKE '%session%' OR key LIKE '%secret%' OR key LIKE '%auth%' OR key LIKE '%codeium%'")
        rows = cur.fetchall()
        if rows:
            print(f'\n表 {table} 中匹配的 key:')
            for row in rows:
                print(f'  {row[0]}')
                # 读取值的前 200 个字符
                cur.execute(f"SELECT value FROM [{table}] WHERE key=?", (row[0],))
                val = cur.fetchone()
                if val and val[0]:
                    v = val[0] if isinstance(val[0], str) else repr(val[0])
                    print(f'    值(前200): {v[:200]}')
    except Exception as e:
        pass

print('\n=== 查找 encrypted sessions 的具体内容 ===')
secret_key = 'secret://{"extensionId":"codeium.windsurf","key":"windsurf_auth.sessions"}'
for table in ['ItemTable', 'itemTable', 'item_table']:
    try:
        cur.execute(f"SELECT value FROM [{table}] WHERE key=?", (secret_key,))
        row = cur.fetchone()
        if row:
            val = row[0]
            print(f'\n表 {table} - sessions 值类型: {type(val).__name__}')
            if isinstance(val, str):
                print(f'  长度: {len(val)}')
                # 尝试解析 JSON
                try:
                    obj = json.loads(val)
                    if isinstance(obj, dict) and 'type' in obj and 'data' in obj:
                        data = obj['data']
                        print(f'  格式: Buffer JSON, data 长度={len(data)}')
                        print(f'  前 20 bytes: {data[:20]}')
                        # 检查前缀
                        prefix = bytes(data[:3])
                        print(f'  前缀(ascii): {prefix}')
                    else:
                        print(f'  JSON keys: {list(obj.keys()) if isinstance(obj, dict) else "not dict"}')
                        print(f'  前200: {val[:200]}')
                except:
                    print(f'  非 JSON, 前200: {val[:200]}')
            elif isinstance(val, bytes):
                print(f'  bytes长度: {len(val)}')
                print(f'  前20 bytes: {list(val[:20])}')
            break
    except:
        pass

conn.close()
"@
```

---

## 5. 检查 Windsurf 安装位置

```powershell
Write-Host "=== Windsurf 安装位置 ==="
$installPaths = @(
    "$env:LOCALAPPDATA\Programs\Windsurf\Windsurf.exe",
    "$env:PROGRAMFILES\Windsurf\Windsurf.exe",
    "C:\Program Files\Windsurf\Windsurf.exe"
)
foreach ($p in $installPaths) {
    if (Test-Path $p) {
        $ver = (Get-Item $p).VersionInfo
        Write-Host "[找到] $p"
        Write-Host "  版本: $($ver.FileVersion)"
        Write-Host "  产品: $($ver.ProductName)"
    }
}
```

---


请把以上所有步骤的输出结果完整发给我，我根据实际路径和加密格式来修正代码。
