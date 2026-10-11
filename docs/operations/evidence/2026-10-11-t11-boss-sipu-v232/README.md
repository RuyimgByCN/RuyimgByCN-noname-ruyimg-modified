# 2026-10-11 T11 应昭批末片：司圃死缚使用面（规则 v232）＋测试注册缺陷修复

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `88ce4dec`；应昭批末片最小切片——司圃(+switch)，四象系首片 boss_shenyi 登记下批）。上游依据：`mode/boss.js` `boss_sipu` 死缚（四面 mod `cardEnabled/cardUsable/cardRespondable/cardSavable`——应昭出牌阶段已用牌数≤2 期间其他角色无法使用/打出/救援）。

## 建模

- **司圃**（权限型 Noop+查询，`BOSS_SIPU` 常量；应昭条目四技全齐 `[妖兽, 封冻, 巡游, 司圃]`）：
  - **激活门控**：commit_damage 尾第九并列臂（虚失/炎狱先例同构），hp≤max/2 一次伤害激活（SkillToggled）；apply 装配即未激活特判扩至 SIPU。
  - **响应禁（cardRespondable）**：respond_shan 选项排除——杀来源=enabled 持有者且当前回合=持有者且已用<3 → 闪选项仅留 Pass（目标必受伤）。
  - **救禁（cardSavable）**：dying_save 选项排除——濒死者=enabled 持有者且当前回合=持有者且已用<3 → 他人救援桃/酒不入选项；持有者自救可用（上游禁令对象=其他角色，持有者不受限）。
  - **使用面（cardEnabled/cardUsable）**：原生回合模型下仅回合玩家可提交 PlayCard——「其他角色无法使用牌」由回合模型天然承载，无需独立校验臂（首版 PlayCard 臂已删：该面在原生模型不可达）。
- FROZEN 名单尾加「司圃」（85 名）。

## 登记差异

- 无懈等其余响应窗未纳入司圃禁令（上游 cardRespondable 覆盖全部响应窗；原生仅建模 respond_shan 与 dying_save 两窗）。
- 上游「已用≤2 期间全场禁出」对使用面的作用面（他人出牌）在原生回合模型不可达——承载方式为模型结构，非逐条规则实现。

## ⚠️ 测试注册缺陷修复（本批横切发现，影响历批计数口径）

修复 sipu 回归时发现 `native_kernel.rs` 存在**属性堆叠窃取**：历批插入脚本把「批次注释+#[test]」反复叠到既有测试 fn 上方，真 fn 的 `#[test]` 被挤走。修复前状态：

- 4 个 fn 多重注册：`boss_taotie_tanyu_*` ×9、`boss_shujing_cuidui_*` ×4、`boss_yingzhao_sipu_*` ×10、`doudizhu_binglin_*` ×2（同一 fn 以同名注册多次，编译不报错、静默重复执行）。
- **20 个历批测试 fn 零属性、从未进入测试二进制**：v208–v231 各批的 `boss_zhuyin_*`/`boss_hundun_*`×2/`boss_qiongqi_*`×3/`boss_futai2_*`/`boss_yandu_*`/`boss_taowu_*`×3/`boss_xiangliu_duqu_*`/`boss_zhuyan_*`×3/`boss_bifang_zhaohuo`/`boss_bifang_honglianx`/`boss_bifang_yanyu`/`boss_yingzhao_fengdong`/`boss_yingzhao_xunyou`/`doudizhu_mode_victory_*`。各批「定向回归通过」记录中，这部分测试在**其后批次的全量/定向运行里实际缺席**。
- 修复：21 份批次注释逐一归还真 fn、属性去重至唯一 `#[test]`（机械审计脚本核验：全文件无多重注册、boss_*/doudizhu_* 零属性清零；注册数 234→233）。
- 连带修复 5 处失修（这些测试自失修起从未运行过）：朱厌三测试 def 偏移 +8→+9、相柳 duqu 测试内书精 +7→+6 / 相柳 +6→+7（v227 毕方批表序插梼杌时仅同步了已注册测试）；招祸测试伤害期望 3→2（漏算凶兽条件伤：目标体力小于来源时杀伤 +1，火属性不加值的断言口径不变）。
- **登记**：修复前注册表含重复注册与缺席并存，历史批次「578/0/1」等计数口径不可比（既虚高重复又漏计缺席）；本批起以机械审计为准。20 个复位测试修复后首次实跑**全绿**（其中 5 个经上述修正后绿）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_yingzhao_sipu_use_ban_first_two` | 五段：①已用<3 期间杀不可闪（选项仅 Pass）②③已用计数 2→3 ④第 3 张后解除、濒死自救桃入选项 ⑤应昭自己回合内濒死：自救可用、他人救援桃被禁、无人救→死亡；全程 `assert_zone_conservation` |
| 既有回归复位 | 20 个历批测试首次以真身实跑全绿（5 个经索引/期望修正）；boss 全滤 28/28 |
| 14 golden | 版本回退探针（232→231 复跑 **全绿**）证明位移唯一来源=版本字段，随后 232 全量重录 2/2 绿 |
| 全量 | **578 通过 / 0 失败 / 1 ignore**（2026-10-11 本机，`cargo test --offline -p engine-native`；native_kernel 233 全部唯一注册） |
| Gate 1b | `RUSTFLAGS="-D warnings" cargo check -p engine-native --lib` exit 0（独立 CARGO_TARGET_DIR，用后即删） |
| 版本 | 规则 231→**232**；存档不变 v53（本批无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互（0 号回合循环）独立待查（Prototype 局验证无碍）。
- 炎狱跨回合计数器精确口径待查。
- 四象系首片（boss_shenyi 神裔共有技[始终正面+判定反转]+青龙条目）下批。
