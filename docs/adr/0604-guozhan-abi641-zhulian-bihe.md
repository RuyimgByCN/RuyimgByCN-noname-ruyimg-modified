# ADR 0604：国战 ABI 641——`GuozhanUseZhulianBihe` 原语（8 文件接线）+ gz_ 技能 DSL 启动

> 首个国战专属 `EffectRequest` 变体，完整 8 文件 VM 驱动接线。用户明确要求「动 ABI」。

## 背景

ADR 0603 完成了国战引擎层规则件，但全部用 `Custom` continuation 绕过了 ABI。用户要求
**动 ABI**——新增国战专属 `EffectRequest` 变体，走完整 VM 驱动接线。珠联璧合「当无中生用」
是最自然的候选：它需要 DSL 可调用（`effect guozhan_use_zhulian_bihe(self)`），必须走 VM 管线。

## 做了什么

### ABI 641 领取
- `ABI-COORDINATION.md`：`ABI-MAX: 641`，领取表加 `| 641 | GuozhanUseZhulianBihe | A (Claude) |`。
- `engine-api/src/version.rs`：`RULE_VM_ABI_VERSION: u32 = 641`。
- `abort_bump_without_claim` 测试验证 ABI-MAX 与 RULE_VM_ABI_VERSION 一致（已过）。

### `EffectRequest::GuozhanUseZhulianBihe { player }`（rule-api/effect.rs）
消耗 `ZHULIAN_BIHE_MARK` 摸两张牌（无中生有效果）。

### 完整 8 文件接线（VM 驱动）

| # | 文件 | 接线 |
|---|---|---|
| 1 | `rule-api/src/effect.rs` | 枚举变体 `GuozhanUseZhulianBihe { player }` |
| 2 | `rule-vm/src/bytecode.rs` | `effect_tag::GUOZHAN_USE_ZHULIAN_BIHE: u16 = 639` |
| 3 | `rule-vm/src/vm.rs` | pop 臂：`GUOZHAN_USE_ZHULIAN_BIHE =>` pop player → 构造 EffectRequest |
| 4 | `engine-core/src/vm_host.rs` | 无需（变体无 skill 字段，透传） |
| 5 | `engine-core/src/hash.rs` | `put_u8(h, 0)` + `&641u16` + player.raw() |
| 6 | `engine-core/src/effect/primitives/mod.rs` | 调度臂 → `resolve_guozhan_use_zhulian_bihe` |
| 7 | `resolve_choice/mod.rs` | 无需（非可恢复 choice） |
| 8a | `content-schema/src/ir.rs` | `StmtIr::EffectGuozhanUseZhulianBihe { player, span }` |
| 8b | `content-compiler/src/typecheck.rs` | `"guozhan_use_zhulian_bihe" =>` 臂 |
| 8c | `content-compiler/src/codegen.rs` | lowering → `Op::HostEffect(GUOZHAN_USE_ZHULIAN_BIHE)` + ABI 641 |
| — | `content-compiler/src/lib.rs` | `ADDITIVE_RULE_VM_ABI_VERSION_641` + `required_rule_vm_abi` 链 |

### 原语实现（`pack_tw_1.rs::resolve_guozhan_use_zhulian_bihe`）
检查 `ZHULIAN_BIHE_MARK` 存在 → 消耗 1 层 → `run_effects(DrawCards { count: 2 })`。

### gz_ 技能 DSL 启动
- `content/guozhan/gz_jianxiong.skill`：国战奸雄（受伤拿伤害牌），def_id=2398363843。
- `content/guozhan/gz_wei_skills.skill`：6 个魏国核心技能（gz_ganglie/gz_luoyi/gz_yiji/
  gz_shensu/gz_jushou/gz_jieming）。
- `content/guozhan/gz_other_skills.skill`：9 个蜀/吴/群技能（gz_kongcheng/gz_jizhi/
  gz_keji/gz_shushen/gz_mouduan/gz_tianxiang/gz_duanchang/gz_mingshi/gz_kuangfu）。
- conformance 加 `guozhan_gz_jianxiong_repro` + `guozhan_wei_batch_repro` +
  `guozhan_other_batch_repro` + `guozhan_zhulian_bihe_abi641`（全部编译验证 + def id 校验）。
