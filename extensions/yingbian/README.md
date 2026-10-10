# yingbian（应变，晋势力）

## 已发布
- 全 23 名角色登记（npack ABI 567 门控）：成济成倅/王祥/宣公主/辛敞/杨芷/杨艳/ol华歆/钟繇/卫瓘/车胄/司马周/ol李肃/晋杨艳/石苞/晋张春华/晋司马懿/晋王元姬/晋司马昭/晋夏侯徽/晋司马师/张虎乐綝/杜预/晋贾充。
- 完整武将（真实运行时）：
  - `xiongshu`（雄术，晋贾充）/`jianhui`（奸回，晋贾充）：ABI 437 既有。
  - `caiyuan`（才媛，晋杨艳）：结束阶段摸两张牌。
  - `chongxin`（崇信，石苞）：出牌阶段限一次重铸一张牌（目标侧重铸以近似省略）。
  - `xinwanyi`（新婉仪，杨芷）：成为牌目标后获得来源一张手牌置入技能牌堆（ABI 567 既有原语）。
  - `qiangshou`（强守，车胄）：有宝物时计算至其他角色的距离 -1。
  - `ybzhuiji`（追击，宣公主）：出牌阶段选择摸二后阶段末失去体力，或回复后阶段末弃两张牌。
- 其余技能为元数据登记 + 文档化近似（显示武将牌机制、应变牌、使命技、冰属性伤害、动态技能获得、跨回合标记等），详见 yingbian_meta.skill 各技能注释。

## 可表达性缺口（未移植机制）
- 显示武将牌（showCharacter）系技能（buchen/taoyin/tuishi/huirong/gaoling/shiren/baoqie/xuanmu）
- 应变牌概念（bingxin/xianwan/xuanbei/maihuo）
- 使命技/起定技状态机（ruilve/chengwu/olchuming）
- 冰属性伤害（qingleng）、动态技能获得（bolan）
- 声明牌名（caozhao）、多段拼点链（sanchen/yanxi/shiduo/huice 类）
