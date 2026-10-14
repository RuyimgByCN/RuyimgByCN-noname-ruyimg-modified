# T13 客户端 worker 切换首片：native 后端适配层（2026-10-14）

## 任务来源

[T13](../../architecture/native-progress.md)「web-client worker 切换」首片：
web-client 新增**原生引擎后端**（engine.native.worker.ts + nativeBackend.ts
适配层），引擎为批 11 的 engine-native-wasm 窄壳。**旧线 worker 未动**——
客户端按环境分流选 worker 文件（UI 层完整切换归后续批）。

## 交付

- **`nativeBackend.ts`**：WorkerRequest 协议子集 → NativeEngine 窄 API 映射
  （init/start/endPhase→next_phase/playCard→play_card/resolveChoice→answer
  [选项值→索引匹配，不匹配报错不静默]/autoStep→auto_step/observer→view_json）；
  未映射请求（activateSkill/useSkillCard/loadReplay/seek）显式报错。
- **`engine.native.worker.ts`**：独立 Worker 入口（协议兼容子集面；标准目录
  内建，扩展包装载面归后续批）。
- **`scripts/build-native.mjs`**：wasm-pack 构建 + recursive 部署到
  `src/worker/native-pkg/`（gitignore，不入库）；`scripts/native-worker-smoke.mjs`
  Node 直跑冒烟（TS strip-types，无浏览器依赖）。

## 冒烟（7/7，[worker-smoke.json](worker-smoke.json)）

create/observer（信息隔离投影 4 座位）/endPhase（status 形状）/autoStep 驱动
（40 步内推进）/playCard（status 形状）/resolveChoice 负例（choice 不匹配报
错）/未映射请求面（类型即拒绝）。

## 登记边界

- **observer schema 与旧线不同**（PlayerView 投影 vs 旧线 observerJson）——
  UI 层（BattleView 等消费旧 schema）适配归后续批；客户端 env 分流同批。
- 标准目录内建；扩展包装载面（C0 装载器 wasm 侧）归后续批。
- activateSkill/useSkillCard 未映射（窄壳窄 API 设计）——随主动技 UI 批。

## 实测记录

| 项 | 结果 |
|---|---|
| worker 冒烟 | **7/7** |
| engine-native 全量 | **738/0/0**（本批纯前端，无 Rust 行为改动） |
| wasm-pack 构建 | build-native.mjs 端到端 ✓ |
