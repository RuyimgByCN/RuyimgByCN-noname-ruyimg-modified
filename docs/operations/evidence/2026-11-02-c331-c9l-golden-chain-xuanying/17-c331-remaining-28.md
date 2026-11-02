# C331 剩餘 28 項：可執行清單（2026-11-02 整理，來源為 2026-10-31 定向核驗）

權威來源：`docs/operations/evidence/2026-10-31-c331-source-followups/`（`translation16.json`、`source-and-bridge-report.raw.txt`）。
登記基準：**已證 262／總 290**，剩 28＝**16 譯名**（17 個 key，lexue1/2 合算一項）＋**12 機制／來源**。

## 一、16 項譯名（全部 conditional-ambiguous-still-blocked）

| # | variant | key | 上游來源（file:line） | 阻塞點 |
| --- | --- | --- | --- | --- |
| 1 | BossBaimang2x | boss_baimang2x | upstream/noname-liuli/noname/mode/boss.js:5652 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 2 | BossBaimang3x | boss_baimang3x | upstream/noname-liuli/noname/mode/boss.js:5740 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 3 | BossBianshen3x | boss_bianshen3x | upstream/noname-liuli/noname/mode/boss.js:7493 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 4 | BossBianshen4x | boss_bianshen4x | upstream/noname-liuli/noname/mode/boss.js:7521 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 5 | BossChiyan2x | boss_chiyan2x | upstream/noname-liuli/noname/mode/boss.js:5127 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 6 | BossChiyan3x | boss_chiyan3x | upstream/noname-liuli/noname/mode/boss.js:5215 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 7 | BossQingmu2x | boss_qingmu2x | upstream/noname-liuli/noname/mode/boss.js:5298 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 8 | BossQingmu3x | boss_qingmu3x | upstream/noname-liuli/noname/mode/boss.js:5386 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 9 | BossXuanlin2x | boss_xuanlin2x | upstream/noname-liuli/noname/mode/boss.js:5469 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 10 | BossXuanlin3x | boss_xuanlin3x | upstream/noname-liuli/noname/mode/boss.js:5564 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 11 | JinlianzhuCancel | g_jinlianzhu | upstream/noname-liuli/noname/card/gujian.js:1776 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 12 | ShengkeCheck | _shengke | upstream/noname-liuli/noname/extension/wuxing/extension.js:62 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 13 | SwdLexue | lexue1 | upstream/noname-liuli/noname/character/swd.js:7098 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 14 | SwdShencaojieGuard | g_shencaojie | upstream/noname-liuli/noname/card/swd.js:2439 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 15 | SwdShenmiguoWindow | g_shenmiguo | upstream/noname-liuli/noname/card/swd.js:2454 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |
| 16 | ZhuWanzhongJi | _xigua_zhuwanzhongji_ | upstream/noname-liuli/noname/extension/标记补充/extension.js:2290 | Actual enabled/install/load order and config/storage/late-write state are not bound. Parsed table absence is not global absence; aliases, computed keys, dynamic |

共同阻塞（16/16）：最終 `lib.translate` 真值未觀測——必須先凍結一份真實安裝／啟用來源實例組合（含宣告依賴、namespace lock、`config.translate`／`lib.config.translate`、mode／video 旗標與觀測點），再用既有 AST 解析器做白名單資料流重放；不得把「無字面條目」或 charlotte 當 false。

## 二、12 項機制／來源（4 份報告分組；括號為對應項數）

- 1 → 數值／來源衝突與 Qianlong 合併（約 3 項：Longxue/Weitong、Wudian sanjian、Swd nigong）
- 2 → 四個救援動作（4 項：Niepan／Shouyin／Chengxin／Shuiyun5）
- 3 → 兩個到期狀態（2 項：Gwent fengyin／Swd fengyin）
- 4 → 崑崙鏡與玄影（2–3 項）


1. Numeric/source conflicts and merged Qianlong: actual Longxue assets select Liuli for mbqianlong/mbweitong. Weitong needs Normal/1000/true on its genuine initial window; Qianlong needs initial Normal/600/true versus gain/damage Normal/0/true. Wudian wd_sanjian and Swd nigong select noname producers: core -25 for each, while Liuli requires -30 and 0 respectively. Resolve a qualified declaration binding rather than hardcode a source filepath. Current noname remains the C22 baseline; Longxue is an explicit Liuli producer, and Liuli deltas remain registered. Official mobile has the two logical skills but no trigger handlers for them, so it is not their native host producer.
2. Four rescue actions: Niepan, Shouyin, Chengxin and Shuiyun5 are chooseToUse, not upstream trigger entries. Native Dying commits then collects AfterTriggers, sorts/asks order, confirms optional host skills, and only then enters intrinsic DyingSaveMachine (Tao/Pass responder rounds). The comparator therefore applies to their current synthetic native entries, but source arrangeTrigger formulas do not establish those bridge tuples. They remain required scope and need a real rescue action/pass/retry/interleaving contract through existing DyingSaveMachine, choice and effect seams. Chengxin/Shuiyun native filters additionally exclude self; selected source filters do not.
3. Two expiry states: source temp fengyin removal happens before addList, at owner full-turn phaseAfter (Gwent) or phaseBegin (Swd). Native host PhaseBegin executes after phase Commit; PhaseEnd is each phase. Queue order cannot encode removal before collection. Re-enabling every skill also loses unrelated disable causes and locked/persevere/forever exemptions. Use existing phase/frame lifecycle for exact expiry and blocker removal; do not manufacture source tuples or derive nonselectability from charlotte.
4. Kunlunjing and Xuanying: both sources grant Kunlunjing1/2 through the parent group, with Normal/1000/selectable rollback at full-turn begin and Normal/1/nonselectable snapshot at full-turn end. One SkillDef tuple cannot express that. Native snapshots also use shared turn storage. Liuli altered-mode check/loseHp differs from current noname. Xuanying damage is registered as a child definition but absent from both parent groups (sha/use only); selected package grants the parent, not damage. Native parent ownership incorrectly makes damage reaction reachable. Keep actual grant reachability separate from the declared Normal/0/true child tuple.

## 三、功能面仍列 BLOCK（同批範圍）

- indexed、forceDie/out/hidden/filterStop、真實 PhaseBegin／死亡窗、來源特定 grant／rescue／expiry。
- host 宣告覆蓋、同技能多 handler、救援動作或新持久 blocker：若需新合同，必須給具體接口／版本方案，不能借 schema4 授權擴大協議。

## 四、與 C9L 的關係

- 玄影（Xuanying）正是第 4 項的一部分：damage 子技只登記在子定義、不在任何 parent group（只有 sha/use），原生父技所有權讓 damage 反應變「可達」——C9L 的 `=> false` 是**臨時阻擋**，正解是 parent-group grant，不是恢復單一雙面行為。
- 其餘 27 項都不在 C9L 範圍；本清單只整理，未做任何登記數字變更。
