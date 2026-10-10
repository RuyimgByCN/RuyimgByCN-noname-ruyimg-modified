# 上游 noname 剩余内容清点（旧线历史记录）

> 2026-09-23 范围更正：noname 与 noname-liuli 的全部扩展已被用户明确纳入重构支持，当前目录基线为 8 + 29 = 37 个来源实例。
> 本页“模式/UI/配置/表现层不可迁移、全部处理完毕”是旧 DSL 管线范围结论，不能作为原生豁免或验收。
> 新方案见 [两仓全部扩展重构](upstream-extension-reconstruction.md)，包括子包、插件、混合规则、资源与运行所需依赖。

> 历史参考（2026-09-16 标注）：本文描述旧线及所注明日期的状态，文中的“当前”、阶段验收和统计不代表新原生线完成度。最新入口见 [旧实现参考](../architecture/legacy-reference.md)。

> 目标：按「旧 JS 语义参考 → DSL 重写 → npack → extensions」把
> `archive/upstream/noname` 下的内容全部处理。本文件登记其中**不进入 npack 管线**
> 的剩余项及其原因，与各包迁移交付共同构成完整清点。
> 覆盖范围以 2026-08-03 17:30 快照为准；**2026-08-08** 上游已更新至 **1.11.5**
> （`dd7d6916`）；**2026-08-15**（第二轮，`2eb3513b`）复核 catalog 为 **2524 / 2524
> 武将、6064 / 6064 技能全部 `ported`**（v1.11.5 新增 9 将 15 技当轮全部移植闭包）。
> 同日保真度：5529 个 `.skill`，完整 5171、近似 358、TODO **0**、noop 2181。
> **2026-08-21 复核注**：当前活数字为 **2526 / 2526 武将、6068 / 6068 技能**，
> `.skill` 5533，完整 5086、近似 447、TODO 0、noop 2094，
> `missing_character_portraits` 45——以 `content/original/catalog.json` 与
> [`../content-fidelity-ledger.md`](../content-fidelity-ledger.md) 为准。
> 明细见 [`../content-fidelity-ledger.md`](../content-fidelity-ledger.md)
> 与 [`upstream-1.11.5-delta.md`](./upstream-1.11.5-delta.md)。
> 另：catalog 的 `missing_character_portraits`（普查时 44，现 45）经逐项核对
> **上游本就无对应头像图**（多为 pot_* 等活动/PVE 武将），无可迁移内容，维持登记。

## 已迁移（npack + extensions 投放）

| 来源 | 目标包 | 交付 |
|---|---|---|
| `apps/core/character/*`（26 包） | `extensions/{bingshi…yingbian}` | 全量闭环（ABI-COORDINATION.md 汇总） |
| `apps/core/extension/玩点论杀` | `extensions/wandianlunsha` | ABI 608-616，9 武将 / 3 卡（B 段 Codex，ADR 0521-0529） |
| `apps/core/extension/英雄杀` | `extensions/yxs` | 43 武将 / 105 技能（C 段 omp，ADR 0530） |
| `apps/core/extension/欢乐卡牌` | `extensions/huanle` | 5 装备卡 / 5 技能（C 段 omp，ADR 0531） |
| `apps/core/extension/杀海拾遗` | `extensions/shahai` | 332 武将 / 1092 技能 / 41 装备卡（C 段 omp，ADR 0532） |
| `apps/core/extension/3D精选` | `extensions/ddd` | 31 名 `ddd_*` 武将全部闭环（B 段 Codex 批次一~二十三 + C 段 omp 收尾批次 0581，ABI 617-640） |

> 注：2026-08-03 全包一致性终验发现并修复 xianding 包 4 名完整武将（dc_shen_liubei/
> dc_sb_liuzhang/dongxu/zhangjian）的 5 个失效技能引用（ADR 0540，E 段 Hermes）；
> 重打包后 504 技能 / 196 武将不变，ABI 607，extensions 三份 npack 一致。diy 包
> 84 个未解析引用经核实为元数据登记设计（ADR 0512，README 声明），非缺陷。

| 来源 | 目标包 | 历史交付 |
|---|---|---|
| `apps/core/noname/library/`（主库技能） | `content/standard` + 各包重写 | 标准牌堆 119 张引擎原生，技能随包重写 |

## 非可迁移项（登记原因，不进入 npack）

### 1. `apps/core/extension/boss`（extension.js，单文件）
纯模式 JS：`game.import("play")` 注册 boss/versus 模式选将池配置（localStorage 驱动、
`lib.characterPack.boss` 注入与 AI 禁用配置）。无技能/角色/卡牌内容定义；模式机制
（boss/versus/tafang）在新引擎未实现。→ 不可迁移。

