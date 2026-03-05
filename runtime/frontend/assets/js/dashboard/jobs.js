(() => {
  const list = document.getElementById('jobs-list');
  const renderJobs = (jobs) => {
    if (!jobs.length) {
      list.innerHTML = '<div class="empty-card">No jobs available.</div>';
      return;
    }
    list.innerHTML = jobs.map((job) => `
      <a class="summary-card" href="/dashboard/jobs/${job.job_id}">
        <div class="summary-row"><span>Job</span><span>${job.job_id}</span></div>
        <div class="summary-row"><span>Status</span><span class="badge">${job.status}</span></div>
      </a>
    `).join('');
  };
  fetch('/api/dashboard/jobs', { headers: { Accept: 'application/json' } })
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {
      if (!ok) {
        list.innerHTML = `<div class="error-card">Failed to load jobs. request_id: ${data.request_id || 'n/a'}</div>`;
        return;
      }
      renderJobs(data.jobs || []);
    })
    .catch((err) => {
      list.innerHTML = `<div class="error-card">${String(err)}</div>`;
    });
})();