# ADR 0384: ABI 515 兵势程普（督作/避憾）

- 状态：已完成
- 日期：2026-08-02
- ABI：515

旧 JavaScript 仅作为语义证据。一个窄原语 `resolve_potchengpu` 复用实体牌移动、选择屏障和绑定伤害减值：督作在获得牌后选择角色获得牌堆中的火杀；避憾在杀伤害结算前令自己或受伤角色将手牌弃至体力值并令伤害减一。

督作移牌使用追加的 `CardMoveReason::Potduzuo` 精确阻断自身递归，不遗留可能误判未来同卡移动的临时状态。门禁：程普 Golden 3/3、兵势包闭包、移植角色闭包、content-compiler 463/463、Rule VM、content-runtime、扩展导入与 conformance 8/8 全部通过。
