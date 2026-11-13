// Placeholder admin module — proves the registry extension point.
// Replace with a real module (file + register entry) when the domain lands.

export function createPlaceholderModule({ id, title, order, description }) {
  return {
    id,
    title,
    order,
    async mount(el) {
      el.innerHTML = `
        <div class="admin-empty">
          <h3>${escapeHtml(title)}</h3>
          <p class="muted">${escapeHtml(description)}</p>
          <p class="muted">This tab is registered through the admin module registry, so it needs no changes to the map or social UI.</p>
        </div>`;
    },
  };
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
