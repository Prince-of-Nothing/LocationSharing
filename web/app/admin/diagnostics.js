// Admin → Diagnostics: user-initiated reports, newest first, with triage.
// Reports are PII-free by design (category + message + public handle only).

export const diagnosticsModule = {
  id: "diagnostics",
  title: "Diagnostics",
  order: 30,
  async mount(el, ctx) {
    el.innerHTML = `
      <div class="admin-toolbar">
        <select id="admin-diag-status" aria-label="Filter by status">
          <option value="">All statuses</option>
          <option value="open">Open</option>
          <option value="triaged">Triaged</option>
          <option value="closed">Closed</option>
        </select>
        <button type="button" id="admin-diag-refresh" class="secondary">Refresh</button>
        <span id="admin-diag-summary" class="muted"></span>
      </div>
      <div id="admin-diag-list"></div>`;
    const status = el.querySelector("#admin-diag-status");
    const list = el.querySelector("#admin-diag-list");
    const summary = el.querySelector("#admin-diag-summary");

    async function load() {
      const query = status.value ? `?status=${encodeURIComponent(status.value)}` : "";
      const data = await ctx.api(`/api/v1/admin/diagnostics${query}`);
      const reports = Array.isArray(data?.reports) ? data.reports : [];
      summary.textContent = `${Number(data?.open ?? 0)} open · ${Number(data?.total ?? reports.length)} matching`;
      render(reports);
    }

    function render(reports) {
      if (!reports.length) {
        list.innerHTML = `
          <div class="admin-empty">
            <h3>No reports</h3>
            <p class="muted">Clients can submit a report from Help &amp; diagnostics. Nothing is collected automatically.</p>
          </div>`;
        return;
      }
      list.innerHTML = reports
        .map(
          (r) => `
        <article class="admin-report" data-id="${Number(r.id)}">
          <header>
            <span class="badge ${r.status === "open" ? "pending" : "accepted"}">${escapeHtml(r.status)}</span>
            <strong>${escapeHtml(r.category)}</strong>
            <span class="muted">#${Number(r.id)} · ${escapeHtml(fmtTime(r.created_at))} · @${escapeHtml(r.reporter)} · ${escapeHtml(r.platform || "unknown")} ${escapeHtml(r.app_version || "")}</span>
          </header>
          <p class="admin-report-message">${escapeHtml(r.message)}</p>
          <label class="muted">Triage
            <select data-action="status">
              ${["open", "triaged", "closed"]
                .map((s) => `<option value="${s}" ${s === r.status ? "selected" : ""}>${s}</option>`)
                .join("")}
            </select>
          </label>
        </article>`,
        )
        .join("");
    }

    list.addEventListener("change", async (event) => {
      const select = event.target.closest('select[data-action="status"]');
      const card = event.target.closest("article[data-id]");
      if (!select || !card) return;
      try {
        await ctx.api(`/api/v1/admin/diagnostics/${Number(card.dataset.id)}`, {
          method: "PATCH",
          body: { status: select.value },
        });
        ctx.setStatus("Report updated.");
        await load();
      } catch (err) {
        ctx.setStatus(err.message, true);
        await load();
      }
    });

    status.addEventListener("change", load);
    el.querySelector("#admin-diag-refresh").addEventListener("click", load);
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
