const Modal = (() => {
  let _resolve = null;
  const overlay = () => document.getElementById('modal-overlay');

  function open(html) {
    overlay().innerHTML = `<div class="modal">${html}</div>`;
    overlay().classList.add('active');
  }

  function close() {
    overlay().classList.remove('active');
    setTimeout(() => { overlay().innerHTML = ''; }, 200);
  }

  overlay()?.addEventListener('click', e => {
    if (e.target === overlay()) { close(); if (_resolve) { _resolve(false); _resolve = null; } }
  });

  return {
    confirm(message, { danger = false } = {}) {
      return new Promise(res => {
        _resolve = res;
        open(`
          <div class="modal-title">Confirm</div>
          <div class="modal-body">${message}</div>
          <div class="modal-foot">
            <button class="btn btn-ghost" id="modal-cancel">Cancel</button>
            <button class="btn ${danger ? 'btn-danger' : 'btn-primary'}" id="modal-ok">Confirm</button>
          </div>`);
        document.getElementById('modal-ok').onclick     = () => { close(); res(true); _resolve = null; };
        document.getElementById('modal-cancel').onclick = () => { close(); res(false); _resolve = null; };
      });
    },

    prompt(message, placeholder = '') {
      return new Promise(res => {
        _resolve = res;
        open(`
          <div class="modal-title">${message}</div>
          <div class="modal-body">
            <input class="input" id="modal-input" placeholder="${placeholder}" style="margin-top:8px">
          </div>
          <div class="modal-foot">
            <button class="btn btn-ghost" id="modal-cancel">Cancel</button>
            <button class="btn btn-primary" id="modal-ok">OK</button>
          </div>`);
        const input = document.getElementById('modal-input');
        input.focus();
        document.getElementById('modal-ok').onclick     = () => { close(); res(input.value); _resolve = null; };
        document.getElementById('modal-cancel').onclick = () => { close(); res(null); _resolve = null; };
        input.onkeydown = e => { if (e.key === 'Enter') document.getElementById('modal-ok').click(); };
      });
    },

    close,
  };
})();