- 共 16 个 gz_ 技能 DSL 编译成功，均为简化语义（标注近似），建立可复制模式。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（含 gz_jianxiong 编译 + ABI 641 原语编译）✅ |
| `abort_bump_without_claim` | **过**（ABI-MAX 641 = RULE_VM_ABI_VERSION 641）✅ |
| 国战单测 | **21 passed** ✅ |
| 国战烟雾 | **2 passed** ✅ |
| 失败集合 | **257 = 257 双向空**（身份局零回归）✅ |
| warnings | **59** ✅ |

**ABI 641 全链验证**：DSL `effect guozhan_use_zhulian_bihe(self)` → codegen →
`Op::HostEffect(GUOZHAN_USE_ZHULIAN_BIHE)` → VM pop → `EffectRequest::GuozhanUseZhulianBihe` →
hash（ABI 641 tag）→ `resolve_guozhan_use_zhulian_bihe` 原语执行。端到端可编译可执行。

## 内容完整落地（补充）

**256 武将技能 DSL 全部实现**：
- 197 个 `gz_` 前缀技能（16 手工 + 181 批量）
- 99 个无下划线 `gz` 前缀技能（`gzguishu`/`gzanyong` 等，另一套命名）
- 168 个非 gz 前缀复用技能（`guicai`/`fankui`/`wanwei` 等，国战包自包含）
- 共 **464 个技能 DSL**，分布在 `content/guozhan/gz_*.skill`

**def id 统一**：手工技能（原硬编码 2162720702 等）改为 CRC32，与 `guozhan_characters.json`
的 skills 字段一致，消除「unavailable skill」加载失败。

**`bootstrap_guozhan` 切换国战武将池**：`ensure_guozhan_package`（standard_pack.rs）把
256 个 gz_ 武将 + 464 技能 DSL 编译进 content registry；`bootstrap_guozhan` 从此从国战
武将池分配双将（而非身份局武将池）。国战烟雾测试通过（256 武将可用）。

**国战卡牌**：`content/guozhan/guozhan_cards.json`——12 张装备卡牌（`{card_def_id,name,kind}`），
格式对齐 `PackageCardEntry`（kind ∈ weapon/armor/horse_off/treasure），通过
`guozhan_package().card_index` 接入 content registry（`GUOZHAN_CARDS`/`guozhan_card_entries`）。

**国战牌堆**：`guozhan_deck_specs()`（deck.rs）= 身份局牌堆 + 12 张国战装备卡（feilongduofeng/
wuliu/sanjian/jilinqianyi/yuxi/dinglanyemingzhu/liulongcanjia/zhaoshu 武器 + minguangkai/
taipingyaoshu/huxinjing 防具 + jingfanma 马），`bootstrap_guozhan` 用此牌堆 → 国战对局
可摸到国战装备。

**国战锦囊进牌堆**：12 种国战专属锦囊（gz_haolingtianxia/gz_kefuzhongyuan/gz_guguoanbang/
gz_wenheluanwu/zhibi/chiling/huoshaolianying/lulitongxin/lianjunshengyan/shuiyanqijunx/
yiyi/yuanjiao）加入 `guozhan_deck_specs`（`CardKind::Other` + crc32 def），国战对局可摸到。

**留后续**：12 种锦囊的**结算**各含多目标 choice 流（如 gz_haolingtianxia 每目标选「出杀/
弃牌」+ 势力判定），完整实现需新增 12 个 CardKind + choice 原语（ABI 642+，每个 ABI 级），
判定为独立后续工作，已登记 ABI-COORDINATION。

## 最终验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（国战向量重钉至 fd901734/9e6fac01，含国战牌堆）✅ |
| 国战单测 | **21 passed** ✅ |
| 国战烟雾 | **2 passed**（256 国战武将池 + 12 装备卡牌国战牌堆对局）✅ |
| 失败集合 | 257 → **247**（0 新增；10 个被他人修复，非我引入）✅ |
| warnings | **59** ✅ |

## 后续

国战内容侧全部就位：256 武将 + 464 技能 DSL + 31 卡牌索引，`guozhan_characters.json`
skills 字段全填充（无空数组）。技能均为简化近似语义（标注近似，与 Task2 保真度清单
模式一致），真实语义可后续逐个精化（复用 ADR 0604 的 ABI 接线模式）。
