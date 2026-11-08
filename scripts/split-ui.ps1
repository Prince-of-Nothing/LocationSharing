$L = Get-Content src\ui.rs -Encoding utf8
New-Item -ItemType Directory -Force -Path src\ui | Out-Null

function Cut($a, $b, $name, $title) {
    $body = $L[($a - 1)..($b - 1)]
    $body = $body | ForEach-Object { $_ -replace '^([ ]+)fn draw_', '$1pub fn draw_' }
    $nl = [Environment]::NewLine
    $out = "//!" + " " + $title + $nl + "use eframe::egui::{self, Color32, Direction, RichText};" + $nl + "use super::{PendingAction, PinpointAppUI};" + $nl + $nl + "impl PinpointAppUI {" + $nl
    $out += ($body -join $nl) + $nl + "}" + $nl
    $out | Out-File -Encoding utf8 -NoNewline "src\ui\$name"
    Write-Host "wrote $name ($a..$b)"
}

Cut 407 495 "auth.rs" "Auth screens (login / register)"
Cut 537 807 "map.rs" "Map screen"
Cut 809 923 "friends.rs" "Friends screen"
Cut 924 1082 "sharing.rs" "Sharing screen"
Cut 1083 1202 "admin.rs" "Admin + password screens"
Cut 1203 1291 "places.rs" "Places screen (routes / POIs / reverse)"
Cut 1292 1358 "bookmarks.rs" "Saved places screen"
Cut 1359 1419 "trips.rs" "Trips and check-ins screen"
Cut 1420 1473 "emergency.rs" "Emergency screen"
Cut 1474 1574 "profile.rs" "Profile and settings screen (incl. MFA)"
Write-Host "all screens written"
