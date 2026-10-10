# U07 收尾：duoqi/youzhan 正式暂缓 + ow/xianjian 清点（2026-10-08）

## duoqi/youzhan 正式暂缓口径（包内登记落位）

- **duoqi（夺启，刘馥）**：依赖①筑城标记存储（zhucheng 标记为按钮池）
  ②`phaseUse` 中止（`evt.skipped`）——原生出牌阶段推进无中止通道。
  口径：待 phaseUse 中止原语建模后随筑城标记联做。
- **youzhan（诱战，田豫）**：trigger global shaBefore——原生映射为
  CardUsed AfterTriggers 触发（语义偏晚：上游在杀结算前）；content 令
  杀目标虚拟 useCard【诱敌深入】（诱敌深入实体/宿主原语 41 已在，但
  「令目标主动使用」+ 响应窗口组合留待 E03 使用前窗口体系扩展）。口
  径：暂缓。
- **落位**：wudian 包 cards.json——generals[].skills reason 更新 +
  skills[] 补 duoqi/youzhan 暂缓条目（mapping:null+reason）；npack 三
  哈希重算。台账 U07 行同步（未映射 2 = duoqi/youzhan）。

## ow/xianjian 武将子目录清点评估

- **ow（轩辕剑外传）**：24 武将 / 107 技能定义（含 woliu2/pingzhang2
  等衍生子技）/ 无 mode 限定。
- **xianjian（仙剑）**：36 武将 / 128 技能定义 / 无 mode 限定。
- **评估**：character/ 管线（武将元数据+装配技能）已有第三例（3D 精
  选）与玩点论杀（U07 generals mapping 通道）先例——元数据入包可复用
  `general_meta` 通道；但 235 条技能定义多为多步/标记/选择型，G1 量级
  为「特大」（与 U05 台账口径一致），首切片应选「单武将 1-2 技能」纵
  切（如 ow_liekong 山现/闪回）而非整包。**本轮仅清点不转换**（无码
  改），台账 U05 行尾注登记。

## 验收口径

- 全量 engine-native **435 passed / 0 failed / 1 ignored** 不变（纯登
  记+包元数据）；依赖包 63/0 不变；golden 不变（wudian 包哈希入
  content lock，不触标准场景）。
- **剩余**：U07 duoqi/youzhan（正式暂缓）；zhenfa 五种阵法；ow/
  xianjian 纵切首切片（待排）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（6 文件，相对 noname-next 根）。
