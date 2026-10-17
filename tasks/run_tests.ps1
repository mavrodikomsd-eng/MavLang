$root = "C:\1\аап"
$tasksDir = "$root\tasks"
$source = "$root\source\main.mavl"
$exe = "$root\target\debug\mavapp.exe"

$testCases = @(
    @{ name="01_fizzbuzz"; mem="borrow"; stdin=$null; args=$null; expected="1`n2`nfizz`n4`nbuzz`nfizz`n7`n8`nfizz`nbuzz`n11`nfizz`n13`n14`nfizzbuzz" },
    @{ name="02_csvsum"; mem="borrow"; stdin=$null; args=$null; expected="60" },
    @{ name="03_wordcount"; mem="borrow"; stdin=$null; args=$null; expected="3`nHELLO" },
    @{ name="04_fileroundtrip"; mem="borrow"; stdin=$null; args=$null; expected="50" },
    @{ name="05_args_none"; mem="borrow"; stdin=$null; args=$null; expected="none" },
    @{ name="05_args_yo"; mem="borrow"; stdin=$null; args="yo"; expected="yo" },
    @{ name="06_counter"; mem="borrow"; stdin=$null; args=$null; expected="5" },
    @{ name="07_fib"; mem="borrow"; stdin=$null; args=$null; expected="55" },
    @{ name="08_normalize"; mem="borrow"; stdin=$null; args=$null; expected="rust lang" },
    @{ name="09_borrowseries"; mem="borrow"; stdin=$null; args=$null; expected="5`n6" },
    @{ name="10_ptroffset"; mem="manual"; stdin=$null; args=$null; expected="99" },
    @{ name="11_inputmath"; mem="borrow"; stdin="21"; args=$null; expected="42" },
    @{ name="12_repconcat"; mem="borrow"; stdin=$null; args=$null; expected="ababab" }
)

$results = @()

foreach ($tc in $testCases) {
    if ($tc.name -eq "05_args_yo") {
        $mavlFile = "$tasksDir\05_args.mavl"
    } else {
        $mavlFile = "$tasksDir\$($tc.name).mavl"
    }
    
    # Copy to source
    Copy-Item -LiteralPath $mavlFile -Destination $source -Force
    
    # Build
    $buildProc = & mavc "build --mem=$($tc.mem)" 2>&1
    $buildOk = $LASTEXITCODE -eq 0
    
    $runOutput = ""
    $runOk = $false
    if ($buildOk) {
        if ($tc.stdin -ne $null) {
            $runProc = $tc.stdin | & $exe 2>&1
            $runOutput = (($runProc | Out-String)).Trim()
            $runOk = $LASTEXITCODE -eq 0
        } else {
            $runArgs = @()
            if ($tc.args -ne $null) { $runArgs += $tc.args }
            $runProc = & $exe @runArgs 2>&1
            $runOutput = (($runProc | Out-String)).Trim()
            $runOk = $LASTEXITCODE -eq 0
        }
    }
    
    $pass = $buildOk -and $runOk -and ($runOutput -eq $tc.expected)
    
    $results += [pscustomobject]@{
        Task = $tc.name
        Mem = $tc.mem
        BuildOK = $buildOk
        RunOK = $runOk
        RunOutput = $runOutput
        Expected = $tc.expected
        PASS = $pass
    }
}

# Restore original main.mavl
Copy-Item -LiteralPath "$root\source\main.mavl.bak" -Destination $source -Force

# Print results
Write-Host "=== РЕЗУЛЬТАТЫ ==="
$results | Format-Table -AutoSize
$passCount = ($results | Where-Object { $_.PASS }).Count
Write-Host "`nИтого: $passCount / 12 прошли полностью"
