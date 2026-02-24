param(
  [Parameter(Mandatory = $true)]
  [string]$Tag
)

$ErrorActionPreference = 'Stop'
$versionRaw = $Tag.TrimStart('v')
$parts = $versionRaw -split '[^0-9]+'
$nums = @($parts | Where-Object { $_ -ne '' })
while ($nums.Count -lt 3) { $nums += '0' }
$msiVersion = "$($nums[0]).$($nums[1]).$($nums[2])"

$archRaw = ($env:RUNNER_ARCH ?? 'X64').ToLowerInvariant()
$arch = if ($archRaw -eq 'x64') { 'x64' } elseif ($archRaw -eq 'arm64') { 'arm64' } else { $archRaw }
$is64Bit = @('x64', 'arm64') -contains $arch
$wixPlatform = if ($is64Bit) { $arch } else { 'x86' }
$programFilesFolder = if ($is64Bit) { 'ProgramFiles64Folder' } else { 'ProgramFilesFolder' }
$componentWin64 = if ($is64Bit) { 'yes' } else { 'no' }

$outDir = "release-assets"
$wxsPath = Join-Path $outDir "retaia-agent.wxs"
$objPath = Join-Path $outDir "retaia-agent.wixobj"
$msiPath = Join-Path $outDir "retaia-agent-$Tag-windows-$arch.msi"
$binDir = (Resolve-Path "target/release").Path
$iconPath = (Resolve-Path "assets/icon/retaia-logo.ico").Path

New-Item -ItemType Directory -Path $outDir -Force | Out-Null

@"
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product
    Id="*"
    Name="Retaia Agent"
    Language="1033"
    Version="$msiVersion"
    Manufacturer="Retaia"
    UpgradeCode="8B4AEE21-A9C4-45E8-B2A2-86A1272AE947">
    <Package InstallerVersion="500" Compressed="yes" InstallScope="perMachine" Platform="$wixPlatform" />
    <MajorUpgrade DowngradeErrorMessage="A newer version of Retaia Agent is already installed." />
    <MediaTemplate />
    <Icon Id="retaiaAppIcon" SourceFile="$iconPath" />
    <Property Id="ARPPRODUCTICON" Value="retaiaAppIcon" />

    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="$programFilesFolder">
        <Directory Id="INSTALLFOLDER" Name="RetaiaAgent" />
      </Directory>
    </Directory>

    <DirectoryRef Id="INSTALLFOLDER">
      <Component Id="cmpAgentCtl" Guid="13D43666-D83A-4067-BB20-B2A638D8D0E6" Win64="$componentWin64">
        <File Id="filAgentCtl" Source="$binDir\\agentctl.exe" KeyPath="yes" />
      </Component>
      <Component Id="cmpAgentRuntime" Guid="B15FA8E3-8607-4A71-8F8B-8125D784F315" Win64="$componentWin64">
        <File Id="filAgentRuntime" Source="$binDir\\agent-runtime.exe" KeyPath="yes" />
      </Component>
      <Component Id="cmpDesktopShell" Guid="D6CB1AB2-5039-4A0E-9355-D61545507183" Win64="$componentWin64">
        <File Id="filDesktopShell" Source="$binDir\\agent-desktop-shell.exe" KeyPath="yes" />
      </Component>
    </DirectoryRef>

    <Feature Id="MainFeature" Title="Retaia Agent" Level="1">
      <ComponentRef Id="cmpAgentCtl" />
      <ComponentRef Id="cmpAgentRuntime" />
      <ComponentRef Id="cmpDesktopShell" />
    </Feature>
  </Product>
</Wix>
"@ | Set-Content -Path $wxsPath -Encoding UTF8

candle.exe -nologo -out $objPath $wxsPath
if ($LASTEXITCODE -ne 0) {
  throw "candle.exe failed with exit code $LASTEXITCODE"
}
light.exe -nologo -out $msiPath $objPath
if ($LASTEXITCODE -ne 0) {
  throw "light.exe failed with exit code $LASTEXITCODE"
}

Write-Host "Built: $msiPath"
