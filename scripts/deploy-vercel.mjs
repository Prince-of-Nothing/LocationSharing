// Deploy web/public to Vercel via the REST API (no CLI login needed).
//
// Usage (PowerShell):
//   $env:VERCEL_TOKEN="vck_..."   # never commit the token
//   node scripts/deploy-vercel.mjs
//
// The script creates the `pinpoint-web` project if missing, uploads every
// file in web/public (including vercel.json with the /api rewrites) as a
// production deployment and polls until Vercel reports it READY.
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

const TOKEN = process.env.VERCEL_TOKEN;
if (!TOKEN) {
  console.error("VERCEL_TOKEN is not set");
  process.exit(1);
}

const ROOT = join(fileURLToPath(new URL(".", import.meta.url)), "..");
const PUBLIC = join(ROOT, "web", "public");
const PROJECT = "pinpoint-web";
const API = "https://api.vercel.com";

async function vercel(path, init = {}) {
  const res = await fetch(API + path, {
    ...init,
    headers: {
      Authorization: `Bearer ${TOKEN}`,
      "Content-Type": "application/json",
      ...(init.headers || {}),
    },
  });
  const text = await res.text();
  let body;
  try {
    body = JSON.parse(text);
  } catch {
    body = text.slice(0, 300);
  }
  return { status: res.status, body };
}

function walk(dir) {
  const out = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) out.push(...walk(full));
    else out.push(full);
  }
  return out;
}

// 1) Ensure the project exists.
const project = await vercel(`/v9/projects/${PROJECT}`);
if (project.status === 404) {
  const created = await vercel("/v9/projects", {
    method: "POST",
    body: JSON.stringify({ name: PROJECT, framework: null }),
  });
  if (created.status >= 300) {
    // Some scoped tokens cannot manage projects; the deployment endpoint may
    // still be allowed (and auto-creates the project), so keep going.
    console.warn(
      "project create not allowed (" + created.status + "); trying deployment anyway",
    );
  } else {
    console.log("project created:", PROJECT);
  }
} else if (project.status !== 200) {
  console.warn("project lookup:", project.status, "continuing");
} else {
  console.log("project exists:", PROJECT);
}

// 2) Upload every file as a base64 entry (legacy raw deployment API).
const files = walk(PUBLIC).map((full) => ({
  file: relative(PUBLIC, full).split(sep).join("/"),
  data: readFileSync(full).toString("base64"),
  encoding: "base64",
}));
console.log("uploading", files.length, "files...");

const deployment = await vercel(`/v13/deployments?forceNew=1`, {
  method: "POST",
  body: JSON.stringify({
    name: PROJECT,
    target: "production",
    files,
    projectSettings: { framework: null },
  }),
});
if (deployment.status >= 300) {
  console.error("deployment create failed:", deployment.status, JSON.stringify(deployment.body));
  process.exit(1);
}
const id = deployment.body.id;
console.log("deployment:", id, "state:", deployment.body.readyState);

// 3) Poll until READY / ERROR (max ~3 min).
for (let i = 0; i < 36; i++) {
  await new Promise((r) => setTimeout(r, 5000));
  const check = await vercel(`/v13/deployments/${encodeURIComponent(id)}`);
  const state = check.body?.readyState;
  const status = check.body?.ready;
  console.log(`  poll ${i + 1}: ${state ?? check.status}`);
  if (state === "READY" || state === "ERROR" || state === "CANCELED") {
    const url = check.body.url;
    const aliases = check.body.alias || [];
    console.log("RESULT:", state);
    console.log("URL: https://" + url);
    for (const a of aliases) console.log("ALIAS: https://" + a);
    process.exit(state === "READY" ? 0 : 1);
  }
  if (status === false && check.status !== 200) break;
}
console.error("timed out waiting for deployment");
process.exit(1);
