// Modular admin console: a registry of independent modules.
// Adding a future domain (diagnostics, audit, jobs) means adding one module
// file and one entry to BUILTIN — no changes to the map/social screens.

import { overviewModule } from "./admin/overview.js";
import { usersModule } from "./admin/users.js";
import { diagnosticsModule } from "./admin/diagnostics.js";
import { auditModule } from "./admin/audit.js";

const $ = (id) => document.getElementById(id);

const BUILTIN = [overviewModule, usersModule, diagnosticsModule, auditModule];

export function createAdminConsole({ api, setStatus, getUser }) {
  const overlay = $("admin-panel");
  const nav = $("admin-nav");
  const content = $("admin-content");
  const title = $("admin-title");
  const who = $("admin-user");

  const modules = new Map();
  let active = null;

  function register(module) {
    if (!module?.id) throw new Error("admin module requires an id");
    modules.set(module.id, module);
  }
  BUILTIN.forEach(register);

  const ctx = {
    api,
    setStatus,
    getUser,
    openModule: (id) => activate(id),
    modules: () => [...modules.values()].sort((a, b) => (a.order || 0) - (b.order || 0)),
  };

  function renderNav() {
    nav.innerHTML = "";
    for (const module of ctx.modules()) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "admin-nav-item";
      btn.dataset.module = module.id;
      btn.textContent = module.title;
      btn.addEventListener("click", () => activate(module.id));
      nav.appendChild(btn);
    }
  }

  async function activate(id) {
    const module = modules.get(id);
    if (!module) return;
    if (active?.destroy) active.destroy();
    active = module;
    nav.querySelectorAll(".admin-nav-item").forEach((btn) => {
      btn.classList.toggle("active", btn.dataset.module === id);
    });
    title.textContent = module.title;
    content.innerHTML = '<p class="muted">Loading…</p>';
    try {
      await module.mount(content, ctx);
    } catch (err) {
      content.innerHTML = `<p class="error">${escapeHtml(err.message || "Failed to load module.")}</p>`;
    }
  }

  function open(id = "overview") {
    const user = getUser?.();
    who.textContent = user ? `${user.email} · ${user.role}` : "";
    overlay.classList.remove("hidden");
    renderNav();
    activate(id);
  }

  function close() {
    if (active?.destroy) active.destroy();
    active = null;
    overlay.classList.add("hidden");
  }

  function refresh() {
    if (!overlay.classList.contains("hidden") && active) activate(active.id);
  }

  return { open, close, refresh, register };
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
