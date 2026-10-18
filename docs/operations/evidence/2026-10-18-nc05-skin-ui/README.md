# NC05 第三类纵向案例：皮肤/UI 组合 + 资源与规则指纹分离（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC05**（[方案 §NC05](../../../architecture/native-convergence-plan.md#nc05)）第三类真实案例
- 案例来源：`skin-switch-upstream-liuli`（L24 皮肤切换：纯配置/表现面 `assets/config.json`，
  无规则模块）+ `select-beautify-upstream-liuli`（L27 选将美化）+ `shahai-swd-upstream-noname`（规则包，共存对照）
- 规则版本 v321 / 存档 v105（无规则与格式变化）
- 结论：**第三类案例通过**（资源经安装副本解析、资源/规则指纹分离、升级与固定锁、组合共存）

## 1 验收回归（新增 `apps/game-server/tests/nc05_skin_ui.rs`，2 项）

| 验收项 | 用例 | 结果 |
|---|---|---|
| 皮肤包安装/装载（无 VM、无卡牌）；**配置资源经 store 解析到安装目录**（规范化后确认位于 `packages/upstream-liuli__skin-switch/...`，不读 drop-in 源目录）；资源索引可枚举；升级到 1.0.1 生效且资源指纹变化；旧锁仍装载旧版本；回退后可装载 | `nc05_skin_ui_asset_separation` | 通过 |
| **资源/规则指纹分离**（同版本内存对照）：仅替换资产条目 `source_hash` → `asset_hash` 变化而 **`logic_hash` 不变** | 同上 | 通过 |
| 皮肤 + 规则 + 选将美化**组合共存**：建局 → 正式选将 → 权威动作查询 → 真实提交 → 组合内容锁存档往返 | `nc05_skin_with_rules_pack_coexists` | 通过 |

## 2 过程中的两条 store 不变量（未绕过，作为正面证据）

1. **同版本不同内容被拒**：改写资源后按 1.0.0 重装 → `Invalid("version 1.0.0 already installed with different content")`——即「安装失败保留旧版本」；测试改用**新版本 1.0.1** 走真实升级路径。
2. **版本进入逻辑指纹**：`compute_logic_hash` 覆盖 `namespace/package_id/version/依赖/模块…`（见 `content-schema` 源码），故升级必然改变逻辑指纹；「资源变化不改逻辑指纹」改用**同版本内存对照**验收（`variant` vs `base`）。

## 3 验证结果

```
cargo test --offline -p game-server --test nc05_skin_ui
cargo test --offline -p game-server --no-fail-fast
python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| 第三类案例定向 | **2 passed / 0 failed** | [targeted.log](targeted.log) |
| game-server 全量 | **61 passed / 0 failed** | [game-server-tests.log](game-server-tests.log) |
| 行数门禁 | 8236 文件 ≤ 2000 行；单测 OK | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC05）

- [ ] 真实**客户端渲染**验收（皮肤/UI 的实际呈现、动画、音频；本批覆盖资源解析与指纹语义，未启动浏览器/桌面端）
- [ ] 皮肤资源的目标平台档位（桌面/移动/Web）与资源变体（`image_variants`/`audio_variants`/`platforms` 字段已存在，未按平台验收）
- [ ] 皮肤切换的**运行时热切换**（当前为安装/启用/升级/回退路径）
- [ ] 跨局档案的产品持久化层与真实客户端；卸载（移除安装）后按锁重新取得
- [ ] 全量 `cargo test -p engine-native`（含 examples）与 Gate 1b：被并行在飞 `examples/nc12_bench.rs` 阻断，待其提交后复跑
