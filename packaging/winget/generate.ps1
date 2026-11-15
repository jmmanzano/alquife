param(
    [Parameter(Mandatory)][string]$Version,
    [Parameter(Mandatory)][string]$ZipPath,
    [Parameter(Mandatory)][string]$OutputDir
)
$ErrorActionPreference = 'Stop'
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw 'Expected a stable version x.y.z' }
$hash = (Get-FileHash -Algorithm SHA256 $ZipPath).Hash
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$id = 'jmmanzano.Alquife'
$url = "https://github.com/jmmanzano/alquife/releases/download/v$Version/alquife-windows-x86_64.zip"
@"
PackageIdentifier: $id
PackageVersion: $Version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
"@ | Set-Content -Encoding utf8 "$OutputDir/$id.yaml"
@"
PackageIdentifier: $id
PackageVersion: $Version
PackageLocale: en-US
Publisher: jmmanzano
PublisherUrl: https://github.com/jmmanzano
PackageName: Alquife
PackageUrl: https://github.com/jmmanzano/alquife
License: MIT
LicenseUrl: https://github.com/jmmanzano/alquife/blob/master/LICENSE
ShortDescription: A terminal-based Subsonic music client with FFmpeg playback.
ManifestType: defaultLocale
ManifestVersion: 1.6.0
"@ | Set-Content -Encoding utf8 "$OutputDir/$id.locale.en-US.yaml"
@"
PackageIdentifier: $id
PackageVersion: $Version
InstallerType: zip
NestedInstallerType: portable
NestedInstallerFiles:
- RelativeFilePath: alquife.exe
  PortableCommandAlias: alquife
Installers:
- Architecture: x64
  InstallerUrl: $url
  InstallerSha256: $hash
ManifestType: installer
ManifestVersion: 1.6.0
"@ | Set-Content -Encoding utf8 "$OutputDir/$id.installer.yaml"
