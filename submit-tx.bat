@echo off
REM Patent Pending Copyright (c) 2025 Xeris Web Co. All rights reserved.
REM Local TX Submitter - Hits 127.0.0.1:4001

REM Generate TX JSON (mock or via cargo if in dev; for release, use static demo)
set TX_JSON={"tx": "base64_mock_tx_here"}  REM Replace with real gen (e.g., xrs-node.exe --bin submit_transaction --local)

REM Submit to local
echo Submitting local TX...
curl -X POST http://127.0.0.1:4001/submit_transaction -H "Content-Type: application/json" -d "%TX_JSON%" > local_transaction.json
if errorlevel 1 (
    powershell -Command "$body = @{tx='base64_mock_tx_here'}; Invoke-RestMethod -Uri 'http://127.0.0.1:4001/submit_transaction' -Method Post -Body $body -ContentType 'application/json'"
)
echo Local TX Submitted - Check local_transaction.json ^| Patent Pending
pause