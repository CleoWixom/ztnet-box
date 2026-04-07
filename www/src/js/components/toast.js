const Toast = (() => {
  const container = () => document.getElementById('toast-container');

  function show(msg, type) {
    const el = document.createElement('div');
    el.className = `toast toast-${type}`;
    el.innerHTML = `<span class="toast-msg">${msg}</span><span class="toast-close" onclick="this.parentElement.remove()">✕</span>`;
    container().appendChild(el);
    setTimeout(() => {
      el.classList.add('removing');
      setTimeout(() => el.remove(), 200);
    }, 4000);
  }

  return {
    success: m => show(m, 'success'),
    error:   m => show(m, 'error'),
    info:    m => show(m, 'info'),
  };
})();
