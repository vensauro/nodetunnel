@echo off
cd ..
cargo build -p nt_client
if not exist "addons\nodetunnel\bin" mkdir "addons\nodetunnel\bin"
if exist target\debug\nt_client.dll (
    copy /Y target\debug\nt_client.dll addons\nodetunnel\bin\
)