# Noname-ruyimg-Modified 开发笔记

基于官方新版 noname（`upstream-noname`）的联机修改版。M1 基础搭建完成于 2026-06-14。

## 环境要求

- Node v26+、pnpm 11+（装在 `~/.local/bin`，PATH 已写入 `~/.zshrc`）
- 仓库是 partial clone（`--filter=blob:none`），working tree 完整、git 历史完整

## 启动命令

```bash
# 联机服务器（packages/server，端口 8082）
cd packages/server && pnpm dev

# 前端 dev（Vite，http://localhost:8080）
pnpm dev
```

## 端口

| 端口 | 用途 | 来源 |
|---|---|---|
| 8082 | 联机服务器（WebSocket 信令） | `packages/server/src/index.ts:307` |
| 8080 | 前端 dev server（Vite） | `scripts/dev.ts` |
| 8089 | 前端入口（302 → /index.html） | `scripts/dev.ts` |

## Git remote

- `upstream-noname` → `libnoname/noname`（官方新版，基础代码源，blob:none partial clone）
- `upstream` → `Helasisy1984/Noname-Liuli-Modified`（琉璃版，武将内容源）
- `upstream-freekill` → `Qsgs-Fans/FreeKill`（联机架构参考，只读）

## 分支

- `ruyimg`：ruyimg 修改版开发分支（基于官方 `main`）。所有改动在此分支提交，与官方历史隔离，方便 `git fetch upstream-noname` 合并更新。

## 已知问题（待后续 milestone 处理）

- **基础牌音效缺失**（`tao`/`sha`/`shan` 等）：noname 核心仓库仅含部分音频（9136 个），基础牌音效需懒人包。M4 琉璃素材迁移时从琉璃 `audio/` 一并补齐。

## 下一步（M2）

账号系统：SQLite + 注册/登录/token，替换 `packages/server` 现有的 `onlineKey` 机制。详见 `docs/superpowers/specs/2026-06-14-online-multiplayer-design.md` §4.1。
