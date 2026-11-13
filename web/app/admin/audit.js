// Admin → Audit: append-only record of account and triage actions.
// Handles and numeric ids only; no emails or coordinates are stored.

export const auditModule = {
  id: "audit",
  title: "Audit",
  order: 40,
  async mount(el, ctx) {
    el.innerHTML = `
      <div class="admin-toolbar">
        <button type="button" id="admin-audit-refresh" class="secondary">Refresh</button>
        <span id="admin-audit-summary" class="muted"></span>
      </div>
      <div id="admin-audit-list"></div>`;
    const list = el.querySelector("#admin-audit-list");
    const summary = el.querySelector("#admin-audit-summary");

    async function load() {
      const data = await ctx.api("/api/v1/admin/audit");
      const entries = Array.isArray(data?.entries) ? data.entries : [];
      summary.textContent = `${Number(data?.total ?? entries.length)} recorded actions`;
      if (!entries.length) {
        list.innerHTML = `
          <div class="admin-empty">
            <h3>No audit entries yet</h3>
            <p class="muted">Role changes, account removals and report triage are recorded here.</p>
          </div>`;
        return;
      }
      list.innerHTML = `
        <table class="admin-table">
          <thead><tr><th>When</th><th>Actor</th><th>Action</th><th>Target</th><th>Detail</th></tr></thead>
          <tbody>
            ${entries
              .map(
                (e) => `
              <tr>
                <td>${escapeHtml(fmtTime(e.created_at))}</td>
                <td>@${escapeHtml(e.actor_username)}</td>
                <td><strong>${escapeHtml(e.action)}</strong></td>
                <td>${escapeHtml(e.target)}</td>
                <td class="muted">${escapeHtml(e.detail || "—")}</td>
              </tr>`,
              )
              .join("")}
          </tbody>
        </table>`;
    }

    el.querySelector("#admin-audit-refresh").addEventListener("click", load);
    await load();
  },
};

function fmtTime(iso) {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? "unknown" : d.toLocaleString();
}

function escapeHtml(value) {
  return String(value ?? "").replace(/[&<>"']/g, (c) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  }[c]));
}
