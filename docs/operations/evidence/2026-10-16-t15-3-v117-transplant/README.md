# 2026-10-16 T15③ / D6：上游 v1.11.7 残差 19 将 39 技旧线移植

核验日期：2026-10-16。范围：旧线 `engine-core`（禁止改 `engine-native`）。  
裁决：D6 启动移植（[决定记录](../2026-10-16-d4-d8-decisions/README.md)）。流程沿用 v1.11.6 批 146：sync 残差清单 → `.skill` / `characters.json` → `rebuild-pack` → 四副本部署 → 种子敏感三件套。

## 清单与状态

见 [inventory.md](inventory.md)。馆藏闭包：**2576/2576 武将 + 6190/6190 技能全部 `ported`**（`source_synced` 残差 0）。  
39 技均为诚实 noop（`modifier hand_limit add 0` + `INTENTIONAL_EMPTY_NOOP` 注记，保真度债 ADR 0597）；零新 Effect/Trigger 原语。

包分布：offline×4 将 / xianding×3 / collab×2 / bingshi·clan·huicui·mobile·newjiang·old·onlyOL·sb·sp2·tw 各 1。  
既有 roster 纠偏：`clan_xunguan` 首技 phantom id→`clandunji`；`jd_sb_sunshangxiang` 枭姬改本地 `jdsbxiaoji`；`shen_zhangjiao` `yizhao`→`dcyizhao` 且 max_hp 3→4。

## ABI

**无变化。** host `RULE_VM_ABI_VERSION` / `ABI-MAX` 仍为 **753**；各包 `rule_vm_abi_version` 未抬升（noop 零原语）。未改 `ABI-COORDINATION.md` 领取表。

## 部署

13 包 `rebuild-pack.sh` 重建后同步：

1. `content/original/ported/<pkg>/<pkg>.npack.json`
2. `extensions/<pkg>/<pkg>.npack.json`
3. `extensions/<pkg>/package.npack.json`
4. `.npack-store/packages/official__<pkg>/0.1.0/package.json`（存在则覆盖；offline/onlyOL 另有 `content/original/packages/`）

日志：[rebuild-pack.log](rebuild-pack.log)。

## 测试

`CARGO_TARGET_DIR=.../.scratch/target-oldline`，禁止 `cargo clean`。

| 命令 | 结果 | 日志 |
|---|---|---|
| `cargo test --offline -p engine-core --no-fail-fast` | **3778 通过 / 0 失败 / 1 忽略**（exit 0） | [engine-core-tests-final.log](engine-core-tests-final.log) |
| `cargo test --offline -p engine-api --no-fail-fast` | **4/0**（含 ABI 账面守护） | [engine-api-tests.log](engine-api-tests.log) |
| `cargo run --offline -p conformance` | **ALL PASS** | [conformance.log](conformance.log) |
| `bash scripts/cross-target-hash.sh` | **ALL CROSS-TARGET CHECKS PASSED** | [cross-target-hash.log](cross-target-hash.log) |
| `node scripts/sync-original-content.mjs --verify-only` | **通过**（legacy-source 366 / portraits 2703） | [sync-verify.log](sync-verify.log) |

种子敏感：roster 平移后对账计数（skill/character/modifier）、collab `olchouniu`/`olsishe` canonical hash 重录、offline/xianding 开局选择排空与 jun_zhaoshou/tanhu 种子窗扩展。

## 回填

- [native-progress.md](../../../architecture/native-progress.md) T15 行③、D6 落点
- [upstream-1.11.5-delta.md](../../../migration/upstream-1.11.5-delta.md) 本轮节
