# ADR 0530：英雄杀归档扩展迁移（yxs）

- 领取：C 段 omp
- 来源：`archive/upstream/noname/apps/core/extension/英雄杀`（GNU GPL v3）
- 交付：`extensions/yxs`（43 武将 / 105 技能 / ABI 460）

## 内容

英雄杀（yxs）是含 43 名武将、113 个上游技能定义的内容扩展。按「旧 JS 语义参考 →
DSL 重写 → npack → extensions」流程迁移：43 武将全部登记入角色索引，105 项技能
定义（20 项真实 DSL 运行时 + 85 项元数据登记），依赖 standard（复用
mashu/wushuang/jianxiong/longdan/keji/xiaoji/jieyin）与 shenhua（复用
qiangxi/kanpo）两包。

## 真实运行时技能（20 项，节选）

- `yxs_fanji`（反击）：受伤害后可对来源使用虚拟【杀】
- `yxsweiyi`（威仪）：受伤害后令来源弃置两张牌
- `tuqiang`（图强）：使用或打出【闪】后摸一张
- `taiji`（太极）：使用或打出【闪】后可使用虚拟【杀】
- `xumou`（蓄谋）：结束阶段摸三张并翻面
- `taolue`（韬略）：手牌上限 +1
- `wumu`（武穆）：黑色【杀】无视距离、红色【杀】不计出杀限制
- `wluoyan`（落雁）：防止伤害改为失去体力
- `xiangma`（相马）：按体力修正进攻/防御距离
- `fengyi`（凤仪）、`zhijie`（知节）、`huli`（护理）、`sheshu`（射术）、
  `chujia`（初嫁）、`yixin`（医心）、`zbudao`（布道）、`luobi`（落笔）、
  `guifu`（鬼斧）、`qiangyun`（强运）、`zhexian`（谪仙）

## 元数据登记（85 项，无运行时行为）

依赖目标替换（门神）、标记生命周期（浴血）、动态技能复制（易装/统御/归隐）、
判定分支（刺客/强掠/演绎）、全局用牌限制（独裁）、座次交换（迁都）、濒死即死
（弑亲/鸩杀/刺酋）等未移植机制，登记元数据并文档化近似，详见
`content/original/ported/yxs/yxs_meta.skill` 各技能注释。

## 验收

content-compiler 514/514、conformance 8/8、28 包空 store 冷导入通过。
源码 `content/original/ported/yxs/` 与 `extensions/yxs` 的 npack 逐字节一致。
