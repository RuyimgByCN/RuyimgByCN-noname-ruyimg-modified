# noname 联机服务器部署

联机服务器是一个轻量 WebSocket 服务（仅依赖 `ws` + `bcryptjs` + Node 内置 `node:sqlite`），适合部署到任意能跑 Node 22+ 的环境。

## 环境要求

- **Node.js ≥ 22.5**（用到内置 `node:sqlite`）
- 服务器内存：1C1G 起步（支持数十人同时在线）

## 构建

```bash
cd packages/server
pnpm install
pnpm build          # → dist/index.js (ESM) + dist/index.cjs (CJS)
```

## 环境变量

| 变量 | 默认 | 说明 |
|---|---|---|
| `PORT` | `8082` | 监听端口 |
| `HOST` | `0.0.0.0` | 监听地址（对外开放用 0.0.0.0） |
| `TLS_CERT` | (空) | TLS 证书路径（提供即启用 `wss://`） |
| `TLS_KEY` | (空) | TLS 私钥路径（与 CERT 同时提供） |
| `ALLOW_ORIGIN` | (空) | 允许的页面来源，逗号分隔（如 `https://game.example.com`）。不设则不校验 |

## 启动

```bash
# 基础启动（裸 ws://，建议前置反向代理做 TLS）
node dist/index.js

# 自定义端口
PORT=3000 node dist/index.js

# 服务器直接做 TLS（wss://）
TLS_CERT=/etc/ssl/noname.crt TLS_KEY=/etc/ssl/noname.key node dist/index.js

# 限制来源（防跨域滥用）
ALLOW_ORIGIN=https://game.example.com node dist/index.js
```

## 部署方式

### 方式 A：反向代理（推荐）

Node 跑裸 `ws://localhost:8082`，前面放 Caddy 自动处理 TLS。**最省心**。

**Caddy 配置**（`/etc/caddy/Caddyfile`）：
```
noname.yourdomain.com {
    reverse_proxy localhost:8082
}
```
- Caddy 自动申请 Let's Encrypt 证书并续期
- 客户端连接：`wss://noname.yourdomain.com`
- 启动：`node dist/index.js`（不需要配 TLS_CERT，TLS 由 Caddy 终结）

**nginx 配置**（需手动用 certbot 申请证书）：
```nginx
server {
    listen 443 ssl;
    server_name noname.yourdomain.com;
    ssl_certificate /etc/letsencrypt/live/noname.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/noname.yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://localhost:8082;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

### 方式 B：服务器直接 TLS

不依赖反向代理，Node 自己加载证书：
```bash
TLS_CERT=/path/cert.pem TLS_KEY=/path/key.pem node dist/index.js
```
适合无 nginx/Caddy 的环境，但要自己管理证书续期。

### 方式 C：托管平台（Railway / Fly.io / Render）

这些平台自动注入 `PORT` 环境变量并处理 TLS：
1. 推送仓库，设置 build 命令 `pnpm --filter @noname/server build`、start 命令 `node packages/server/dist/index.js`
2. 平台分配的域名直接用 `wss://` 连接
3. 数据持久化：注意 `data/noname.db` 是本地文件，平台重启会丢——如需持久化，用平台的 volume 挂载 `packages/server/data/` 目录

## Docker / Docker Compose 部署

### 快速启动（Compose）

```bash
cd packages/server
docker compose up -d
```

SQLite 数据卷 `noname-data` 自动持久化（账号/战绩/好友），容器重启不丢数据。

### 手动 Docker

```bash
cd packages/server
docker build -t noname-server .
docker run -d \
  --name noname-server \
  -p 8082:8082 \
  -v noname-data:/app/data \
  -e ALLOW_ORIGIN=https://game.example.com \
  noname-server
```

### 配合 Caddy 的 Compose（推荐公网）

在宿主机上用 Caddy 做反向代理 + 自动 TLS，Compose 只暴露内网端口：

```yaml
# docker-compose.yml 改 ports 为只监听本地
ports:
  - "127.0.0.1:8082:8082"
```

Caddyfile：
```
game.yourdomain.com {
    reverse_proxy localhost:8082
}
```

## 客户端连接

游戏前端 → 联机模式 → 输入服务器地址：
- 本地：`localhost:8082`
- 公网（反向代理）：`noname.yourdomain.com`（443 端口，浏览器自动 wss://）
- 公网（直连 TLS）：`noname.yourdomain.com:8082`

## 数据

- `packages/server/data/noname.db`：SQLite 数据库（账号表 + 房间是内存的，重启清空）
- 房间状态在内存（`rooms` Map），服务器重启房间清空——这是预期行为

## 进程管理（生产）

用 PM2 或 systemd 保持常驻：

**PM2**：
```bash
pm2 start dist/index.js --name noname-server
pm2 save
pm2 startup    # 开机自启
```

**systemd**（`/etc/systemd/system/noname.service`）：
```ini
[Unit]
Description=noname server
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/noname/packages/server
ExecStart=/usr/bin/node dist/index.js
Environment=PORT=8082
Restart=always
User=noname

[Install]
WantedBy=multi-user.target
```

## 排错

| 症状 | 原因 |
|---|---|
| 浏览器连不上 `ws://` 公网地址 | HTTPS 页面禁止明文 WebSocket（mixed content），必须用 `wss://`（反代或直配 TLS） |
| `Cannot find package 'sqlite'` | Node < 22.5，或 build 产物 `node:sqlite` 前缀丢失（已用 tsup onSuccess 修正） |
| 连接被拒（`verifyClient` 返回 false） | `ALLOW_ORIGIN` 配置了但客户端来源不在白名单 |
