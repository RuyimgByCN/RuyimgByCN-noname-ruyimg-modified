# R05-I 原字节锁链工厂——派发规格（2026-11-04 定稿，待 H 落地后派发）

合同依据：proposal-v3 §7（持久增量清单）＋§9 场景11。既有基建：crates/engine-native/tests/boundary_snapshot.rs
（命令信任边界＋挂起机器快照/恢复骨架，C2 末 71/71 绿）。

## 工厂要做的锁链
D/R/G 与三 compound 的**每个等待点**：
1. D 窗口序各段（Before/Begin/Begin1/Begin2/End/After）、R 逐张 held 挂起、G Filter/RecordHistory/Transfer/After/Aborted、
   双枯竭→detached、跳过路径（outcome=Skipped）。
2. 三 compound 每个询问点（tiqi 确认/符号两问、baoshu 成本 chooseTarget、dcbihuo 双面 chooseTarget）。
3. 每点断言四件事：
   a) **字节幂等**：save→restore→save 逐字节相等（含 events、choice/context、RNG 原始对、绑定/游标）。
   b) **合法重答**：恢复后应答继续，终局与不存档直跑同果（哈希相等）。
   c) **分片 abort**：save 后原引擎丢弃（drop），restore 出的新引擎继续——恢复路径非原地。
   d) **拒档负例**（每类至少 1）：伪 Transfer（伪造 held/eligible）、错 I（compound 绑定错技能）、
      错锁/版本不符→restore 拒绝且错误分类正确；G.After 历史引用不得再次算 owner（§7）。
4. 持久增量边界守卫：断言机器快照不含非清单字段（marks/SkillInstance.enabled 复用、不新增 R05 binding hash）。

## 实施要点
- 新测试文件 tests/r05i_byte_chain.rs（登记 groups/misc.rs）＋工厂 helper（每等待点＝探针钩子：在窗口/询问开放时
  截获 Engine 存档；参照 boundary_snapshot 的 open_choice 截停法＋D/R/G 帧推进的既有 r05 测试布阵）。
- 红先行：工厂至少 3 个锁链修前红（哪些等待点当前 save→restore→save 非字节幂等/恢复拒收——若全绿则作为守卫固化并报告）。
- 三 crate 全量＋门禁；证据 /private/tmp/noname-r05/evidence/i-{01,02}.log＋i-report.md。
