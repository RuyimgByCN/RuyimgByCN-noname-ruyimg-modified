# 旧线无头 harness（G0/E01 · D5）

上游 `archive/upstream/*` **只读**。运行副本放在仓库外的
`scratch/legacy-harness/{liuli,noname}`，可在副本上装依赖或打启动补丁。

## 能力

| 模块 | 作用 |
|---|---|
| `lib/env.mjs` | jsdom 页面、本地资源加载、断网语义、浏览器桩 |
| `lib/rng.mjs` | 可复现 `Math.random`（mulberry32） |
| `lib/vtime.mjs` | 虚拟时钟（加速 `game.delay`） |
| `lib/stubs.mjs` | Cordova/confirm 策略 |
| `lib/boot.mjs` | 琉璃版一键启动与 `waitLib` |
| `probe-liuli-boot.mjs` | 启动链路诊断 |
| `sample-lx-battle.mjs` | 龙血玄黄 / Boss 诸葛亮 / ylyg* 轨迹采样 |
| `generate-g0-docs.py` | 生成 `docs/migration/g0/` 37 份清单 |

## 用法

```bash
cd scripts/legacy-harness
npm ci   # 或 npm install；node_modules 已 gitignore

# 诊断启动（默认 scratch/legacy-harness/liuli）
node probe-liuli-boot.mjs /path/to/scratch/legacy-harness/liuli 20

# 龙血/Boss/ylyg 采样 → docs/operations/evidence/2026-10-16-t9-lx-battle-trace/
node sample-lx-battle.mjs /path/to/scratch/legacy-harness/liuli

# 重生成 G0 文档
python3 generate-g0-docs.py
```

## 副本补丁（仅 scratch）

琉璃 `game.js` 在无头环境下需：

1. `window.game` 赋值处同步暴露 `lib/ui/get/ai/_status`；
2. `localStorage.legacy_harness=1` 时拉长 `resetGameTimeout`，避免 5s 看门狗清局。

补丁只落在 `scratch/`，不写入 archive。
