# Remove cut screen fns from src/ui.rs (ranges 1-indexed).
$ranges = '407,495;537,807;809,923;924,1082;1083,1202;1203,1291;1292,1358;1359,1419;1420,1473;1474,1574' -split ';'
$L = Get-Content src\ui.rs -Encoding utf8
$cut = @{}
foreach ($r in $ranges) {
    $p = $r -split ','
    for ($i = [int]$p[0]; $i -le [int]$p[1]; $i++) { $cut[$i - 1] = $true }
}
$keep = @()
for ($i = 0; $i -lt $L.Count; $i++) { if (-not $cut.ContainsKey($i)) { $keep += $L[$i] } }
$keep -join "`n" | Out-File -Encoding utf8 -NoNewline src\ui.rs
Write-Host ("kept " + $keep.Count + " of " + $L.Count + " lines")
