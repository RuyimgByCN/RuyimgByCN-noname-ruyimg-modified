// Batch legacy JS fixture — 10 skills in one file (§36 batch migration test)
var skills = {
    // 1. Trigger skill (forced)
    yiji: {
        trigger: { player: "damageEnd" },
        forced: true,
        content: function () { player.draw(1); },
    },
    // 2. Modifier skill
    mashu2: {
        mod: { globalFrom: function (f, t, d) { return d - 1; } },
    },
    // 3. Active skill with enable
    jiedao: {
        enable: "phaseUse",
        filterCard: function (card) { return get.name(card) == 'sha'; },
        content: function () { 'step 0' player.useCard({ name: 'sha' }); },
    },
    // 4. viewAs skill
    longdan2: {
        viewAs: { name: 'shan' },
        enable: "use",
        filterCard: function (card) { return get.color(card) == 'red'; },
    },
    // 5. Trigger with filter
    ganglie2: {
        trigger: { player: "damageEnd" },
        forced: true,
        filter: function (event, player) { return event.num > 0; },
        content: function () { 'step 0' player.draw(1); },
    },
    // 6. Skill with ai block
    kurou2: {
        enable: "phaseUse",
        filterCard: function (card) { return true; },
        check: function (card) { return 8; },
        content: function () { player.loseHp(1); player.draw(2); },
        ai: { order: 2, result: { player: 1 } },
    },
    // 7. Frequent trigger
    tiandu2: {
        trigger: { global: "judgeEnd" },
        frequent: true,
        content: function () { player.draw(1); },
    },
    // 8. Skill with group subskills
    wushuang2: {
        group: ["wushuang2_1", "wushuang2_2"],
        subSkill: {
            wushuang2_1: { trigger: { player: "shaBegin" }, forced: true },
            wushuang2_2: { trigger: { target: "shaBegin" }, forced: true },
        },
    },
    // 9. Simple passive mod
    yingzi2: {
        mod: {
            maxHandcard: function (player, num) { return num + 1; },
        },
    },
    // 10. Direct trigger (no forced)
    fankui2: {
        trigger: { player: "damageEnd" },
        direct: true,
        content: function () { 'step 0' player.draw(1); },
    },
};
