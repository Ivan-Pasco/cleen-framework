# Frame Framework Build Script (Windows PowerShell)
# Builds Frame CLI and all runtime components

param(
	[string]$Target = "",
	[switch]$Debug = $false,
	[switch]$Verbose = $false,
	[switch]$CliOnly = $false,
	[switch]$WasmOnly = $false,
	[switch]$Help = $false
)

$ErrorActionPreference = "Stop"

# Colors for output
$InfoColor = "Cyan"
$SuccessColor = "Green"
$WarningColor = "Yellow"
$ErrorColor = "Red"

# Paths
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$BuildDir = Join-Path $ProjectRoot "build"

# Functions
function Write-Info {
	param([string]$Message)
	Write-Host "[INFO] $Message" -ForegroundColor $InfoColor
}

function Write-Success {
	param([string]$Message)
	Write-Host "[SUCCESS] $Message" -ForegroundColor $SuccessColor
}

function Write-Warning {
	param([string]$Message)
	Write-Host "[WARNING] $Message" -ForegroundColor $WarningColor
}

function Write-Error {
	param([string]$Message)
	Write-Host "[ERROR] $Message" -ForegroundColor $ErrorColor
}

function Show-Help {
	Write-Host "Usage: .\build-all.ps1 [OPTIONS]"
	Write-Host ""
	Write-Host "Options:"
	Write-Host "  -Target <triple>    Target triple (e.g., x86_64-pc-windows-msvc)"
	Write-Host "  -Debug              Build in debug mode instead of release"
	Write-Host "  -Verbose            Verbose output"
	Write-Host "  -CliOnly            Build only the CLI binary"
	Write-Host "  -WasmOnly           Build only WASM components"
	Write-Host "  -Help               Show this help message"
	exit 0
}

function Test-Rust {
	if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
		Write-Error "Cargo not found. Please install Rust from https://rustup.rs/"
		exit 1
	}

	$rustVersion = cargo --version
	Write-Info "Using $rustVersion"
}

function Test-WasmTarget {
	$targets = rustup target list --installed
	if ($targets -notmatch "wasm32-unknown-unknown") {
		Write-Warning "WASM target not installed. Installing..."
		rustup target add wasm32-unknown-unknown
	}
}

function Build-Cli {
	Write-Info "Building Frame CLI..."

	Set-Location $ProjectRoot

	$buildMode = if ($Debug) { "" } else { "--release" }
	$targetArg = if ($Target) { "--target $Target" } else { "" }
	$verboseArg = if ($Verbose) { "--verbose" } else { "" }

	$cargoArgs = "build $buildMode --bin frame $targetArg $verboseArg".Trim()

	if ($Target) {
		Write-Info "Building for target: $Target"
	}

	try {
		Invoke-Expression "cargo $cargoArgs"
		Write-Success "Frame CLI built successfully"

		# Show binary location
		$buildType = if ($Debug) { "debug" } else { "release" }
		$binaryPath = if ($Target) {
			Join-Path $ProjectRoot "target\$Target\$buildType\frame.exe"
		} else {
			Join-Path $ProjectRoot "target\$buildType\frame.exe"
		}

		if (Test-Path $binaryPath) {
			Write-Info "Binary location: $binaryPath"
			$size = (Get-Item $binaryPath).Length / 1MB
			Write-Info "Binary size: $($size.ToString('F2')) MB"
		}
	}
	catch {
		Write-Error "Failed to build Frame CLI"
		exit 1
	}
}

function Build-WasmComponents {
	Write-Info "Building WASM runtime components..."

	Set-Location $ProjectRoot

	# Ensure WASM target is installed
	Test-WasmTarget

	$buildMode = if ($Debug) { "" } else { "--release" }
	$verboseArg = if ($Verbose) { "--verbose" } else { "" }

	$cargoArgs = "build $buildMode --target wasm32-unknown-unknown -p host-bridge $verboseArg".Trim()

	try {
		Invoke-Expression "cargo $cargoArgs"
		Write-Success "WASM components built successfully"

		# Show WASM locations
		$buildType = if ($Debug) { "debug" } else { "release" }
		$wasmDir = Join-Path $ProjectRoot "target\wasm32-unknown-unknown\$buildType"

		if (Test-Path $wasmDir) {
			Write-Info "WASM modules location: $wasmDir"
			Get-ChildItem -Path $wasmDir -Filter "*.wasm" | ForEach-Object {
				$size = $_.Length / 1KB
				Write-Host "  - $($_.Name) ($($size.ToString('F2')) KB)"
			}
		}
	}
	catch {
		Write-Error "Failed to build WASM components"
		exit 1
	}
}

function Optimize-Wasm {
	Write-Info "Optimizing WASM modules..."

	$buildType = if ($Debug) { "debug" } else { "release" }
	$wasmDir = Join-Path $ProjectRoot "target\wasm32-unknown-unknown\$buildType"

	if (Get-Command wasm-opt -ErrorAction SilentlyContinue) {
		Get-ChildItem -Path $wasmDir -Filter "*.wasm" | ForEach-Object {
			Write-Info "Optimizing $($_.Name)..."
			$optimized = "$($_.FullName).opt"
			wasm-opt -Oz $_.FullName -o $optimized
			Move-Item -Path $optimized -Destination $_.FullName -Force
		}
		Write-Success "WASM optimization complete"
	}
	else {
		Write-Warning "wasm-opt not found. Install binaryen for WASM optimization"
		Write-Warning "  Download from: https://github.com/WebAssembly/binaryen/releases"
	}
}

function New-BuildManifest {
	Write-Info "Creating build manifest..."

	$buildType = if ($Debug) { "debug" } else { "release" }
	$targetName = if ($Target) { $Target } else { "native" }

	if (-not (Test-Path $BuildDir)) {
		New-Item -ItemType Directory -Path $BuildDir | Out-Null
	}

	$manifest = @{
		version = "1.0.0"
		build_type = $buildType
		target = $targetName
		built_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
		components = @{
			cli = @{
				path = "target\$targetName\$buildType\frame.exe"
				type = "binary"
			}
			runtime = @{
				host_bridge = @{
					path = "target\wasm32-unknown-unknown\$buildType\host_bridge.wasm"
					type = "wasm"
				}
			}
		}
	}

	$manifestPath = Join-Path $BuildDir "manifest.json"
	$manifest | ConvertTo-Json -Depth 10 | Set-Content -Path $manifestPath

	Write-Success "Build manifest created: $manifestPath"
}

# Main execution
function Main {
	if ($Help) {
		Show-Help
	}

	Write-Info "Frame Framework Build Script"
	Write-Info "============================"
	Write-Host ""

	Test-Rust

	if (-not $WasmOnly) {
		Build-Cli
		Write-Host ""
	}

	if (-not $CliOnly) {
		Build-WasmComponents
		Write-Host ""

		# Optimize WASM in release mode
		if (-not $Debug) {
			Optimize-Wasm
			Write-Host ""
		}
	}

	New-BuildManifest

	Write-Host ""
	Write-Success "Build complete!"
	Write-Info "Run '.\target\release\frame.exe --version' to verify installation"
}

Main
