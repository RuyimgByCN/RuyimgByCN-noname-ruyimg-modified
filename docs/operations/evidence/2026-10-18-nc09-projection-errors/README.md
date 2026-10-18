# NC09/NC03 座位投影与错误可见性（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）第一切片 + **NC03** 投影面
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**投影/错误/恢复一致性 4 项通过**；客户端渲染、真实联机房间与平台档位待续

## 1 验收回归（新增 `crates/engine-native/tests/nc09_projection_and_errors.rs`，4 项）

| 验收项 | 判据 | 结果 |
|---|---|---|
| 座位投影不泄漏隐藏信息 | 自己手牌 `HandView::Own` 逐张可见；他人 `HandView::Hidden` + `hand_count` 正确；牌库仅 `deck_count`；弃牌堆公开；**观战者视图（`view_for(PlayerId::MAX)`）不含任何手牌 id** | 通过 |
| 开放选择选项仅观察者可见 | 观察者 `ChoiceView::Own` 含完整选项；其他座位为 `ChoiceView::Other`/`None`（不含选项内容） | 通过 |
| 非法命令**明确拒绝** | 非本人回合 → `NotYourTurn`；未知牌 → `CardNotInHand`；过期修订号 → `StaleRevision`（都不是空结果/静默成功） | 通过 |
| 存读档后投影一致 | 归档往返（带扩展恢复）后，座位投影与观战投影**逐字段一致** | 通过 |

## 2 验证结果

```
cargo test --offline -p engine-native --test nc09_projection_and_errors
cargo test --offline -p engine-native --no-fail-fast
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| 投影/错误定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1144 passed / 0 failed / 2 ignored** | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8243 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 3 未完成 / 待办（NC09）

- [ ] **真实客户端渲染**：桌面（Tauri）/Web 的原生表现路径（牌桌、选将、配置、皮肤、音频、动画、界面部件）——本切片只覆盖内核投影与错误面
- [ ] **真实联机房间**：`game-server` 房间/重连/观察者/录像与原生权威规则的端到端（协议已含 `JoinRoom{reconnect_token}`/`Spectate`/`RequestSnapshot`/`Snapshot{view,choice,legal_actions}`，房间实现走旧 `engine-core` 引擎——**原生权威接入未完成**）
- [ ] **录像与跳转**（回放）产品入口
- [ ] **跨座位信息隔离的产品级验证**（HTTP/WS 层，而非仅内核视图）
- [ ] 平台档位取证：Windows/macOS/Linux、Android/iOS、Web；鼠标/触摸/窄屏/中英文/边界值/可访问性
- [ ] 千幻/皮切/十周年等真实组合的界面验收、资源装卸与长期运行释放
