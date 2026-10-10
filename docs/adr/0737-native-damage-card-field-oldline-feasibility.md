# ADR 0737：DamageData 携牌字段与 N12 旧线对照可行性评估

日期：2026-09-16
状态：已实施（DamageData）/ 已评估（N12 旧线对照）
关联：[0736](0736-native-simayi-guicai-fankui.md)、[0721](0721-native-runner-baseline.md)、进度文档 N08/N12

## 背景与决定

### 1. DamageData 携牌字段（官方迁移前置）

郭嘉遗计、曹操奸雄等官方技能需要知道「伤害由哪张牌造成」。`DamageData`
增加 `card: Option<CardId>`：

- `damage_data_with` 增加第五参数；`damage_data` 便捷构造为 None；
- 各伤害来源透传：杀（`Sha.card`）、决斗/南蛮/万箭（锦囊 `self.card`，
  AoeTarget 机器新增 `card` 字段透传）、铁索传导（继承原伤害 `d.card`）；
  无牌伤害（闪电雷击、调试注入）为 `None`；
- 宿主日志 `DamageDealt` 同步增加 `card` 字段（日志不入摘要，无哈希影响）；
- 编码变化 → `STANDARD_RULESET_VERSION` 9 → 10，14 golden 重录并通过。

验收：`sha_damage_dealt_carries_card`（杀伤害日志 card=该杀）与
`lightning_damage_has_no_card`（雷击 card=None）。

### 2. N12 旧线等价负载对照——可行性评估（不启动实施）

调查结论（2026-09-16）：

- 旧线位置：`archive/Noname-ruyimg-Modified`（pnpm/tsx TS monorepo，
  packages：extension/fs/jit/server，面向客户端扩展包构建）；
- 工具链：node v26.4.0 / npm 12.0.2 可用；
- **依赖未安装**（node_modules 不存在）——构建旧线需要一次 pnpm/npm 安装
  （工作区环境变更，体积大），本批不执行；
- 旧线无独立无头引擎 CLI：等价负载需要在旧线 jit/规则引擎上重写
  n09 的三负载驱动器（FFA/身份/判定）；
- **等价性边界**：新旧线 RNG 语义不同（新线 RuleRng 种子流 vs 旧线随机），
  哈希级对照不可能；可行的是**聚合统计对照**（命令数/事件数/逐区守恒/
  延迟分位），且必须先统一负载脚本与统计口径。

结论：旧线对照需要专门的「安装 + 无头驱动器开发」批次，属于跨工作区
环境变更，不宜与内核批次混合。已完成可行性调查并把上述边界登记为
N12 的实施前提。

## 影响

- 测试 167 项（native_kernel 90），净增 2。规则版本 9 → 10，14 golden 重录。
- N08 后续迁移路径：遗计/奸雄现在有了 `DamageData.card` 前置，
  可在后续批次按司马懿模式复制（目录注册 + 机器 + 存档 + 验收）。
