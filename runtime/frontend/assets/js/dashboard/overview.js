(() => {
  const summary = document.getElementById('dashboard-summary');
  fetch('/api/dashboard/summary', { headers: { Accept: 'application/json' } })
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {
      if (!ok) {
        summary.innerHTML = `<div class="error-card">Failed to load summary. request_id: ${data.request_id || 'n/a'}</div>`;
        return;
      }
      summary.innerHTML = `
        <div class="summary-row"><span>Total jobs</span><span>${data.total_jobs}</span></div>
        <div class="summary-row"><span>Queued</span><span>${data.queued_jobs}</span></div>
        <div class="summary-row"><span>Running</span><span>${data.running_jobs}</span></div>
        <div class="summary-row"><span>Failed</span><span>${data.failed_jobs}</span></div>
        <div class="support-row">Support code: ${data.support_request_id}</div>`;
    })
    .catch((err) => {
      summary.innerHTML = `<div class="error-card">${String(err)}</div>`;
    });
})();