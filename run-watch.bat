@echo off
chcp 65001 >nul
echo 启动热重载开发模式...
echo 修改代码后会自动重新编译运行
echo.
set PATH=%PATH%;%USERPROFILE%\.cargo\bin
cargo watch -x run
