@echo off
REM Patent Pending Copyright (c) 2025 Xeris Web Co. All rights reserved.
REM XerisCoin Local Alpha v0.1.0 - Windows Launcher
REM Run: Double-click or cmd /c start-local.bat
REM Patent Pending: Triple Consensus Test (Local-Only)

echo === XerisCoin Local Alpha v0.1.0 Starting ===
echo Patent Pending: Local-Only Mode (127.0.0.1)

REM 1. Gen local keypair (if not exists)
if not exist "keypair.json" (
    echo Generating local keypair...
    keypair_gen.exe
) else (
    echo Keypair already exists.
)

REM 2. Init genesis & start node/explorer (background)
echo Initializing genesis ^& node on 127.0.0.1...
start /b xrs-node.exe --local-alpha
set NODE_PID=%errorlevel%
timeout /t 3 /nobreak >nul

REM 3. Airdrop to local wallet (1000 XRS - use mock or extract pubkey)
REM Extract pubkey from keypair.json (simple: assume first run uses mock)
set LOCAL_PUBKEY=LocalPubkey123...  REM Replace with real extraction if needed (e.g., via PowerShell)
REM For real: Use PowerShell to parse JSON, but keep simple
echo Claiming local airdrop...
curl -X POST http://127.0.0.1:4001/airdrop/%LOCAL_PUBKEY%/1000000000000
REM Fallback if curl missing: Use PowerShell
if errorlevel 1 (
    powershell -Command "Invoke-RestMethod -Uri 'http://127.0.0.1:4001/airdrop/LocalPubkey123.../1000000000000' -Method Post"
)

REM 4. Start local miner (background)
echo Starting local miner...
start /b minerd.exe -l

REM 5. Example TX submit
echo Submitting example TX...
call submit_tx.bat

REM 6. Status ^& wait
echo === All Running! ===
echo Node: 127.0.0.1:4000 ^| Explorer: http://127.0.0.1:8081 ^| Miner: Local MH/s
echo Ledger: local-ledger.dat ^| Genesis: xrs-genesis.json
echo Stop: Close windows or Ctrl+C in cmd.
echo View Explorer: Open browser to http://127.0.0.1:8081/blocks
pause
REM Cleanup on exit (optional): taskkill /f /im xrs-node.exe >nul 2>&1