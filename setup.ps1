# 批量图片分割工具 - Rust 版本环境设置脚本
# 以管理员身份运行 PowerShell 后执行此脚本

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  批量图片分割工具 - Rust 环境设置" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# 检查是否以管理员身份运行
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")
if (-not $isAdmin) {
    Write-Host "[警告] 建议以管理员身份运行此脚本" -ForegroundColor Yellow
    Write-Host ""
}

# 1. 安装 Rust
Write-Host "[1/4] 检查 Rust 安装..." -ForegroundColor Green
$rustInstalled = $null -ne (Get-Command rustc -ErrorAction SilentlyContinue)

if (-not $rustInstalled) {
    Write-Host "Rust 未安装，正在下载安装程序..." -ForegroundColor Yellow
    
    # 下载 rustup-init
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupPath = "$env:TEMP\rustup-init.exe"
    
    try {
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath -UseBasicParsing
        Write-Host "下载完成，开始安装 Rust..." -ForegroundColor Green
        
        # 运行安装程序
        & $rustupPath -y --default-toolchain stable
        
        # 添加环境变量
        $env:PATH += ";$env:USERPROFILE\.cargo\bin"
        [Environment]::SetEnvironmentVariable("PATH", $env:PATH, "User")
        
        Write-Host "Rust 安装完成！" -ForegroundColor Green
    } catch {
        Write-Host "[错误] 安装失败: $_" -ForegroundColor Red
        Write-Host "请手动访问 https://rustup.rs/ 安装" -ForegroundColor Yellow
        exit 1
    }
} else {
    Write-Host "Rust 已安装: $(rustc --version)" -ForegroundColor Green
}

Write-Host ""

# 2. 安装 Visual Studio Build Tools
Write-Host "[2/4] 检查 Visual Studio Build Tools..." -ForegroundColor Green
$vsInstalled = Test-Path "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
$vsCommunity = Test-Path "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"

if (-not ($vsInstalled -or $vsCommunity)) {
    Write-Host "Visual Studio Build Tools 未安装" -ForegroundColor Yellow
    Write-Host "正在安装...这可能需要几分钟时间" -ForegroundColor Yellow
    
    try {
        winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended" --accept-package-agreements --accept-source-agreements
        Write-Host "Visual Studio Build Tools 安装完成！" -ForegroundColor Green
    } catch {
        Write-Host "[警告] 自动安装失败，请手动安装:" -ForegroundColor Yellow
        Write-Host "winget install Microsoft.VisualStudio.2022.BuildTools --override `"--wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended`"" -ForegroundColor Cyan
    }
} else {
    Write-Host "Visual Studio 环境已存在" -ForegroundColor Green
}

Write-Host ""

# 3. 安装依赖
Write-Host "[3/4] 安装项目依赖..." -ForegroundColor Green
$env:PATH += ";$env:USERPROFILE\.cargo\bin"

try {
    cargo fetch
    Write-Host "依赖安装完成！" -ForegroundColor Green
} catch {
    Write-Host "[错误] 依赖安装失败: $_" -ForegroundColor Red
}

Write-Host ""

# 4. 构建项目
Write-Host "[4/4] 构建项目..." -ForegroundColor Green

try {
    # 加载 VS 环境
    if (Test-Path "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat") {
        & "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
    } elseif (Test-Path "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat") {
        & "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
    }
    
    cargo build --release
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "==========================================" -ForegroundColor Green
        Write-Host "  构建成功！" -ForegroundColor Green
        Write-Host "==========================================" -ForegroundColor Green
        Write-Host "可执行文件: .\target\release\batch-image-splitter.exe" -ForegroundColor Cyan
        Write-Host ""
        
        $runNow = Read-Host "是否立即运行程序? (Y/n)"
        if ($runNow -eq "Y" -or $runNow -eq "y" -or $runNow -eq "") {
            & ".\target\release\batch-image-splitter.exe"
        }
    } else {
        throw "构建失败"
    }
} catch {
    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Red
    Write-Host "  构建失败！" -ForegroundColor Red
    Write-Host "==========================================" -ForegroundColor Red
    Write-Host "错误信息: $_" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "请尝试以下步骤:" -ForegroundColor Cyan
    Write-Host "1. 重启终端或电脑" -ForegroundColor White
    Write-Host "2. 运行 .\build.bat 手动构建" -ForegroundColor White
    Write-Host "3. 检查 Visual Studio Build Tools 是否正确安装" -ForegroundColor White
}

Write-Host ""
Write-Host "按任意键退出..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
