# U05 xianjian 第三十批切片（规则 v172，2026-10-09）

南宫煌照妖（zhaoyao）单技闭环——南宫煌升全闭环（仅 zhangmu 响应链暂
缓）。前置：
[适盈死亡取消型](../2026-10-09-xianjian-shiying-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js zhaoyao）

- `zhaoyao`：global phaseDrawBegin——他人摸牌阶段开始、双方有手牌 →
  chooseToCompare 拼点 → 赢 → draw 2 + 选两张 he 置牌堆顶（先选在
  上）。

## 改动（分开陈述）

- **宿主行为**：`XianjianZhaoyao`（tag 193，PhaseBegin 确认门 direct：
  **确定性首张拼点**（舌战群儒同款登记差异——双方 hand.first 比点，
  无选牌询问）→ 赢→摸 2+SelectCards 选两张手牌（必选）→离手压牌堆顶
  （先选在上=后选的先压栈）→ 输/平无事）——六表+ALL 192→193。无新
  状态字段。
- **登记差异**：①确定性首张拼点（无双方选牌询问）②**原生触发在固定
  摸牌前**，原生线 PhaseBegin AfterTriggers 在固定摸牌后=摸后拼点（双
  方手牌各含新摸 2 张）。
- **cards.json**：南宫煌 zhaoyao mapping null→host + rehash 两步；**版
  本** 171→172；a0 追加 193。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`nangonghuang_zhaoyao_compare_and_
  top`（布阵保证首张点数必赢→Draw-begin 比点→摸 2+置顶询问→先选者
  落牌堆顶+手牌净 0）。
- **全量通过**：engine-native **503 passed / 0 failed / 1 ignored**
  （v171 基线 502 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 171→172 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。

## 剩余

- 南宫煌 zhangmu（响应链）单技暂缓；xianjian 其余 10/36 将（按缺口
  族）；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