### 2. `apps/core/extension/cardpile`（extension.js，单文件）
纯模式配置 JS：`arenaReady` 覆盖牌堆构成（total/sha/huosha/… 花色点数计数）与
模式牌堆 UI。无技能内容；自定义牌堆构成不在内容模型内。→ 不可迁移。

### 3. `apps/core/extension/coin`（extension.js，单文件）
本地 UI JS：chess 模式金币商店（`ui.coin` 系统栏、商店弹窗、本地配置）。无技能
内容；纯表现层。→ 不可迁移。

### 4. `apps/core/card/{huodong,xianxia,yongjian,zhulu}.js`（模式卡组）
活动/仙侠/用简/逐鹿模式的专属卡组（自定义卡牌种类、模式门控 `mode: [...]`），
模式未实现。→ 不可迁移。
`apps/core/card/guozhan.js`（国战卡组）：**已随国战模式迁移**（2026-08-05，
A 段 Claude，ADR 0604）——31 卡牌索引 + 464 技能 DSL（gz_ 197 + 无下划线 99 +
复用 168）+ 256 武将 skills 全填充，引擎 `mode/guozhan.rs` 就绪。
`extra.js/sp.js/yingbian.js/standard.js` 卡组由既有 extra/sp/yingbian 包技能迁移
与标准牌堆覆盖，不重复登记。

### 5. `apps/core/character/{perfectPairs,rank,replace}.js`（客户端元数据）
`window.noname_character_*` 客户端辅助数据：选将配对、评分分级、同名武将替换表。
无规则语义，仅影响本地客户端展示/选将 UI；新引擎角色索引无对应字段。→ 不可迁移。

### 6. `apps/core/mode/*.js`（游戏模式）
identity/versus/stone/tafang/chess/boss/guozhan/doudizhu/brawl 等模式脚本
（合计约 1.9MB 模式 JS）。模式引擎不在本仓库内容管线范围。→ 不可迁移。

### 7. 表现层资源（`apps/core/theme|image|audio|font|layout|mode` 下的皮肤/图片/
音频/字体/布局）
全部为表现层资源；迁移原则「皮肤、CSS 与音频仅属表现层，不进入 npack 或运行时」
（架构计划多处记录）。→ 不可迁移。

## 角色索引补齐（2026-08-04，ADR 0582）

- 14 包 133 名缺失武将全部入角色索引（技能零未解析、零重复 ID），全局 2,896 唯一
  武将；standard 3 名（甘夫人/公孙瓒/旧界李典）因架构约束回退：引擎内嵌 standard
  由 standard_m1.skill 源编译 + characters.json 嵌入构成（standard_pack.rs），
  角色技能必须自包含于 m1 的 65 项定义；同 ID 镜像方案会触发 retain 去重与
  conformance 向量重钉（M1 变更），并发期不可行——3 将技能已由 sp/refresh 包迁移，
  仅角色条目不入内嵌索引（2026-08-04 复核确认）。

## 技能层面闭合审计（2026-08-04）

