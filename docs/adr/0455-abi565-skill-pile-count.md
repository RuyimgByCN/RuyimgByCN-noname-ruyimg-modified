# 0455 ABI 565 skill_pile_count + 钟会链 zyexin/zzili（Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

expansion（技能牌堆）链路的第一个积木：`skill_pile_count(player, skill_def)` 表达式
查询,解锁钟会链 zyexin（存权）/zzili（觉醒判定）,并为后续 quanji 系铺路。

## 变更

### 引擎（ABI 565）

| 文件 | 变更 |
|---|---|
| `rule-api/src/query.rs` | `QueryRequest::SkillPileCount { player, skill_def }` + `QueryResponse::SkillPileCount { count }`（响应端与另一 agent 在途实现对齐） |
| `rule-vm/bytecode.rs` | `query_tag::SKILL_PILE_COUNT = 31`（30 已被 SKILL_CONTENT_FLAG 占用） |
| `rule-vm/vm.rs` | 请求解码（响应解码复用既有 `{ count }` 分支） |
| `content-compiler/typecheck.rs` | 表达式注册 + arity(2) + 第二参 expect_u32 |
| `content-compiler/codegen.rs` + `lib.rs` | tag 映射 + ABI 565 常量 |
| `engine-core/query/mod.rs` + `cache.rs` | 查询实现（`skill_pile_cards(player, def).len()`）+ 序列化 tag 31 |
| `engine-api/version.rs` | 565 |

### 技能（2 个,yijiang 钟会）

| 技能 | DSL 表达 | 未建模/近似 |
|---|---|---|
| `zyexin`（自新） | `damage owner_is_target/owner_source` 双 trigger → `gain_top_card_to_skill_pile(self)` | 自伤双事件各触发一次 |
| `zzili`（自立） | `phase_prepare` + `if 3 < skill_pile_count(self, 3289308596)` → `change_max_hp(-1)` + 觉醒标记 | 「获得排异」未建模（需使用技能牌堆牌原语） |

## 落地

- yijiang 126 技能 / 27 武将 / ABI 565。
- `extensions/yijiang/` 同步（package.npack.json 同源,README 更新）。

## 备注

并发说明：实现期间 rule-api 的 QueryResponse 端被另一 agent 并行添加（`{ count }` 字段
设计）,已对齐为「请求端 player/skill_def + 响应端 count」;其 rule-api 在途编辑曾导致
瞬时编译失败,稳定后 6 crate cargo check 全绿。

## 验收

- 6 crate cargo check 通过;yijiang 打包 ABI 565;zyexin/zzili 触发索引正确。
