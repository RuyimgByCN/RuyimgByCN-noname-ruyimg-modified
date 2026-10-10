# Bingshi ported content

`kunfen_pot_weiyan` is the locked end-phase HP-loss/draw skill gained when 势魏延 fails 忠傲.

`mb_xiahoushang` / `mbtanfeng` rewrites the legacy prepare-phase branch selection as a typed,
resumable discard-or-fire-damage effect; the legacy JavaScript is not executed.

`guoyuan` rewrites 清蹈、修耕、陈赦 with the generic complete-card boundary and typed resumable
choices; the two 修耕 helper skills are package-local definitions.

`mb_chenzhi` rewrites 权宠、任行 with the shared extra-turn queue, round-scoped skill state,
discard history, draw, and target-card discard pipelines.

`mb_huangzu` rewrites 鸱张、断鞅 with damage-card distance modifiers, per-card response-color
locks, playable skill piles, random selection, recast, and the normal card-use pipeline.

`mb_luyusheng` rewrites 润微、霜怀 with authoritative reveal/color selection, phase-scoped
loss/reset and gained-card tracking, damage prevention, discard-pile gain, and distance queries.

`mb_sunjun` rewrites 凶图、险帅 with target-hand reveal/discard, discard-pile suit history,
damage-triggered second activation, and first-hand-card-of-suit use-limit queries.

`mb_tianfeng` rewrites 刚鲠、死谏 with multi-card gifts, end-turn hand extrema,
hand-loss/dying triggers, cumulative last-stand cost, and a one-shot discard helper.

`mb_yanghong` rewrites 间计、远谟 with a shared secret comparison card, no-limit virtual Sha,
field-card movement, attack-range population deltas, and capped optional draws.

`mb_zhangyan` rewrites 飞径、骁戈 with damage-trick-to-Sha conversion, seat-path secret
discards, color-selected extra targets, bound damage prevention/recovery/reclaim, and Duel follow-up.

`pangxi` rewrites 蓄业、狂想 with minimum/maximum hand checks, HEJ draw-pile placement,
atomic full-hand exchange, and physical received-card exhaustion tracking.

`pot_chengpu` rewrites 督作、避憾 with gain-triggered physical fire-Sha transfer and
exact hand-to-HP discard before reducing the bound Sha damage.

`pot_xiahouba` rewrites 砺兵、破袭 with skill-local cyclic state, exact discarded-Sha reuse,
distance/use-count bypass, and target-scoped response suppression.

`pot_zhangren` rewrites 伏暗、引弦 with round-scoped suit records, turn discard-suit history,
no-limit virtual Sha, temporary non-locked-skill suppression, and dynamic attack range.

`pot_huanjie` rewrites 共谋、正朔 with whole-hand exchange, turn-scoped 奇策/看破 grants,
and an all-player discard, reshuffle, then four-card redeal.

`pot_sunchen` rewrites 逆固、戮连 with distinct-suit active costs, seat-ordered optional gifts,
bounded next-damage bonuses, target linking, conditional draw, and forced fire damage.

`pot_lusu` rewrites 好施、缔盟 with authorized cross-player hand use/response, two empty-hand
refills per grant, next-turn cleanup, and the existing resumable whole-hand exchange pipeline.

`pot_dongzhao` rewrites 妙略、迎驾 and the physical 瞒天过海 card with distinct-suit creation,
canonical zhinang gain, same-name trick history, queued extra turns, gain-then-seat-order-give,
and intrinsic hand-limit/discard-to-limit exemptions.

`pot_dengai` rewrites 屯田、凿险、急袭 with mark-backed charge and capacity state, ordered
heart-card distribution, discard-pile threshold rewards, current-turn target history, and the
ordinary per-target Wuxie/Shunshou pipeline.

`pot_chenjiao` rewrites 清严、策断 with round-increasing revealed response costs, persistent
shown-card lockout, reverse attack-range targeting, simultaneous reveal semantics, tied-majority
color materials, and the ordinary physical multi-card Sha pipeline.

`pot_taishici` rewrites 酣战、战烈、振锋 with authoritative draw-to-X rules, turn-scoped
discarded-Sha quotas, capped 烈 marks, selectable virtual-Sha enhancements, and persistent
limited-skill X-source rewrites.

`sp_zhonghui` rewrites 俟资、挟志、狱难、克昌 with capped charge spending, exact per-turn
Sha-user HP loss, permanent overflow bonuses, same-round death awakening, faction change, and
lord-wide Qun Sha distance plus the awakened unrespondable upgrade.

`pot_xinxianying` rewrites 诫节、清识 with private hand reveal, round-scoped suit unions,
unique-maximum twice-per-round bonuses, temporary suit-specific use-limit removal, identity-team
relations, and resumable draw/discard choices.

`pot_weiyan` rewrites 忠傲、壮誓、饮战、狂骨 with mission success/failure state,
the first-N-card distance/response/use-count policies, exact Sha-resolution follow-ups, and the
upgraded 背水 branch; the legacy skin/audio switches remain presentation-only.

`pot_zhouyu` rewrites 炽沄、焰洄、焚涛、雄姿 with resumable card gifts and reveals,
turn-scoped physical-card history, linked fire-damage choices, and the limited branch lock;
legacy skin changes remain presentation-only.

`pot_lvyi` rewrites 隳律、颂谳、势施 with round-scoped color grants, exact-card phase-end
settlement, discarded-color Wuxie conversion during the bound Wuzhong resolution, and per-card
response locks frozen from accumulated damage sources; legacy JavaScript is semantic evidence only.

`pot_yuji` rewrites 道转、符济 with resumable source-free basic-card declarations, per-turn and
per-round name limits, current-turn-player costs, paired tagged-card gifts, per-source suit gains,
Sha/Shan follow-ups, and next-card buffs expiring at the next own turn; legacy JavaScript is semantic
evidence only, while skin, CSS, and audio switches remain presentation-only.

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/bingshi/*.skill \
  --characters content/original/ported/bingshi/characters.json \
  --cards content/original/ported/bingshi/cards.json \
  --metadata content/original/ported/bingshi/metadata.json \
  --out content/original/ported/bingshi/bingshi.npack.json \
  --package-id bingshi
```
