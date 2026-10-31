# C9c：部分竞争／恢复修复与12场语义处置，仍 BLOCK

核验2026-10-31，私有331/106／ABI758／WS6，base5fc+C8精确170叠加+C9b。21增量manifest dca881deb88eefc63d7836bb0bff5fd3962b18d54352c5f20c580c95215b8379；214组合0ab2b44eac80e7c1d591ebe560e8be69ab4ff72a4ac35b7db92edc5c4f4925ae。见[实现原报告](implementation.raw.txt)、[父完整性核验](noname-parent-c9c-final-audit.json)、[复制／私下保留映射](archive-map.json)。Main未合入C生产代码。

三个真实不可变SkillDef（Jueqing／Wuxin／Zuijiu）通过原Frame.queue／TriggerOrder／C7 admission竞争，原inline副本移除；有限内部NativeDamageSkill tag85连通codec／hash／restore／A0 inventory。窗口恢复的Intrinsic／BeforeTriggers及缺order伪造反例拒收，实际byte／fullsave continuation通过；damage日志先于该受害者changeHp解链，新增原红和修复保留。

[独立源码复审](review-ordering-P1.raw.txt)确认上述修复，但仍BLOCK一个具体P1：Huoshen等inline source skill抢在First handler之前；Jidong／Chengxin状态、Begin与Begin1／4、End／After源回调须用真实definition／provider与owner角色进入同一竞争，不能把Boolean／equipment存储当作intrinsic证明。共享DeathFlow／provider／LossEnd完整后续未实现。

最终full native(kernel/content/facade --no-fail-fast)1796/1/75、exit101，仅golden组12 mismatch；独立`-Dwarnings`all-targets0、定向24/0、unit2／source9032／layering0。编码器初始1001有效行门禁红及按damage codec职责拆分后的绿保留，未改golden expected。原始错误／警告／source绑定在original目录。

实际C8 baseline=gitarchive5fc+f5829f…精确170 overlay，原14场group先绿、只观察命令seam后的instrumented group再绿。与最终当前源12场362对capture比较完整Core、event、RNG、各seat view／choice和referee，无未知规范化。11场仅在已观察命令／终态业务面相同；chain_fire_propagation于0007-after出现真实顺序差异：旧提前Linked2=false，新先受害者DamageDealt再其自身Linked(false)，源content.ts12092–12120及Liuli game.js34767–34810支持新行为。不能从完整存档／调度budget／frame差异推出任意内部瞬态或fuel路径等价。

机器处置及每份JSON／snapshot／archive原SHA见[12场表](original/golden-business-dispositions.json)。复审指出outcome_raw虽保留但比较脚本未逐项比较；后续需独立核返回判别／错误原因（仅accepted state_hash是争议golden），再刷新变动源码的必要current captures。**剩余排序修复、返回语义核对及独立审查前不重录golden**。

父fresh核5215原产物、214／21源码tar及362capture对，只证明完整性；本目录收最终源码、命令／错误日志和全部362对最终capture所指文件。约3.215GB的5fc原gitarchive及中间重复／失败trace原件仍私下保留且由原索引标识，未删除、未冒称全数发布。剩余排序修复由同一私有writer继续，当前状态见[进度主稿](../../../architecture/native-progress.md)。
