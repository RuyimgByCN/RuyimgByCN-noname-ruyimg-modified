# 2026-10-10 T15①：旧线 ABI 账面对齐（734 → 753）

任务来源：[进度主稿 T15①](../../../architecture/native-progress.md)；开工 HEAD `e3c53403`。

## 核查结论：735–753 批已完成，仅宿主声明版本滞后

| 证据 | 结果 |
|---|---|
| [ABI-COORDINATION.md](../../../../../ABI-COORDINATION.md) 领取表 | 735–753 全部登记（2026-09-13/14/15），备注均为"host bump N 随本批落地"；743 空号与 content-compiler 常量缺 743 一致 |
| `content-compiler` additive 常量 | `ADDITIVE_RULE_VM_ABI_VERSION_{726…742,744…753}` 全在（`lib.rs:750` 起）；`codegen.rs` 按 `required_abi.max(常量)` 计算包需求 |
| `engine-core` 宿主实现 | 753 号 `ResolveXiangshu` 在 `effect/primitives/mod.rs:4079` 与 `hash.rs:7220`（canonical hash 臂）落地；抽查 735–752 同型在位 |
| 源 npack | `content/original/ported/{sp,sp2,xianding}` 要求 **745/753/751**（编译器计算所得，证明打包时工具链认为宿主批已落地） |
| 部署副本（改前） | `extensions/{sp,sp2,xianding}` 为 **714/713/706**——旧打包，落后于源 |
| 宿主声明 | `engine-api/src/version.rs` `RULE_VM_ABI_VERSION=734`、ABI-MAX 734 |

compat 判据（`content-runtime/src/compat.rs`）：`host == package || package ∈ [3, host)`。对 host 单调——升 host 只放宽、不收紧，已部署低版本包不受影响；而 745/753/751 的源 npack 在 734 宿主下被拒装。

## 改动内容

1. `crates/engine-api/src/version.rs`：`RULE_VM_ABI_VERSION` 734 → **753**。
2. `ABI-COORDINATION.md`：`ABI-MAX: 734` → **753**，并加 2026-10-10 账面对齐注记（置于 ABI-MAX 行下，不在领取表内，不触碰编号解析）。
3. `scripts/rebuild-pack.sh sp|sp2|xianding` 重打包：重建 npack 与源 npack 模块逐字段一致（`modules` 相等、abi 745/753/751）——打包确定性复核通过。
4. 部署：重建产物覆盖 `extensions/{sp,sp2,xianding}/package.npack.json` 与同名 `<pkg>.npack.json` 双副本（部署 714/713/706 → 745/753/751）。
5. 连带（防漂移守卫暴露）：`content-schema` 的 manifest 默认 `rule_vm_abi_version = RULE_VM_ABI_VERSION` 且入 manifest 哈希，升 753 后 `converted_packages_match_generator` 重新生成产出 753、与仓库内 734 旧产物不一致。按生成器重生成 `packages/extensions/{cardpile-upstream-noname,cardpile-upstream-liuli}/package.npack.json`（各恰好 1 行 diff：abi 734→753；logic/asset 哈希不变）。

**附带发现（已回退，登记待办线索）**：`gen_cardpile_packages` 示例会连带重写 huanle/d3/gujian/gwent/hearth/yunchou/wudian/wuxing 等全部转换包，但生成器内容**落后于仓库内手工维护的包数据**——重生成把 huanle 2026-10-07 手工补齐的酒尾等牌覆盖丢失（16 个测试失败实证：卡数 130≠131、`hnl_jiuwei_6` 未知），已 `git checkout` 回退这批连带重写，仅保留有字节守卫要求的 cardpile 双包。结论：**生成器示例对 cardpile 之外不可直接重跑**，待更新生成器或为各包补齐防漂移守卫（与 T12 逐包切片、T6 装载器收尾相关）。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-api` | **4/4**（含 `abort_bump_without_claim`、`claims_are_unique` 守护对升级后账面） |
| 31 个部署包 compat 复核（host=753） | 全部可装、零拒装 |
| `cargo test --offline -p engine-native --test g1_cardpile`（重生成后） | 5/5（防漂移守卫恢复绿） |
| `cargo test --offline -p content-runtime`（重生成后） | 33/0 |
| `cargo test --workspace --no-fail-fast`（升级+部署+cardpile 重生成、连带重写回退后） | **5098/0/2**，exit 0——与升级前基线总数一致（行为中性）——[完整日志](workspace-after-abi753.log) |

## 边界

- 本项只对齐账面与部署副本；不新增原语、不改引擎行为。
- ②（`hand_limit_bonus` 哈希缺口）与③（19 将 39 技移植排期）仍待领取。
