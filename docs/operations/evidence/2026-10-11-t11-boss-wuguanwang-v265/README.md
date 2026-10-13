# 2026-10-11 T11 十殿阎罗五官王片：枳枉/公正/血池（规则 v265）＋五官王条目（十殿阎罗首片）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续；十殿阎罗族首片，族首勘见[十殿阎罗首勘](../../archive/)语境——13 将清点+换任族级机制专项标记见主稿）。上游依据：`boss_wuguanwang`（吕|岱/qun/hp4/skills `[boss_zhiwang, boss_gongzheng, boss_xuechi]`，boss.js:1326）；`boss_zhiwang`@3630 / `boss_gongzheng`@3652 / `boss_xuechi`@3663。

## 建模

- **枳枉**（查询型 Noop，`BOSS_ZHIWANG`）：**CardsMoved commit 显式臂**（原默认臂注释「区移动作已完成」处加处理）——其他角色（to≠持有者）获得牌结算后，其随机弃置一张手牌（随机→首张确定性近似登记）。摸牌阶段摸牌走 `CardsDrawn` 事件天然隔离=上游排除 draw 同口径；planetarian 孙策化彩蛋分支不落登记。
- **公正**（查询型 Noop，`BOSS_GONGZHENG`）：Judge begin 臂——判定区有牌则弃置首张（上游 phaseZhunbeiBegin 判定区 randomGet 弃置→首张确定性近似登记；鬼计同款臂，准备阶段原生锚=判定阶段开始）。
- **血池**（查询型 Noop，`BOSS_XUECHI`）：End begin 臂——座位序首个其他存活角色失去 2 点体力（上游 randomGet→首个确定性近似登记）。
- **五官王条目**：`BOSS_GENERALS` 尾追加（def 索引 43，hp4，male）`[枳枉, 公正, 血池]`；上游 group **qun→群目录直取**（首个非 shen 群雄乱入条目）。

## 调试 API 新增

`debug_give_hand_card(from, to, card)`：真实 `CardsMoved` 事件面的测试布阵入口（state 克隆+push_frame 走完整结算）——枳枉类 gain 触发技能测试必需（wugu 机器直接 hand.push 不发事件；debug_place 手牌回收分支只查不删=甄姬批已知重影坑）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_wuguanwang_zhiwang_gongzheng_xuechi` | ①枳枉正例：debug_give 真实 CardsMoved → 2 号获牌后弃原手牌（手牌收支断言）②公正：闪电置判定区（无懈窗放行）→ 穿过整回合循环到判定阶段开始弃首张（闪电未及判定）③血池：End 首个其他存活（0 号）失 2 ④往返 |
| 版本递增探针 | 普通批口径：265→264 回退复跑 golden **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 行级程序化重录 2/2 绿 |
| 全量 | **622 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 264→**265**；存档不变 v61 |

## 坑位实录

- **循环轮数按回合跨度估算**：公正验证需从 1 号 End 穿过 0/2/3 号整回合到 1 号 Judge（10 轮不足，18 轮）。
- **闪电置判定区开无懈窗**：置入延时锦囊使用时开窗（Pending）——需 `pass_nullify_windows` 放行后才入判定区。
- holders.first() 解引用：`to` 为拷贝 u8 不加 `*`（E0614）。

## 遗留登记（本批新增）

- 枳枉：planetarian 彩蛋分支不落；CardsMoved 面覆盖 give 类获得，wugu 机器直接 hand.push 不发事件（表面差异登记——上游 wugu gain 会触发枳枉，原生不触发）；随机弃→首张。
- 公正/血池：随机→首张/首个确定性近似。
- 下批候选：宋帝王（黑绳/胜负/恩怨）、卞城王（雷舟/雷斧/雷诛）、泰山王（复读/苦久/热恼）、转轮王（轮回/往生/反噬+魔道共用）——勘验均已完备。
