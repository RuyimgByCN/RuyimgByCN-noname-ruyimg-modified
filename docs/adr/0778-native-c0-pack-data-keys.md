# ADR 0778：原生扩展包数据键表与自描述装载（C0 续）

日期：2026-09-24　|　状态：已采纳
关联：[ADR 0777](0777-native-c0-native-loader.md)（本 ADR 取代其两处，见文末）、[演进提案 A0/C0](../architecture/architecture-evolution-plan.md)、[台账 E02/G1](../migration/upstream-extension-reconstruction.md)、[审查修复证据](../operations/evidence/2026-09-24-review-fixes/README.md)

## 背景

ADR 0777 的装载器通过 `PackSpec` 接收逐包映射：牌种映射表、宿主行为键表、
牌存在性派生技能、武器射程、装备技、包名/版本/牌名前缀，均由调用方（测试）
用 Rust 写出。审查发现：

- 牌种与数据键的对应没有引擎侧唯一定义，同一牌种在不同包里写不同键
  （炉石 `leishajian` → `LeiSha`）。
- 包元数据、射程与装备技不在包内，只拿到包数据得不到同一个 `ExtensionPack`，
  台账 E02 要求的"转换产物独立发布、无需重编译"做不到。
- 古剑 18 个食物/装备伴随技能、gwent 5 个状态技能的名称与行为本是引擎固定
  语义，却要逐包声明。
- 武将技能名单仍经 `Box::leak` 分配；缺省势力/性别静默取群/女，点数与体力无界。

## 决策

1. **引擎提供全局数据键表（宿主 ABI，只增不改）**
   - `CardKind::key()` / `from_key()` / `ALL`：86 个牌种各有唯一键，`ALL` 按存档
     编码序排列。包内牌名 `name` 可自取，`kind` 必须写本表的键。
   - `HostBehavior::key()` / `from_key()` / `ALL`：74 个行为各有唯一键（`host:`
     前缀，已发布的数据键保持原值），`ALL` 按 tag 序排列。
   - `InitHook::key()` / `from_key()`。
   - `card_kind_host_skills(kind)` / `card_kind_equip_skills(kind)`：牌种伴随技能
     （古剑 18、gwent 5）与固有装备技由引擎登记，技能名全局唯一。
   - [A0 测试](../../crates/engine-native/tests/a0_tag_registry.rs)锁定键双射、键唯一与
     伴随技能表一致性；键函数为穷尽 match，新增变体漏写键即编译失败。
2. **包数据自描述**：`cards.json` 新增必需的 `pack` 段（`name`、`version`、
   `description`、`card_prefix`、可选 `init_hook`）；牌可带 `range`（仅武器栏牌种，
   建模为通用武器 `CardDef::weapon`）与 `equip_skills`。装载器入口改为
   `bridge_pack(json)` / `bridge_generals(json)`，不接收逐包参数；`PackSpec`、
   `PresenceSkill` 删除。
3. **装载次序是确定性规则**：宿主技能先按牌首次出现登记伴随技能（去重），再接
   `skills` 数组中有 `mapping` 的条目。该次序决定目录技能编号（`SkillDefId`），
   改变次序即改变编号。
4. **坏包一律报错，不取默认值**：点数 1–13、体力 1–255、`pack.version` ≥ 1；势力、
   性别必填且取值受限；`host_api` 须为整数；`mapping` 非字符串报错；包内宿主技能
   重名（含与伴随技能重名）报 `DuplicateSkill`；装备技未在本包登记报
   `UnknownEquipSkill`；同名武将技能映射到不同行为报 `DuplicateSkill`。
5. **名单进符号表**：`names::intern_list` 为武将技能名单提供共享存储，重复装载
   返回同一切片。

## 兼容与版本

- 目录字节、canonical_hash 与存档编码不变：牌名仍为 `{card_prefix}{name}_{序号}`；
  古剑伴随技能的首次出现序与迁移前 `PackSpec` 声明序一致；玩点论杀三尖仍为
  `Weapon`、射程 3、装备技 `wd_sanjian`。规则版本 60、存档格式 v2 均不递增，
  golden 未变。
- 包数据：玩点论杀、古剑两包的 `cards.json` 增加 `pack` 段（玩点论杀另加射程与
  装备技），`source_hash`/`asset_hash` 随之更新；`logic_hash` 与内容锁（名称:版本）
  不变。
- 其余 6 份测试 bridge 暂不受影响。迁移时各包补 `pack` 段并改用规范键：炉石
  `leishajian` → `leisha`；`shenenshu` 现临时映射摸牌语义，写 `jihuocard` 并保留
  登记差异。

## 后果

- 新增牌种或宿主行为须同时给出数据键（编译期强制）；键发布后不改名。
- 装载器与测试不再含逐包映射代码，包数据是唯一事实来源。
- `interned_count` 只观测符号表；验收另以指针同一性断言防止名称/名单绕过符号表
  分配（先报红实证见审查修复证据）。
- 取代 ADR 0777 的两处：「装载器边界」中以 `PackSpec` 为输入；「后果」中"武将
  技能名单切片为一次性分配"的取舍。ADR 0777 的择址决策（宿主层独立 crate）不变。
