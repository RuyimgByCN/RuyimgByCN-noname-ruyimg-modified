# O4 内容债修正：离线回退路径 + 跨包 roster 依赖声明（2026-10-14）

## 任务来源

[O4 交付测量](../2026-10-14-o4-wasm-delivery/README.md) 登记的两项存量内容债。

## 债务① 修复：`loadEmbeddedStandard` 离线回退改为全依赖闭包装载

- **根因**：内嵌 standard 的 roster 刻意跨包引用 refresh 技能（ADR 0542 delta），且 refresh/shenhua/yijiang 等**逐级跨引**——单包/部分装载的 roster 闭包校验必然拒绝（传递闭包不可截断）。
- **修复**（crates/engine-wasm）：新增 `fallback_closure.rs.inc` 内嵌表（standard 之外的 **18 个依赖 npack** 逐字 include_bytes，路径相对 src/）+ `loadEmbeddedStandard` 重写为「load_standard_package + 逐个 load_package 闭包 npack + 统一闭包校验」。
- **体积代价（实测）**：wasm 8,857,322 → **15,176,936 B（14.5MB 原始）**（+6.8MB=18 个 npack JSON 原文；brotli 后 JSON 压缩比 ~8:1，预计 +1MB 级——离线回退的真实成本）。**登记**：如 15.6MB 原始体积不可接受，回退路径可改为「错误提示要求注入内容束」（语义上等于移除离线回退）——产品决策项。
- **回归**：`loadEmbeddedStandard` 单调用（无 bundle）→ startGame → autoStep×40 → observerJson 全链路通过——[smoke-embedded-fallback.txt](smoke-embedded-fallback.txt)。bundle 路径回归不变绿（loadContentBundle 19 包照常）。

## 债务② 修复：yijiang/tw manifest 依赖声明补齐

- **发现**：yijiang.liyan→newjiang.dcduliang、yijiang→bingshi、tw.sb_sp_zhugeliang→sb.sbhuoji——roster 跨包引用未在 manifest.dependencies 声明。
- **修复**（`apps/content-cli/examples/add_package_deps.rs`）：外科 manifest-only 编辑——追加 dependencies + `compute_logic_hash()` 重算（**不触碰 skill_index/character_index/modules**——roster def_ids 与现存编译产物绑定，重编译全面失配：rebuild-pack.sh 实测 tw 悬空 2→36，已回滚改用外科路径）。
  - yijiang：deps +newjiang +bingshi，logic_hash `7c8eda47…`→`ef08c0f0…`
  - tw：deps +sb，logic_hash `c3d80448…`→`d357801c…`
- **连带**：`metadata.json` 同步补齐（rebuild-pack.sh 的 manifest 来源）；4 个引用两 npack 的测试目标 200/0 全绿。

## 验证

| 项 | 结果 |
|---|---|
| wasm32 编译（含新内嵌表） | exit 0 |
| 债务① 定向回归（Node） | 通过（[smoke-embedded-fallback.txt](smoke-embedded-fallback.txt)） |
| 债务② 定向回归（4 npack 测试目标） | 200/0 |
| bundle 路径冒烟复核 | 通过（loadContentBundle 照常） |
| engine-core 全套 | **3,778/0/0** |
| Gate 1b（engine-native --all-targets） | exit 0 |

## 登记边界

- wasm 原始体积 8.9→14.5MB 为离线回退的真实成本；产品若弃离线回退可回退本批内嵌（单提交粒度）。
- 其余 ported 包 roster 的跨包引用已由内容锁承载（闭包扫描 3 处违约全部由 bingshi/newjiang/sb 提供——已声明+已入种子闭包）。
