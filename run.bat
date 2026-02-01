@echo off
chcp 65001 >nul
echo 正在启动 Batch Image Splitter...
cd /d "%~dp0"
cargo run
echo.
echo 按任意键退出...
pause >nul
