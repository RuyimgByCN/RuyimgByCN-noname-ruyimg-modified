# T12 swd deferred 消解批 113：灵矢（2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 460）——**149/184 技条目已映射**（deferred 34→33；未映射技名 34 项，183 个名字口径见下）
- **无规则版本/存档版本变化**（无新字段；14 golden 零扰动，未重录）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（lingshi 812–846 行：mod + lingshi_hit/lingshi_draw 双子技逐段核验）、`translate.js`（lingshi=「灵矢」）

## 语义与接线

上游 `lingshi`（计亭 `swd_jiting`，hp4 女）为锁定技，四条效果：

| 面 | 上游 | 接线（原生） |
|---|---|---|
| 攻击范围 | `mod.attackFrom(from,to,distance) = distance - from.countCards("e") * 2` | `state.rs::computed_distance`：持有者（a 侧）装备区每张牌 `-2`，下限 1 由末尾 `max(1)` 钳制（`mashu`/`ljifeng`/`poxiao` 同族） |
| 杀不可闪避 | `lingshi_hit`：`trigger {player:"shaBegin"}` + 武器或防具在装备区 → `trigger.directHit = true` | `kernel.rs::ShaResolveMachine`：`lingshi_direct_hit`（武器或防具槽）并入直中判定——跳过防具判定与 `respond_shan` 询问（`poyue`/`lianwu`/`shoulie` 同族） |
| 摸牌阶段 | `lingshi_draw`：`phaseDrawBegin num++`（马在装备区） | `kernel.rs` Draw 阶段：摸牌基数 `2+1`（`baiyi` 同位；跳摸路径不进入本臂＝上游 `phaseDrawBegin` 未触发同口径） |
| 杀限次 | `mod.cardUsable`：`card.name==="sha" && player.getEquip(5)` → `num+1` | `engine.rs` ShaLimit 臂：宝物在装备区时本回合额外一张杀（`once_used` 额度、回合开始重置；元宝肉/回火同款） |

- `HostBehavior::SwdLingshi`（tag 460）：旗标行为，无机器步（`effect → Flow::Done`；`trigger_meta=None`），四条效果全部内联——`XianjianTianxian`（天弦）同款。
- 包数据 `swd_jiting` 补挂 `host:lingshi`（`packages/extensions/shahai-swd-upstream-noname`），`rehash_npack` 重算资产/逻辑哈希。

**登记差异**：①上游 `attackFrom` 只在 `get.distance(...,"attack")` 面生效，原生沿用同族做法折入 `computed_distance`（顺手牵羊/兵粮寸断等非攻击距离查询同样受减——`mashu`/`poxiao`/`ljifeng` 既有口径）；②`num+1` 以「每回合一次额外杀额度」承载，可观测上限同为 2（第三张仍 `ShaLimit`）；③`countCards("e")` 计数按原生装备区实体数（五栏），改造/附着牌不额外计数。

## 反例实证（逐臂回退探针）

四条臂分别以 `skill_by_name("lingshi")` 关闭后重跑定向测试，均在对应断言失败（[probe-arms-off.log](probe-arms-off.log) 等四份）：

| 关闭的臂 | 失败断言 | 日志 |
|---|---|---|
| 全部四条 | ②「1 件装备 → 攻击范围 +2 覆盖座位距 2」（实得 `Rejected(IllegalTargets)`） | [probe-arms-off.log](probe-arms-off.log) |
| 保留距离、关其余 | ③「有武器 → 不出闪询问（直中）」 | [probe-distance-only.log](probe-distance-only.log) |
| 保留距离+直中 | ④「有马 → 摸牌阶段 2+1」 | [probe-distance-direct.log](probe-distance-direct.log) |
| 保留距离+直中+摸牌 | ⑤「宝物 → 第二张杀可用」（实得 `Rejected(ShaLimit)`） | [probe-distance-direct-draw.log](probe-distance-direct-draw.log) |

恢复全部四臂后定向测试通过（[targeted.log](targeted.log)）。

## 验证

- **定向回归** `swd_deferred_lingshi_range_direct_hit_draw_and_sha_limit`：①无装备时座位距 2 超攻击范围被拒（反例）；②挂马（无射程装备）→ 距离 2-2=1 合法；③挂赤龙牙（武器）→ 1 号持闪仍照常受伤、闪未消耗；④判定阶段→摸牌阶段手牌 +3；⑤挂青龙令珠（宝物）→ 第二张杀可用、额度用尽后第三张 `ShaLimit`。1 passed / 0 failed。
- **swd 套件**：`cargo test --offline -p engine-native --test g1_shahai_swd` **107 passed / 0 failed / 0 ignored**。
- **全量**：`cargo test --offline -p engine-native --no-fail-fast` **895 passed / 0 failed / 1 ignored**（exit 0；1 ignored＝既有 `xly_state_growth_probe` 诊断探针）——[engine-native-tests.log](engine-native-tests.log)。
- **Gate 1b**：独立 `CARGO_TARGET_DIR` 下 `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0**——[gate1b-all-targets.log](gate1b-all-targets.log)。
- **键表守护** `host_behavior_keys_bijective` 更新为 **457**（tag 460；`ALL` 数组长度同步）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `content-runtime` 的 `rehash_npack`（按 `asset_paths` 重算 `source_hash` + logic/asset 哈希）重算；真实安装（`import_extensions`）通过。
- **14 golden 零扰动**：`cargo test --offline -p engine-native --test replay_scenarios` **2/2**（含清单联动守护）——[replay-golden-v301.log](replay-golden-v301.log)。
- **维护守护**：`production_sources_have_no_debug_output_or_env_switches` **1/1**——[maintenance-guards.log](maintenance-guards.log)。
- **装载面**：`cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast` **66 passed / 0 failed / 0 ignored**（exit 0）——[deps-tests.log](deps-tests.log)。
- 未触碰标准目录（等价性由定向测试与全量零 golden 位移体现）。

## 结论

- **149/184 映射，deferred 34→33**；**计亭（`swd_jiting`）整将闭环**（`guanhu`+`lingshi`）——79 将中 51 将全映射。
- `T12-SWD-DEFERRED-SKILLS` 仍开放（余 34 个未映射技名：多步/跨包引用/缺事件型）。
