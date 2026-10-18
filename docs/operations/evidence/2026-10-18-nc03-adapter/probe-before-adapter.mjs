// NC03 一次性反例探针（改动前适配层）：复现 2026-10-18 架构审查的三条源码反例。
//
// 复跑步骤（工作目录 noname-next/）：
//   1. git show HEAD:apps/web-client/src/worker/nativeBackend.ts \
//        > apps/web-client/src/worker/nativeBackend.before.ts   # 需先 checkout 到修复前提交
//   2. node --import ./apps/web-client/scripts/ts-register.mjs \
//        docs/operations/evidence/2026-10-18-nc03-adapter/probe-before-adapter.mjs
//   3. 删除第 1 步生成的 .before.ts（该副本不随仓库保留）
// 本探针只证明修复前源码存在这三条反例；修复后的回归见
// apps/web-client/scripts/native-adapter.test.mjs（pnpm --dir apps/web-client test）。
const beforeUrl = new URL(
  "../../../../apps/web-client/src/worker/nativeBackend.before.ts",
  import.meta.url,
);
const { nativeDispatch, nativeEnvelope } = await import(beforeUrl.href);

const VIEW = JSON.stringify({
  seat: 0,
  players: [
    {
      seat: 0,
      alive: true,
      hp: 4,
      maxHp: 4,
      hand: { own: [11] },
      handCount: 1,
      equips: [],
      judge: [],
      turnedOver: false,
      skills: [],
      revealedGenerals: [],
    },
  ],
  deckCount: 10,
  discard: [],
  table: [],
  turn: 0,
  phase: "play",
  round: 1,
  outcome: null,
});
const rejected = JSON.stringify({ outcome: "rejected", revision: 7, finished: false });

function handle(choice, reply) {
  return {
    seed: 1,
    players: 4,
    observer: 0,
    engine: {
      view_json: () => VIEW,
      is_finished: () => false,
      revision: () => 7,
      open_choice_json: () => (choice === null ? "null" : JSON.stringify(choice)),
      answer: () => reply,
      next_phase: () => reply,
      play_card: () => reply,
      auto_step: () => reply,
    },
  };
}

// 反例 1：候选值 42/99 → 索引 token 0/1，提交 token 0 却按候选值查找
{
  const h = handle(
    { id: 7, options: [{ card: 42 }, { card: 99 }], min: 1, max: 1, allowCancel: false },
    rejected,
  );
  const env = nativeEnvelope(h);
  const token = env.choice.options[0].token;
  const legal = env.legal_actions.find((a) => a.ResolveChoice);
  try {
    nativeDispatch(h, { type: "resolveChoice", actor: 0, choiceId: 7, selections: [token] });
    console.log("NOT REPRODUCED 1");
  } catch (error) {
    console.log(
      `REPRODUCED: emitted card choice token ${token} (label ${env.choice.options[0].label_key}) cannot be consumed for card 42; legal action sends ${JSON.stringify(legal.ResolveChoice.selections)}: ${error}`,
    );
  }
}

// 反例 2：引擎返回 rejected，适配层仍返回 envelope.result = null
{
  const h = handle({ id: 7, options: [{ card: 42 }], min: 1, max: 1, allowCancel: false }, rejected);
  const env = nativeDispatch(h, { type: "resolveChoice", actor: 0, choiceId: 7, selections: [42] });
  console.log(
    `${env.result === null ? "REPRODUCED" : "NOT REPRODUCED"}: engine outcome=rejected discarded; returned envelope.result is ${JSON.stringify(env.result)}`,
  );
}

// 反例 3：ChoiceOption Kind(105) 被标成 0
{
  const h = handle({ id: 9, options: [{ kind: 105 }], min: 1, max: 1, allowCancel: false }, rejected);
  const env = nativeEnvelope(h);
  const label = env.choice.options[0].label_key;
  console.log(
    `${label === "0" ? "REPRODUCED" : "NOT REPRODUCED"}: ChoiceOption Kind(105) becomes label ${label}`,
  );
}
