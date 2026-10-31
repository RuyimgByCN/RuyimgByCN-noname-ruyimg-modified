# R05：三技能精确运行时合同设计（2026-10-31登记）

本页保全设计演进；实施／版本授权见裁决28及29，不表示规则验收。Main当前5fc0f774、330／NGSA105／ABI758／WS6；`xianding/tiqi`涕泣、`xianding/baoshu`宝梳、`sp2/dcbihuo`避祸三份原INTENTIONAL_EMPTY_NOOP仍属必做缺失功能，未改成登记差异。实际331及随后332验收关系不变。

[准确源码审计](source-audit.raw.txt)纠正早期mobile／bihuo混配。三个固定compound采用新SDK／rule-vm发布路线，但所需Prepare／Draw全阶段、真实gain历史、签名标记寿命、Damage Source／End、异步draw与恢复／投影均未实现。191处draw调用（kernel44／content147、68文件）完整清单在不可变[v2](proposal-v2.raw.txt)附录；清单不代表191路径已迁移／验收。

原[提案v1](proposal-v1.raw.txt)及[审查](review-v1.raw.txt)保留七项阻断。[v2](proposal-v2.raw.txt)解决兼容政策、191caller范围和source死亡filterStop，但[复审](review-v2.raw.txt)发现Transfer后held引用矛盾、official专属能力限制、Next复活enabled歧义、宝梳addMark零／负参数及重复持久指纹问题。

[v3规范](proposal-v3.raw.txt)分别修订为分步历史引用与实体唯一owner、任意验证调用包／逻辑技能身份隔离、Next死亡保原enabled、宝梳m0按源默认+1且负数／Some0保留、删除R05专用持久hash。五项已在[设计复审](review-v3.raw.txt)关闭；这仅是设计反例回答，不是代码Fixed。复审仍有P1：现浏览器contentLock只是string[]，packsJson已经重序列化，而Rust content_hash锁原包字节，完整ContentLock及原字节未贯通实际各恢复参数。

[v4附件](proposal-v4.raw.txt)补齐原始字节、完整锁及Browser／WASM／Desktop／server／HTTP／WAL恢复数据流。[v4复审](review-v4.raw.txt)关闭这些设计接缝，但保留一项P1：同时接收裸packs和验证上下文，仍能改牌的range却保持来源标签。

[v5七条勘误](proposal-v5.raw.txt)删除独立裸packs输入，由受控工厂从不可变验证上下文直接构建Catalog／Engine，保留可信Rust接口与现有分层。[最终设计复审](review-v5.raw.txt)No issues／OK with notes，最后一项设计P1关闭，可提交具体用户契约审批。没有产品源码实现或测试通过结论。

历史批准的v3＋v4＋v5合同、跨端原文／保存信封、旧扩展缺上下文拒绝政策及333／106／759／WS7见[裁决28原选择与稿件SHA](user-decision28.json)。用户随后[裁决29](../2026-10-31-c331-runtime-contract/user-decision29.json)批准C完整Core机制与schema5，并将档号明确调整为**C331/106→改命332/106→R05-333/107**；R05 ABI759／WS7不变。具体合同仍为28的v3＋v4＋v5，29只替代其R05档号106预约，并批准把必要通用Damage机制先在C落实；Prepare／191Draw及三R05技能仍待实际331→332之后。旧批准记录和raw稿件不改写，产品实施及真实验收仍pending。

NC10裁决27仅授权Own `source_skill`及兼容剥离，R05使用随后明确的裁决28，不能相互借用范围。三noop、12类场景、原生／真实WASM／上游／平台完整验收全部pending。全体提案及review原始字节按[复制清单](copy-manifest.json)逐SHA保全；按实际331→332前置后实施，不能用本文替代实际源码验证。
