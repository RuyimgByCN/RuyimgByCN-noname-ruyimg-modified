# CLAN 确定性移植

## 已移植技能（5）

- `clanhuanyin`（涣音，荀采）：进入濒死时将手牌补至四张。
- `clanqingli`（清厉，荀爽）：每回合结束时将手牌补至体力上限，最多五张。
- `chenliuwushi`（陈留武氏）：同名状态标记每层令手牌上限加一。
- `clanbalong`（拔垄，荀淑）：每回合首次体力变化后，锦囊牌为手牌中唯一最多类别时展示手牌并补至存活角色数。
- `clandandao`（弹道，荀爽）：锁定技。任意角色本局首次判定结算后，拥有者增加 1 点体力上限；「每名角色整局仅一次」以判定者身上的技能标记近似（`judgment_resolved` + `mark_count(event_player, …)`）。

## 武将登记

族包武将普遍为三技能武将且技能互相关联，尚未有武将技能齐全，暂不登记整名武将（遵循「暂不发布整名武将」惯例）。

## 打包命令

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/clan/clanhuanyin.skill \
  content/original/ported/clan/clanqingli.skill \
  content/original/ported/clan/chenliuwushi.skill \
  content/original/ported/clan/clanbalong.skill \
  content/original/ported/clan/clandandao.skill \
  --metadata content/original/ported/clan/metadata.json \
  --out content/original/ported/clan/clan.npack.json \
  --package-id clan
```

## 原语缺口（E 段候选，推进族包所需）

族包其余技能依赖以下引擎能力，当前 DSL 无法表达，需在 E 段（559–599）领取新 ABI：

- ~~家族（hasClan）查询~~（部分落地：引擎 `player_has_clan` + clanzhongliu 过滤器、本批 `owner_alive_event_player_self_or_clan_<族名>`；剩余 clanmuyin/clanshize 需手牌上限比较/响应检测等配套原语）
- 横置/重置指定角色或自身（clanfangzhen / clanxieshu / clanbaozu / clandianzhan 等）
- 技能使用计数重置 refreshSkill（clanzhongliu）
- ~~牌名字数 cardNameLength（clanjiejian / clanhuanghan / clanxieshu / clanyuzhi）~~ 2026-08-15：jiejian/huanghan/xieshu 零原语重写落地；yuzhi 已由 08-10 批次覆盖
- 转换技状态读写（clandongxu / clanguangu / clanlianzhu / clanjiexuan / spshidi 同型）
- 手牌点数和判定（clanqiajue）
- gaintag 牌上标记（clanjiannan / clanxsyingxiang / clanshenjun / clanyuzhi 等）