- 全量比对上游 26 包 character/skill.js + card/*/skill.js 顶层技能与全部 npack
  skill_index：**仅 1 项未迁移**——shenhua `guhuo`，为上游注释「技能错误，请勿引用」
  的废弃定义（现行于吉新蛊惑 `xinfu_guhuo` 已迁移，含 guhuo_guess/guhuo_phase
  质疑流程）；角色实际引用技能 100% 已迁移。
- 内置扩展技能覆盖：英雄杀 113 名全覆盖（含 6 项内部辅助登记 yxs_fanji2/
  tongyu_guiyin/jieyong3-6；oldbolehuiyan/2 为上游废弃定义不登记）；欢乐卡牌 5/5；
  杀海拾遗 1,093 名全覆盖（`_shengke`/`wuxingpan_skill` 属 play/wuxing 五行模式
  脚本，模式类不可迁移）；3D精选 64 名全覆盖（`ddd*_old` 4 项为上游废弃定义）。
- 全仓技能定义合计 7,332 项，分布 31 包。

## 全量门禁复跑（2026-08-04 14:30）

- engine-core 全量测试 88/88（11 个测试目标；修复并发引擎改动导致的原
  `original_xianding_zhupeilan` 陈旧结构字段缺失，测试文件补齐
  all_sources_were_hand/originating_view_as_skill_def/skill_excluded_targets），
  rule-vm/engine-api/content-schema/rule-api 19/19、content-compiler 531/531、
  conformance 8/8、31 包空 store 冷导入全部通过；§37.10 清理后工作区与共享
  target 均不存在。

## 全量工作区门禁（2026-08-04 17:55）

- `cargo test --workspace`：35 个测试二进制、746 项通过；仅存 1 项失败
  `original_bingshi_pot_caoshuang::package_publishes_complete_pot_caoshuang_at_abi568`
  ——断言 bingshi npack 为 33 将/96 技能/1 卡，而当前定稿 npack（并发 agent
  16:23 重打包）为 34 将/99 技能/0 卡（含 ABI 521 孙韶角色条目）。bingshi 为
  B 段活跃域，测试对齐由其批次负责，不代改；我方交付（yxs/huanle/shahai/ddd、
  角色索引补齐、real_extensions_lock 常驻门禁、conformance）全部通过。
- 新增常驻门禁 `engine-core/tests/real_extensions_lock.rs`：31 包全量安装 →
  锁加载 → 引擎开局。

## 内容修正批次（2026-08-04 18:30）

- 卡牌回归：bingshi【瞒天过海】（ABI 539 交付，cards.json 存在但 16:23 重打包漏传
  --cards）与 extra 长安大剑 6 件装备（cards.json 晚于 npack 构建）恢复，重建投放。
- 开局受损语义：77 个角色条目 hp<maxHp 被压平（跨 19 包），按上游 hp/maxHp 全量
  修正（含 initial_hp）；逆转之神等开局受损角色恢复 4/5 等语义。
- 十常侍：10 名 scs_* 角色上游 hp:0（未显形），引擎校验拒绝 max_hp==0，统一修正
  为 max_hp 1 / initial_hp 0 的最小合法表示。
- 工作区全量门禁：`cargo test --workspace` **67 二进制 / 866 项全部通过**（含并发
  agent 的 armor/tengjia/renwang 等新测试）；中途共享 CARGO_TARGET_DIR 缓存损坏
  （serde_json/parking_lot E0220），按 §37.10 直接删除目录重建后全绿。

## 并发重打包波次修复（2026-08-05 17:30）

- 并发批次（8-04 20:43–21:59）重打包留下坏中间态：diy ported npack 35 技能/0 将、
  clan 5/0、ddd/shenhua 双副本陈旧——全量审计后按 ported 源重建全部 30 个 ported
  npack 并恢复部署双副本（30/30 ported==deployed、31/31 双副本一致）。
- diy 技能 ID 漂移：并发批次重编号 nsqiyue 等技能 ID（3934300xxx → 新 ID），13 处
  角色悬空引用按上游技能名全量重映射；补齐系统技能 `dualside`（双面武将标记）元
  数据 stub（2750719950）；全 31 包**零悬空引用**。
- 门禁：cargo test --workspace **67 二进制 / 890 项通过**、conformance ALL PASS
  （含新增 guozhan 向量）、31 包冷导入通过；.npack-store 重建
  （content_lock bd8dd9e8…）。

## 清点结论

`archive/upstream/noname` 下全部内容已处理完毕：技能/角色/卡牌内容全部进入
extensions（26 包 + wandianlunsha + yxs + huanle + shahai + ddd 全 31 将闭环），
剩余项全部为模式/UI/配置/表现层，按上述原因登记不可迁移；旧 JavaScript 只作
语义证据，不进入 npack 或运行时。

## 静态资源迁移（2026-08-05，ADR 0610/0611，C 段 omp）

用户指示静态资源（图片/语音）也须迁入本项目：

- **角色立绘**（`apps/core/image/character/` 2,576 文件）：24 ported 包
  2,366 张 + standard 30/30 按 name_key 精确匹配挂接（`portrait_asset`），
  `std_` 前缀回退仅用于 standard；~180 张对应未迁移/废弃武将，留档。
- **音频**（`apps/core/audio/` 9,257 文件 / 609M）：skill 语音 6,343 份
  （按技能名，1,022 未命中=上游未迁移技能）、die 语音 1,486 份（按武将名，
  136 未命中）、card 2 份（按卡名；156 基础牌语音属引擎域登记不迁移）；
  voice/effect/background（全局/编号序）登记不迁移。
- 全部 31 资产包重打包部署，logic_hash SAME-HASH 核验；32 包空 store 冷导入
  （content_lock=ee263cfb…）+ `real_extensions_lock` 门禁 + conformance 8/8
  通过。
