(async () => {
  const raw = await (await fetch('/extension/史诗卡牌/extension.js')).text();
  const saved = game.import;
  let pack;
  try {
    game.import = function (kind, factory) {
      if (kind === 'extension') { pack = factory(lib, game, ui, get, ai, _status); return; }
      return saved.apply(this, arguments);
    };
    (0, eval)(raw);
  } finally { game.import = saved; }
  if (pack?.name !== '史诗卡牌') throw Error('actual extension registration failed');
  window.l10Extension = pack;
  lib.config.extension_史诗卡牌_meihuaskills = true;
  lib.config.extension_史诗卡牌_shichangshi = false;
  lib.config.extension_十周年UI_enable = false;
  const events = [];
  for (const [name, definition] of Object.entries(pack.package.skill.skill)) {
    lib.skill[name] = definition;
    game.finishSkill(name);
  }
  Object.assign(lib.translate, pack.package.skill.translate);
  try { pack.precontent(); events.push({step:'precontent',success:true}); }
  catch (e) { events.push({step:'precontent',error:String(e),stack:e.stack}); }
  try { pack.content({},pack); events.push({step:'content_without_decadeUI',success:true}); }
  catch (e) { events.push({step:'content_without_decadeUI',error:String(e),stack:e.stack}); }
  for (const name of ['reguhuo','rangjie','k_danggu']) {
    if (!lib.skill[name]) throw Error('actual skill missing ' + name);
    game.finishSkill(name);
  }
  window.l10SourceLoad = {events,extensionName:pack.name,loadedSkillKeys:['reguhuo','rangjie','k_danggu'],
    precontentBody:pack.precontent.toString(),reguhuo:lib.skill.reguhuo.content?.toString(),
    rangjie:lib.skill.rangjie.content.toString(),k_danggu:lib.skill.k_danggu.content.toString(),
    k_danggu_contentx:lib.skill.k_danggu.contentx.toString(),
    dependencies:{EpicFX:typeof EpicFX,utils:typeof window.EpicFX?.utils,dui:typeof window.dui,dcdAnim:typeof window.dcdAnim,getEPICscs:typeof window.getEPICscs}};
  return JSON.stringify(window.l10SourceLoad);
})()
