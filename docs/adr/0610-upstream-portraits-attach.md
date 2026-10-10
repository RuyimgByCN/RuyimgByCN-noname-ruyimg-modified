# ADR 0610：上游 26 包角色立绘挂接（Phase 2）

> 状态：已落地（2026-08-05）
> 领取：C 段（omp），2026-08-05
> 前置：ADR 0609（fork 5 包资产）

## 背景

上游 `apps/core/image/character/` 2,576 张角色立绘（376M）按 name_key 归属到
26 个上游包（bingshi/clan/collab/diy/extra/huicui/jsrg/key/mobile/newjiang/
offline/old/onlyOL/refresh/sb/shenhua/shiji/sixiang/sp/sp2/standard/sxrm/tw/
xianding/yijiang/yingbian）。

## 决策

1. 以各包 `characters.json` 的 `name_key` 精确匹配 `image/character/<name_key>.<ext>`；
   匹配即复制到 `content/original/ported/<pkg>/assets/character/` 并挂
   `portrait_asset = "character_<name_key>.<ext>"`。无匹配不发明别名。
2. standard（引擎内嵌包）走 `content/standard/` 源重打包，匹配规则追加
   `std_` 前缀回退（30/30 全部命中）。
3. 非 ASCII 文件名（拼音变音符号，如 `sunsháo.jpg`）经 `url_safe_asset_id`
   变换为 `_`（`character_sunsh_o.jpg`）——portrait_asset 使用变换后的 id
   （编译器行为，12 处修正）。
4. 重打包必须保持 `logic_hash` 不变（资产不参与逻辑哈希，属既有设计）——
   全部 25 包 SAME-HASH 核验通过后部署。

## 落地清单

- 24 个 ported 包：2,366 张立绘挂接（331/332 shahai 例外已在 0609；0 匹配的
  fork 5 包 + jiange 用 fork 资产，不重复挂接）。
- standard：30/30（含 std_ 回退）。
- 部署：npack 双副本 + `extensions/<pkg>/assets/` 同步。

## 验证

- 25 包 logic_hash 与现网 npack 逐一比对 SAME-HASH（资产变更不影响语义）。
- 32 包空 store 冷导入通过（content_lock=ee263cfb…）。
- `real_extensions_lock` 门禁 1/1 通过。
- 上游 2,576 张中约 180 张未命中（对应未迁移/废弃武将），留档不挂。

## 后续

- ADR 0611：音频挂接。
- 皮肤/立绘的运行时消费 API 不在本 ADR 范围。
