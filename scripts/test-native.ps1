$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
pnpm tauri build --debug --no-bundle --config src-tauri/tauri.smoke.conf.json
$executable = (Resolve-Path src-tauri/target/debug/RVNFlow.exe).Path
$fixtureRoot = Join-Path $env:TEMP ('rvnflow-native-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $fixtureRoot | Out-Null
$oldSmoke = $env:RVNFLOW_NATIVE_SMOKE
$oldDirectory = $env:RVNFLOW_TEST_DATA_DIR
try {
    $env:RVNFLOW_NATIVE_SMOKE = '1'
    foreach ($scenario in @('first-launch', 'restart-running')) {
        $directory = Join-Path $fixtureRoot $scenario
        New-Item -ItemType Directory -Path $directory | Out-Null
        if ($scenario -eq 'restart-running') {
            $saved = Get-Content (Join-Path $fixtureRoot 'first-launch/app-state.json') -Raw | ConvertFrom-Json
            $saved.timer.status = 'running'
            $saved.timer.routineComplete = $false
            $saved.timer.routineCompletedFocus = 0
            $saved.timer.focusSinceLongBreak = 0
            $saved.timer.durationMs = 60000
            $saved.timer.remainingMs = 60000
            $saved.timer.wallDeadlineMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() + 60000
            $saved | ConvertTo-Json -Depth 20 | Set-Content (Join-Path $directory 'app-state.json')
        }
        $env:RVNFLOW_TEST_DATA_DIR = $directory
        $primary = Start-Process -FilePath $executable -WindowStyle Hidden -PassThru
        try {
            $deadline = [DateTime]::UtcNow.AddSeconds(30)
            $ready = Join-Path $directory 'smoke-ready.txt'
            while (!(Test-Path -LiteralPath $ready) -and !$primary.HasExited -and [DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 100; $primary.Refresh() }
            if (!(Test-Path -LiteralPath $ready)) { throw "$scenario did not reach native smoke readiness; inspect $directory" }
            $secondary = Start-Process -FilePath $executable -WindowStyle Hidden -PassThru
            if (!$secondary.WaitForExit(10000)) { Stop-Process -Id $secondary.Id; throw 'Second launch created a lingering process' }
            if (!$primary.WaitForExit(25000)) { throw "$scenario did not finish; inspect $directory" }
            $result = Get-Content (Join-Path $directory 'smoke-result.txt') -Raw
            if ($result -ne 'passed') { throw "$scenario : $result" }
            $state = Get-Content (Join-Path $directory 'app-state.json') -Raw | ConvertFrom-Json
            if ($state.tasks.tasks.Count -ne 1) { throw 'Native restart lost a task' }
            Write-Output "$scenario passed (native modes, minimize/restore, tray hide/show, single instance, saved tasks, automatic 4x50/10 routine)"
        } finally {
            $primary.Refresh()
            if (!$primary.HasExited) { Stop-Process -Id $primary.Id }
        }
    }
    Write-Output "Isolated test diagnostics retained at $fixtureRoot"
} finally {
    $env:RVNFLOW_NATIVE_SMOKE = $oldSmoke
    $env:RVNFLOW_TEST_DATA_DIR = $oldDirectory
}
