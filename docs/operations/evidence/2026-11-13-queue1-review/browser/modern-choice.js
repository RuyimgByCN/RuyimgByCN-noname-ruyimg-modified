new Promise((resolve, reject) => {
  let ticks = 0;
  const tick = () => {
    const { _status, ui, game } = q1;
    if (window.queue1Error) return reject(Error(window.queue1Error));
    const event = _status.event;
    if (_status.paused && event?.name === window.queue1Expected) return resolve(JSON.stringify({
      event: event.name, prompt: event.prompt, player: game.players.indexOf(event.player),
      forced: event.forced, confirm: ui.confirm?.str, controls: event.controls,
      hand: game.me.getCards('h').map(c => ({ name: c.name, suit: c.suit, rank: c.number })),
      buttons: ui.dialog?.buttons?.map(b => ({ text: b.innerText, link: b.link?.name || b.link })),
    }));
    if (++ticks > 500) return reject(Error('choice timeout: ' + event?.name));
    setTimeout(tick, 20);
  };
  tick();
});
