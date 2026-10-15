# T12 swd 武将面批㉚：妮可/韦达/淳于衡

- 日期：2026-10-15
- 范围：三将 partial 转正——**42/79 名**（上批 39 将+3）；新增 HostBehavior tag 343-344
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（huanjian/xuying/jueqing 逐段核验）

## 三将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd妮可 (qun hp3, 女) | 幻剑 huanjian（灵武 insertPhase=引擎缺口 defer/明镜外部依赖 defer） | SwdHuanjian(tag 343)=marker：viewAs 链臂黑牌→冰魄针（JiguanBingpoCard 目标+邻座各弃机器复用） |
| swd韦达 (qun hp3, 女) | 渔人 yueren（**def 复用**——喀玛同技同 mapping；贞烈 NOT FOUND/对决 limited defer） | 零新行为零代码——cards.json 条目映射即生效 |
| swd淳于衡 (wei hp2, 男) | 绝情 jueqing（**builtin 白名单 8→9**——绝情在冻结键表、标准目录 JUE_QING 原生）+虚影 xuying（笙笙 NOT FOUND defer） | SwdXuying(tag 344)：DamageBefore 帧共用 Damage 载荷——prevented=true 写回+手牌非空 LoseHp 1（Boss 绝情同窗；**空手=完全免伤**上游语义） |

## 设计要点

- **虚影载荷勘误**：DamageBefore 帧共用 `EventData::Damage` 载荷（非独立变体）；取消写回字段=`prevented`（非 cancelled——DamageData 无 cancelled 字段，prevented 即提交段跳过扣血语义）。
- 韦达证明「同技跨将复用」零成本：cards.json 映射 host:yueren 即完成整条链。

## 验证

- **定向回归** `swd_generals_b70_huanjian_weida_xuying`：幻剑黑牌→冰魄针弃牌路由；渔人复用杀结算；虚影有手=取消+失去 1（3→2）+手牌保留、空手=完全免伤（3 不变）。
- **全量**：engine-native 782 passed / 0 failed（无哈希面改动——golden 零扰动；lib.rs 冻结键数 341 漏更由 a0 测试拦截后补正）。
- **Gate 1b**：exit 0（陈旧布阵块清理后复跑绿）。

## 结论

- **42/79 名**（upstream 79 将 183 技）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
