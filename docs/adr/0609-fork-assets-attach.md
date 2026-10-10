# ADR 0609：fork 5 内容扩展静态资源挂接（Phase 1）

> 状态：已落地（2026-08-05）
> 领取：C 段（omp），2026-08-05
> 前置：ADR 0581（ddd 收尾）、ADR 0606（jiange）、fork 清点 ADR 0605

## 背景

用户指示：静态资源（图片/语音等）也必须迁移到本项目，不再视为表现层豁免。
此前约定（皮肤/UI/音频不进入 npack）被本次需求反转：fork 5 个内容扩展的
全部资产须随各自 npack 打包投放。

编号说明：0607/0608 已被 Claude A 段 gwent 系列（shoulijian ABI 642 /
dudayuanshuai ABI 643）的 ADR 文件占用，本系列顺延为 0609-0611。

## 决策

1. 每个 fork 内容扩展的资产复制到 `content/original/ported/<pkg>/assets/`，
   以 `resource-compiler` 的 `--assets` 编译进 npack（asset_id = URL-safe 相对路径，
   `/` → `_`，如 `character_yxs_qinqiong.jpg`、`audio_card_kuwu_male.mp3`）。
2. 角色立绘通过 `portrait_asset` 字段挂接（包内 asset id），仅精确匹配 name_key；
   无匹配者不挂接（不发明别名）。
3. 卡牌/语音无 schema 字段可引用（PackageCardEntry 无 asset 字段），资产入包即可，
   UI 按 `card_<name>` / `audio_card_<name>_<sex>` 约定引用。
4. `compute_logic_hash` 只覆盖语义字段，资产加入不影响逻辑哈希；`asset_hash`
   覆盖资产——重打包后 asset_hash 必然变化，属预期。
5. 部署：npack 同时写入 `extensions/<pkg>/<pkg>.npack.json` 与
   `extensions/<pkg>/package.npack.json`；资产复制到 `extensions/<pkg>/assets/`。

## 落地清单

| 包 | 资产数 | 内容 | portrait_asset 挂接 |
|---|---|---|---|
| yxs（英雄杀） | 44 | 43 立绘 + 1 音频 | 43/43 |
| huanle（欢乐卡牌） | 16 | 6 卡图 + 10 语音 | —（无武将） |
| wandianlunsha（玩点论杀） | 13 | 3 卡图 + 9 立绘 + 1 语音 | 9/9 |
| ddd（3D精选） | 31 | 31 立绘（fork 版替换） | 31/31 |
| shahai（杀海拾遗） | 577 | 573 图 + 4 语音 | 331/332（`diy_sp_zhouyu` 无精确匹配，留空） |

## 验证

- 5 包重打包部署完成；32 包空 store 冷导入通过（content_lock=3468c49de13c…）。
- `cargo test -p engine-core --test real_extensions_lock`：1/1 通过
  （全部扩展安装 → 锁加载 → 引擎开局）。
- 逻辑哈希对 yxs 复核不变（资产无关语义字段，符合设计）。

## 后续

- ADR 0610：上游 26 包角色立绘挂接（image/character 2,576 文件按 name_key 归属）。
- ADR 0611：技能语音挂接（audio 按包归属）。
- 卡牌图/语音的引擎侧消费（皮肤/立绘/语音查找 API）不在本 ADR 范围。
