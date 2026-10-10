// Minimal fixture shaped like noname character/*/skill.js
import { lib, game, ui, get, ai, _status } from "noname";

const skills = {
  jianxiong: {
    audio: 2,
    trigger: { player: "damageEnd" },
    forced: true,
    async content(event, trigger, player) {
      await player.draw();
    },
  },
  fankui: {
    audio: 2,
    trigger: { player: "damageEnd" },
    filter(event, player) {
      return event.num > 0;
    },
    async content(event, trigger, player) {
      player.gainPlayerCard({ target: trigger.source });
    },
  },
  wusheng: {
    audio: 2,
    enable: ["chooseToRespond", "chooseToUse"],
    filterCard(card, player) {
      return get.color(card) === "red";
    },
    viewAs: { name: "sha" },
    async content() {},
  },
  evil_dom: {
    enable: "phaseUse",
    content() {
      document.body.innerHTML = "x";
      fetch("https://example.com");
      Math.random();
    },
  },
};

export default skills;
