@echo off
chcp 65001 >nul
echo ==========================================
echo  批量图片分割工具 - Rust 版本构建脚本
echo ==========================================
echo.

:: 检查 Rust 是否安装
where rustc >nul 2>nul
if %errorlevel% neq 0 (
    echo [错误] 未检测到 Rust，请先安装 Rust
    echo 访问 https://rustup.rs/ 安装
    pause
    exit /b 1
)

echo [1/3] 检查 Rust 版本...
rustc --version
cargo --version
echo.

:: 设置 VS 环境（如果安装了 VS Build Tools）
echo [2/3] 设置编译环境...
if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" (
    call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
    echo 已加载 Visual Studio Build Tools 环境
) else if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" (
    call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
    echo 已加载 Visual Studio Community 环境
) else (
    echo [警告] 未找到 Visual Studio 环境，尝试使用默认设置
    echo 如果编译失败，请安装 Visual Studio Build Tools:
    echo winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
)
echo.

:: 构建项目
echo [3/3] 开始构建...
cargo build --release

if %errorlevel% equ 0 (
    echo.
    echo ==========================================
    echo  构建成功！
    echo ==========================================
    echo 可执行文件位置: target\release\batch-image-splitter.exe
    echo.
    echo 按任意键运行程序...
    pause >nul
    target\release\batch-image-splitter.exe
) else (
    echo.
    echo ==========================================
    echo  构建失败！
    echo ==========================================
    echo 请检查错误信息并安装必要的依赖
    pause
)
