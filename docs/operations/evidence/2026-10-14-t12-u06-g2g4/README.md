# T12/U06 G2–G4 产品通道验收（示例来源=欢乐卡牌，2026-10-14）

## 任务来源

[台账 U06](../../migration/upstream-extension-reconstruction.md) G1+G5 已达成后，按 E04/G2–G4 契约（「AI 只见合法座位信息，配置持久、资源释放与交互可复查」）补产品通道验收——U06 为示例来源。

## 交付（g1_huanle.rs 新增 `u06_g2g4_channels_acceptance`，五通道一测）

| 通道 | 断言 |
|---|---|
| **目录（G2）** | 7 张扩展卡入合并目录（base+7）；卡名带 `hnl_` 前缀（符号表命名空间隔离） |
| **搜索（G2/G3）** | 7 卡全部按名 `position` 检索命中 |
| **资源（G4）** | npack `content_hash` 与安装记录一致（指纹保真）+ 安装后 `assets/cards.json` 与仓库源**字节级一致**（资产原样保真） |
| **配置（G3）** | store 重开后激活集存续（`upstream-noname/huanle-cards` 持久）；`manifest.enabled=false` 的目录导入被跳过并进 `skipped` 清单 |
| **界面/AI 隔离（G4）** | `ai_session(1)` 视角：自己手里的扩展牌具体可见（`HandView::Own` 含 def），他人手牌不泄露；`ai_session(2)` 视角下 1 号手牌为 `Hidden`；扩展 def 可按 id 检索（UI 按 def_id 渲染） |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | **1/1** 绿（五通道一测）——[日志](u06-g2g4-targeted.log) |
| g1_huanle 全目标 | **21/21**（20 既有+1 本批） |
| engine-native 全量 | **689/0/0**（688→689）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0（两跑各抓出 1 条 unused 项，已清）——[gate1b-all-targets.log](gate1b-all-targets.log) |

## 坑位实录

- 安装记录 `content_hash` 对 **package.npack.json** 字节（非 assets/cards.json）——资源断言分两层：npack 指纹 + 安装后 assets 字节与源一致（`extension_root` 指回 drop-in 目录，无复制）。
- `ActiveSet.packages` 键为 `namespace/package_id` 复合 slot（非 struct）——按 key 后缀断言。
- `bridge()` 测试助手返回 Result（本批忘 `.expect` 由 Gate unused 友邻暴露）。

## U06 状态推进

G1 ✓ + G5 ✓ + **G2–G4 通道 ✓**（本批）——U06 欢乐卡牌成为**首个全通道（G0–G5）验收通过的来源**；通道验收模板（目录/搜索/资源/配置/隔离五断言）可复制到其余 14 包。
