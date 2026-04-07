const Table = {
  // Render a filterable table
  // cols: [{key, label, render?}], rows: array, options: {filter, actions?}
  render(cols, rows, { filter = '', emptyMsg = 'No items', actions } = {}) {
    const filtered = filter
      ? rows.filter(r => JSON.stringify(r).toLowerCase().includes(filter.toLowerCase()))
      : rows;

    if (!filtered.length) return `<div class="empty-state"><div class="empty-state-icon">📋</div><h3>${emptyMsg}</h3></div>`;

    const headers = cols.map(c => `<th>${c.label}</th>`).join('') +
      (actions ? '<th></th>' : '');
    const body = filtered.map(row => {
      const cells = cols.map(c => `<td>${c.render ? c.render(row, row[c.key]) : (row[c.key] ?? '')}</td>`).join('');
      const acts = actions ? `<td>${actions(row)}</td>` : '';
      return `<tr>${cells}${acts}</tr>`;
    }).join('');

    return `<div class="table-wrap"><table><thead><tr>${headers}</tr></thead><tbody>${body}</tbody></table></div>`;
  },
};
