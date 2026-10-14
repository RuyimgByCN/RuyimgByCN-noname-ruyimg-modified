# T13 worker 切换第二片：client env 分流 + observer 适配首层（2026-10-14）

## 交付

- **client.ts env 分流**：`VITE_ENGINE_BACKEND=native` 时选
  `engine.native.worker.ts`（native 模式 bundle 忽略——标准目录内建），
  否则旧线 `engine.worker.ts`（默认不变，零破坏）。
- **observer schema 适配首层**：native view_json（驼峰 rulesetVersion）→
  ObserverSnapshot 核心字段映射（revision/round/turn/phase 大小写规约/
  seats 核心面）；深字段（identity/skills/known_cards/character_def）置空
  ——**登记差异**，后续批补全。
- 适配逻辑与 nativeBackend 同源（phaseMap/字段映射两处一致，冒烟直测
  nativeBackend.nativeObserverToSnapshot）。

## 冒烟（8/8，[worker-smoke.json](worker-smoke.json)）

新增 observer_adapter_core：适配后 4 座位/存活≥2/round≥0/revision=279
（rulesetVersion 驼峰字段勘正——首版读 snake_case 得 undefined）/
phase 大小写规约。

## 登记边界

- typecheck（vue-tsc）通过；**真实浏览器回归**仍需后续批（worker 生命周期/
  UI 组件联动）。
- 深字段适配、BattleView 等组件消费、扩展包装载面归后续批。
- engine-native 无行为改动：**738/0/0**。
