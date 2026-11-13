// Per-friend sharing controls: one sheet over /api/v1/locations/shares.
// Directional: this edits only what I share with the selected friend.

const $ = (id) => document.getElementById(id);

export function initSharing({ api, setStatus, getUser, onChanged }) {
  const sheet = $("share-sheet");
  let friendship = null;
  let current = null;

  function friendName(f) {
    const me = getUser?.()?.id;
    return f.requester_id === me ? f.receiver_name : f.requester_name;
  }

  async function open(f) {
    friendship = f;
    const me = getUser?.()?.id;
    const isOutgoing = f.requester_id === me;
    $("share-title").textContent = `Share with ${friendName(f)}`;
    $("share-subtitle").textContent = isOutgoing
      ? "You decide exactly what this friend can see."
      : "They can already see what you allow below.";
    $("share-incoming").textContent =
      "What they share with you is controlled by them and appears on the map only while active.";
    try {
      const shares = await api("/api/v1/locations/shares");
      current = (Array.isArray(shares) ? shares : []).find(
        (s) => s.friendship_id === f.id && (s.direction === "to_friend" || s.direction === "mutual"),
      );
      const active = Boolean(current?.is_active);
      $("share-enabled").checked = active;
      $("share-precision").value = current?.precision_level || "exact";
      $("share-duration").value = current?.expires_at ? "24" : "0";
      $("share-revoke").classList.toggle("hidden", !current);
      $("share-sheet").classList.remove("hidden");
      $("place-sheet").classList.add("hidden");
      $("route-panel").classList.add("hidden");
    } catch (err) {
      setStatus(err.message, true);
    }
  }

  async function save() {
    if (!friendship) return;
    const enabled = $("share-enabled").checked;
    const body = {
      friendship_id: friendship.id,
      direction: "to_friend",
      precision_level: $("share-precision").value,
      duration_hours: Number($("share-duration").value),
      is_active: enabled,
    };
    try {
      if (current) {
        await api(`/api/v1/locations/shares/${current.id}`, { method: "PATCH", body });
      } else if (enabled) {
        await api("/api/v1/locations/shares", { method: "POST", body });
      }
      setStatus(enabled ? "Sharing updated." : "Sharing stopped.");
      await onChanged?.();
      close();
    } catch (err) {
      setStatus(err.message, true);
    }
  }

  async function revoke() {
    if (!current) {
      close();
      return;
    }
    try {
      await api(`/api/v1/locations/shares/${current.id}`, {
        method: "PATCH",
        body: { is_active: false },
      });
      setStatus("Sharing stopped.");
      await onChanged?.();
      close();
    } catch (err) {
      setStatus(err.message, true);
    }
  }

  function close() {
    sheet.classList.add("hidden");
    friendship = null;
    current = null;
  }

  $("share-save").addEventListener("click", save);
  $("share-revoke").addEventListener("click", revoke);
  $("share-close").addEventListener("click", close);

  return { open, close };
}
