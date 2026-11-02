# C331 占用 WS7 的边界报告与裁决 31（2026-11-02）

核验对象：Main `f5e560db`（C331 phase7 合并提交，父 `a4483b64`＋`6d36f47a`），规则 331／NGSA 106／content schema 5／ABI 758。本页事后补齐裁决 22／29 要求的精确边界，并登记用户对 WS 号码的答复；不改代码。

## 一、协议面实际改动

`a4483b64..f5e560db` 中所有线格式相关路径的完整差异见 [wire-diff.patch](wire-diff.patch)（sha256 `d80b1d6d…`）。归纳：

| 位置 | 改动 | 对旧端的影响 |
| --- | --- | --- |
| `crates/engine-api/src/choice.rs` `ChoiceRequest` | `min_count`／`max_count` 由 `u8` 放宽为 `u32`；自定义反序列化拒绝 `min_count > max_count` | 数值 ≤255 时 JSON 字节不变；>255 时 WS6 端按 `u8` 解析失败 |
| `crates/engine-protocol/src/lib.rs` | `NETWORK_PROTOCOL_VERSION` 6→7 | — |
| `crates/engine-api/src/version.rs` `VersionBundle` | `network_protocol` 6→7；`content_schema` 3→5（schema5 已由裁决 29 批准） | — |
| `apps/game-server/src/lobby.rs`、`session.rs` | Hello 仍要求 `network_protocol` 完全相等，提示改为 “WS7 Hello required” | WS6 客户端在握手阶段被拒，不进入房间 |
| `apps/web-client/src/engine/networkClient.ts` | 客户端常量 7，错误文案改为动态 `WS${NETWORK_PROTOCOL_VERSION}` | WS7 客户端连 WS6 服务端同样在握手阶段被拒 |

消息类型、命令、`ChoiceOption`、`ChoiceKind` 均未改；选序沿用既有 `OrderTriggers`／`Answer`。

## 二、为什么需要放宽、为什么不能留在 WS6

- 放宽来自已批准范围：[ADR 0788](../../../adr/0788-native-player-trigger-order.md) 要求“第257项可达，不能只依赖256上限”；回归 `apps/game-server/tests/c331_wide_choice.rs` 以 255／256／257 张实体牌验证最终询问、恢复后选择与取消。
- 若保留 WS6：旧 WS6 端在出现 >255 的选择时才于对局中途解析失败，属于非原子的运行期不兼容；协议号注释规定破坏性线格式改动须升号。升 WS7 把不兼容前移到握手，旧端得到明确拒绝且不进入对局。
- 改回 WS6 需要另设按对端版本的计数守卫并重新认证，且仍无法让 WS6 端回答 >255 的选择。

## 三、裁决 31（2026-11-02，用户在对话中明确答复）

用户对“批准 C331 占用 WS7、R05 改用 WS8 吗？”选择 **“批准，R05 改 WS8”**，记录见 [user-decision31.json](user-decision31.json)。

- C331：规则 331／NGSA 106／ABI 758／**WS7**（追认本页第一节的线格式改动与握手拒绝）。
- 改命 332：沿用当时的 WS7，不另升协议号（仍按裁决 24 只改规则 331→332）。
- R05：规则 333／NGSA 107／ABI 759／**WS8**，替代裁决 28／29 中 R05 的 WS7；其余合同、顺序与条件不变。
- 本裁决只处理 WS 号码，不表示 332、R05 已实施，也不放行 NC10 或其他门禁。

## 四、仍需执行

按裁决 25 发布 `f5e560db` 前，先在该提交补跑并入库前端／WASM 重建／Worker smoke／跨目标／Tauri IPC 原始日志及 7 个浏览器案例（见进度主稿 2026-11-02 扫描）。
