new Promise((resolve, reject) => {
  let ticks = 0;
  const tick = () => {
    if (_status.paused && _status.event.name === window.queue1Expected) return resolve(JSON.stringify({
      event: _status.event.name, controls: _status.event.controls, confirm: ui.confirm?.str,
      prompt: _status.event.prompt, forced: _status.event.forced,
      hand: game.me.getCards('h').map(c => ({ name: c.name, suit: c.suit, rank: c.number })),
    }));
    if (++ticks > 500) return reject(Error('choice timeout at ' + _status.event.name));
    setTimeout(tick, 20);
  };
  tick();
});
