# ADR 0625：国战锦囊「知己知彼」（zhibi）——维持元数据登记

> 承接 [ADR 0624](0624-guozhan-chiling-trick-settlement.md)：国战 12 锦囊最后一张——
> zhibi（知己知彼）。与 [ADR 0621](0621-guozhan-yuanjiao-map-existing.md) 对 zhibi 的
> 初判一致：**纯信息揭示，确定性规则引擎不可表达，维持元数据登记**。至此国战 12 锦囊
> 盘点闭环：11 张真实 CardKind 结算 + zhibi 元数据登记。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

guozhan.js:1313 `zhibi`（梅花 3 / 梅花 4，牌堆 2 张）：
- `filterTarget`：其他有手牌或暗置武将的角色；`recastable: true`（可重铸）。
- `content`：将目标记入 `player.storage.zhibi`（持久列表，供特定角色技能消费）→
  使用者选择观看 手牌 / 主将 / 副将（仅列目标确实拥有的项）→ **对话框展示**信息。
- 全程**无任何牌堆/体力/牌区状态变更**——仅信息揭示与一次持久记录。

## 为什么不可表达

1. **信息揭示**：确定性规则引擎的状态对局外可见；「观看」不改变可哈希状态。
   「观看手牌/暗置武将」在引擎中无状态落点（ADR 0621 同判）。
2. **`storage.zhibi` 记录**：上游持久于 `player.storage`，供 gwent/角色技能消费；
   引擎无通用 per-player storage 语义，且该记录的消费者未随国战内容迁移。
3. **`recastable`（重铸）**：引擎当前无重铸原语（弃置重铸摸一）；若后续加
   重铸原语，zhibi 可先以「可重铸」参与结算。

## 元数据登记（维持现状）

- `content/deck.rs`：`("zhibi", Suit::Club, 3)` 与 `("zhibi", Suit::Club, 4)` 保持
  `CardKind::Other`（不可作为锦囊打出），牌堆/花色/点数已登记。
- 不新增 CardKind / trick_effect / EffectRequest；`RULE_VM_ABI_VERSION` 不变。

## 国战 12 锦囊盘点（闭环）

| 锦囊 | CardKind | 状态 |
|---|---|---|
| yiyi 以逸待劳 | 43 | ADR 0614 ✓ |
| shuiyanqijunx 水淹七军 | 44 | ADR 0615 ✓ |
| lulitongxin 戮力同心 | 45 | ADR 0616 ✓ |
| kefuzhongyuan 克复中原 | 46 | ADR 0617 ✓ |
| guguoanbang 固国安邦 | 47 | ADR 0618 ✓ |
| huoshaolianying 火烧连营 | 48 | ADR 0619 ✓ |
| lianjunshengyan 联军盛宴 | 49 | ADR 0620 ✓ |
| yuanjiao 远交近攻 | 37 映射 | ADR 0621 ✓ |
| wenheluanwu 文和乱武 | 50 | ADR 0622 ✓ |
| haolingtianxia 号令天下 | 51 | ADR 0623 ✓ |
| chiling 敕令 | 52 | ADR 0624 ✓ |
| **zhibi 知己知彼** | Other | **本 ADR：元数据登记** |

## 验证

| 检查 | 结果 |
|---|---|
| 引擎改动 | 无（纯文档 ADR） |
| conformance / workspace | 基线不变（金标已于 ADR 0624 重钉） |

## 后续

1. **gwyewu（叶舞）造卡入口**：需随机弃牌原语（ADR 0613 已登记；本 ADR 不再重复）。
2. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
3. 若引擎未来加入「重铸」原语，zhibi 可升级为可重铸卡片参与结算。
