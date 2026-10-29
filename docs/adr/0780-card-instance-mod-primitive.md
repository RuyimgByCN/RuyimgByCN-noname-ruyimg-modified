# ADR 0780：卡牌实例受控合成原语（CardMod 伴随表）

日期：2026-10-11　|　状态：已采纳
关联：[ADR 0777/0778](0777-native-c0-native-loader.md)、[T18](../architecture/native-progress.md)、`crates/engine-native-kernel/src/state.rs`

## 背景

上游 swd 机关术（`lingjianduanzao`）在运行时按名字组合合成/改造卡牌实体：
`lib.card[name] = copy(equip)` + 数值升级（getValue 封顶 10）+ `card.init`
原位变形。原生目录（`Catalog`）为共享只读结构，名称经符号表 `'static`
分配（0777），卡牌实体仅为目录位索引（`CardId`）——**运行时修改目录定义
不可行**（共享目录跨局复用、unbounded 增长、破坏名称纪律）。

## 决策

**受控合成 = 伴随表建模**：`GameStateCore` 级（挂 `PlayerState`）新增
`card_mods: BTreeMap<u32, u32>`（装备实体 → 附着灵剑实体）。合成不改变
装备实体的目录定义与归属，仅登记「该卡已附着哪张灵剑」。

- **登记差异**：①名称合成/数值升级仅入改造表与日志（目录不可变，点数升
  级未接入任何消费点——当前无原生机制读取合成后数值）；②jiqi/yuchan 池
  暂缓（机关术全部附着灵剑）；③改造不随卡牌转移（换装/获得离去即失效，
  表随拥有者存档）。
- **存档**：`card_mods` 以 len+(u32,u32) 序列入 `PlayerState` 尾部，存档
  格式 44→**45**（v44 字节被版本门禁拒绝，无生产迁移数据）。
- **日志**：不新增 LogEvent 变体（演示呈现暂缓；状态表即可观测）。

### 备选方案（弃）

1. **运行时目录克隆**（copy def + intern 新名）：破坏名称符号表纪律、跨
   局 unbounded、共享目录并发不可行。
2. **完整卡牌实体合成**（脱离目录的新实体）：同上且实体生命周期（回收/
   守恒校验）无解。
3. **不建模**：机关术无可观测面，切片无意义。

## 验收

`g1_shahai_swd.rs::swd_jiguanshu_attaches_card_mods_and_roundtrips`：施放
后 CardMod 恰一条且附着实体为灵剑；CardMod 随存档字节往返一致；无装备不
可用。
