const State = (() => {
  const _store = {
    nodeStatus:      null,
    networks:        [],
    peers:           [],
    metrics:         null,
    metricsStatus:   null,
    activeNetworkId: null,
    tokens:          [],
    moons:           [],
    controllerNets:  [],
    exitnodeStatus:  null,
    config:          null,
  };
  const _listeners = {};

  return {
    get(key)         { return _store[key]; },
    set(key, value)  {
      _store[key] = value;
      (_listeners[key] || []).forEach(fn => fn(value));
    },
    on(key, fn)      {
      if (!_listeners[key]) _listeners[key] = [];
      _listeners[key].push(fn);
      return () => { _listeners[key] = _listeners[key].filter(f => f !== fn); };
    },
  };
})();
