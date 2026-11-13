// Admin → Overview: deployment counters. Pure view over /api/v1/admin/overview.

export const overviewModule = {
  id: "overview",
  title: "Overview",
  order: 10,
  async mount(el, ctx) {
    const data = await ctx.api("/api/v1/admin/overview");
    const cards = [
      ["Users", data.users],
      ["Active users", data.active_users],
      ["Sessions", data.sessions],
      ["Friendships", data.friendships],
      ["Shares", data.shares],
      ["Open reports", data.diagnostics],
      ["Audit events", data.audit_events],
    ];
    el.innerHTML = `
      <p class="muted">All accounts, including anonymous and never-signed-in users. Pinpoint ${escapeHtml(data.version || "")} · in-memory view when no database is configured.</p>
      <div class="admin-cards">
        ${cards
          .map(
            ([label, value]) => `
          <div class="admin-card">
            <span class="admin-value">${Number(value ?? 0)}</span>
            <span class="admin-label">${escapeHtml(label)}</span>
          </div>`,
          )
          .join("")}
      </div>
      <p class="muted">Diagnostics counts open reports. Audit lists role changes, deletions and triage actions.</p>`;
  },
};

function escapeHtml(value) {
  return String(value ?? "").replace(/[&<>"']/g, (c) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  }[c]));
}
