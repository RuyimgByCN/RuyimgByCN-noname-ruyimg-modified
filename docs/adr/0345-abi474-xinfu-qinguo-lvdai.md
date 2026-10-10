# ADR 0345：ABI 474 勤国装备授杀复合 + sp2 吕岱

本批为 sp2 包吕岱（`lvdai`，2417250409）的「勤国」（`xinfu_qinguo`，2648676099，
定义位于 sp 包）完整移植，需要 ABI 474 新过滤器与复合。

## 原语设计（ABI 474）

- 新过滤器 `owner_equip_zone_changed`：CardsMoved 且拥有者装备区有牌进入/离开。
- 复合 `resolve_xinfu_qinguo(player)`（`EffectRequest::ResolveXinfuQinguo`，本次使用
  的牌由 vm_host 捕获）：`card_used owner_is_source` 触发，校验本次使用为装备牌后
  打开目标选择（可取消）→ `UseVirtualCardOn(player, Sha, target)`。
- 回复分支：`cards_moved owner_equip_zone_changed` + DSL 条件
  `hp(self) < max_hp(self)` 与 `empty_equip_slots(self) == 5 - hp(self)`（等价
  「装备区牌数 == 体力值」，引擎空位查询固定按 5 槽计算）。

## 打包与验证

- `xinfu_qinguo.skill` 加入 sp2 README 显式列表；`lvdai`（男/吴/4 体力，技能
  [xinfu_qinguo]）加入 sp2 `characters.json`；catalog（sp 包条目）标记 `ported`。
- sp2 npack 重建（ABI 474，45 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`。
- 运行时测试 `original_sp2_xinfu_qinguo.rs`（3 用例全绿）：装备后虚拟【杀】命中；
  使用【杀】不触发；装备数降至等于体力时回复 1 点。
- 旧 JavaScript 不进入 npack 或运行时。
