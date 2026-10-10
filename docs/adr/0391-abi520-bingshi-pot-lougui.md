# ADR 0391: ABI 520 兵势娄圭（灌沙/集驭）

- 状态：已完成
- 日期：2026-08-02
- ABI：520

旧 JavaScript 仅作为语义证据。`resolve_potguansha` 在出牌阶段结束时弃置全部手牌和装备牌，从共享牌堆依次取得至多等量基本牌，并以临时技能标记增加本回合手牌上限；限定次数复用触发器的整局计数，不增加第二套状态。

`resolve_potjiyu` 弃置一张手牌后，从共享牌堆取得每种尚未获得类别的基本牌、锦囊牌和装备牌各至多一张。技能实例只记录本次取得的实体牌与本阶段已重置次数；这些牌全部离开手牌时重置一次主动技次数，整个出牌阶段最多重置两次，阶段结束清理。

门禁：势娄圭 Golden 3/3、兵势全量 Golden、content-compiler 468/468、Rule VM、content-runtime、engine-api ABI 协调、26 个扩展包导入与 conformance 8/8 全部通过；既有 conformance 哈希未漂移。兵势包为 ABI 520、16 名武将、38 个技能，logic hash 为 `37dcd44ed25d8c445ef133fa7083d1f2f2971750fb57f13112fe68971233cae3`，三份包文件 SHA-256 均为 `3fb8f59382ee81a54730c00cf9d39a4cea324b6df11a48b1a190cfbf1fc94cbf`，导入后的 content lock 为 `5a4aabbf5ba4f0c25d66d4c5455fd2d189404a5d16f12e42d87267c27c82defb`。全量 engine-core tests 的编译另被 `original_xianding_batch4.rs` 引用不存在的旧 `ContinuationKind` 变体阻断，与本 ADR 无关。
