# T12 swd 武将面批 82：福/肯迪

- 日期：2026-10-15
- 范围：两将转正（1 整将 + 1 partial）——**71/79 名**（上批 69 将+2）；新增 HostBehavior tag 371–372
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（yudun/zhanxing 逐段核验）

## 两将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd福 (qun hp5, 男) | 愚钝 yudun | **整将**：SwdYudun(tag 371)=锁定 marker 双面——①`engine.rs` PlayCard 门禁：锦囊类不可使用（上游 mod cardEnabled/Respondable/Savable 三面全禁；登记差异：原生仅使用位校验）②ActivateSkill 双段机：SelectCards 两张锦囊手牌（消费段校验）→ TargetChoice 攻击距离内 → 首张 use_as=Sha 声明进入杀结算、次张弃置（丈八蛇矛机同型）；`kernel.rs` 出杀计数豁免（use_as=Sha ∧ 实体非杀族 ⊃ yudun——上游 yudun_count stat.sha-- 同口径） |
| swd肯迪 (qun hp3, 男) | 占星 zhanxing（kbolan defer） | SwdZhanxing(tag 372)=ActivateSkill usable:1——SelectCards 弃任意张手牌 → 亮等量牌堆顶牌（入弃堆）：方块摸 2/红桃已受伤回 1 否则 +1 护甲/梅花各敌人（非友）随机弃 1/黑桃 TargetChoice 任一存活受 1 无来源雷伤 |

## 设计要点

- **锦囊判定复用 `is_basic_trick_or_equip`**（非基本 ∧ 非装备 ∧ 三类——地/机关异类天然排除，rvm_bridge kind_is_ordinary_trick 同源口径）。
- **出杀计数豁免按实体牌种区分**：PlayCard 常态出杀也带 use_as=Sha 声明位，须以「实体非杀族」为合取段（批 15 良策转化的「声明性 use_as 非转化专属」勘误同口径）——首版仅查 use_as 致普通杀误豁免，测试断言抓漏。
- **敌对关系=非友**（`is_friend_of` 取反；上游 getEnemies 同口径）。
- 登记差异：愚钝打出/响应面未落（仅使用位校验，往事封禁同口径）；亮牌面按实有张数（洗弃堆补底）。

## 验证

- **定向回归** `swd_generals_b82_yudun_zhanxing`：
  - 愚钝反例：无中生有直接使用被拒（CardLimit）；正例：普通杀后用双锦囊（无中+顺手）当杀命中 p1（4→2，两段伤害）。
  - 占星正例：弃 1 张亮黑桃 → 目标窗选 p1 受 1 雷伤（4→3）；反例：无手牌 ActivateSkill 拒。
- **包装载**：71 将入册（cards.json 69→71）。
- **全量**：engine-native **794 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。

## 结论

- **71/79 名**（福为整将，肯迪 1/2 技 partial）。
- 四项用户决策维持待用户裁决清单登记。
