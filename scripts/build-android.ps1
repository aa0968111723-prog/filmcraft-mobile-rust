param(
    [Parameter(Mandatory = $true)][string]$SdkRoot,
    [Parameter(Mandatory = $true)][string]$JavaHome,
    [string]$NdkVersion = '27.2.12479018',
    [string[]]$Abis = @('arm64-v8a', 'x86_64')
)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path $PSScriptRoot -Parent
[xml]$taskManifest = Get-Content -LiteralPath (Join-Path $taskRoot 'packaging/android/AndroidManifest.xml') -Raw
$taskVersion = $taskManifest.manifest.GetAttribute('versionName', 'http://schemas.android.com/apk/res/android')
if ($taskVersion -notmatch '^[0-9A-Za-z.-]+$') { throw 'Invalid Android version name.' }
$taskFontHash = (Get-FileHash -LiteralPath (Join-Path $taskRoot 'apps/mobile/assets/fonts/NotoSansCJKtc-Regular.otf') -Algorithm SHA256).Hash
if ($taskFontHash -ne 'dce08bd4fd91aa8aa76ed8fea4b694c2dfb8550f67871e326843212ddbeb88b4') {
    throw 'Bundled Chinese font does not match the pinned upstream file.'
}
$taskOutput = Join-Path $taskRoot 'target/android'
$taskAssets = Join-Path $taskOutput 'assets'
$taskSigning = Join-Path $taskRoot '.local/signing'
$taskBuildTools = Join-Path $SdkRoot 'build-tools/36.0.0'
$env:JAVA_HOME = $JavaHome
$env:PATH = (Join-Path $JavaHome 'bin') + [IO.Path]::PathSeparator + $env:PATH
$env:ANDROID_NDK_HOME = Join-Path $SdkRoot ('ndk/' + $NdkVersion)
$taskPreviousEncodedFlags = $env:CARGO_ENCODED_RUSTFLAGS
$env:CARGO_ENCODED_RUSTFLAGS = @(
    '-C', 'link-arg=-Wl,-z,max-page-size=16384',
    "--remap-path-prefix=$env:USERPROFILE=/build-user",
    "--remap-path-prefix=$($env:USERPROFILE.Replace('\','/'))=/build-user",
    "--remap-path-prefix=$taskRoot=/filmcraft-mobile",
    "--remap-path-prefix=$($taskRoot.Replace('\','/'))=/filmcraft-mobile"
) -join [char]31
$taskTargets = @{ 'arm64-v8a' = 'aarch64-linux-android'; 'x86_64' = 'x86_64-linux-android' }
foreach ($taskAbi in $Abis) {
    if (-not $taskTargets.ContainsKey($taskAbi)) { throw "Unsupported ABI: $taskAbi" }
}
foreach ($taskPath in @($taskOutput, $taskAssets, $taskSigning)) {
    New-Item -ItemType Directory -Path $taskPath -Force | Out-Null
}
Push-Location $taskRoot
try {
    $taskNdkArgs = @('ndk', '--platform', '26')
    foreach ($taskAbi in $Abis) { $taskNdkArgs += @('-t', $taskAbi) }
    & cargo @taskNdkArgs build -p filmcraft-mobile-app --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'Rust Android build failed.' }

    # Include license texts from the resolved Android graph, plus font license sidecars.
    $taskMetadataText = & cargo metadata --format-version 1 --locked --filter-platform aarch64-linux-android
    if ($LASTEXITCODE -ne 0) { throw 'Could not resolve third-party notices.' }
    $taskMetadata = $taskMetadataText | ConvertFrom-Json
    $taskNotice = [Text.StringBuilder]::new()
    [void]$taskNotice.AppendLine("影片工坊 $taskVersion — 第三方授權原文")
    foreach ($taskPackage in ($taskMetadata.packages | Sort-Object name, version)) {
        [void]$taskNotice.AppendLine("`n=== $($taskPackage.name) $($taskPackage.version) ===")
        [void]$taskNotice.AppendLine("License expression: $($taskPackage.license)")
        [void]$taskNotice.AppendLine("Source: $($taskPackage.repository)")
        $taskPackageDir = Split-Path $taskPackage.manifest_path -Parent
        $taskLicenseFiles = @(Get-ChildItem -LiteralPath $taskPackageDir -File -Recurse |
            Where-Object { ($_.Name -match '(LICENSE|LICENCE|COPYING|NOTICE|COPYRIGHT|OFL|UFL)' -or ($_.Extension -eq '.txt' -and $_.DirectoryName -match '[/\\]fonts([/\\]|$)')) -and $_.Length -lt 200000 })
        foreach ($taskLicenseFile in $taskLicenseFiles) {
            [void]$taskNotice.AppendLine("--- $($taskLicenseFile.Name) ---")
            [void]$taskNotice.AppendLine([IO.File]::ReadAllText($taskLicenseFile.FullName))
        }
    }
    $taskNoticesPath = Join-Path $taskAssets 'THIRD-PARTY-NOTICES.txt'
    [IO.File]::WriteAllText($taskNoticesPath, $taskNotice.ToString(), [Text.UTF8Encoding]::new($false))
    Copy-Item -LiteralPath (Join-Path $taskRoot 'LICENSE-MIT') -Destination $taskAssets
    Copy-Item -LiteralPath (Join-Path $taskRoot 'LICENSE-APACHE') -Destination $taskAssets

    $taskUnsigned = Join-Path $taskOutput 'unsigned.apk'
    & (Join-Path $taskBuildTools 'aapt2.exe') link --manifest (Join-Path $taskRoot 'packaging/android/AndroidManifest.xml') -I (Join-Path $SdkRoot 'platforms/android-36/android.jar') -A $taskAssets -o $taskUnsigned
    if ($LASTEXITCODE -ne 0) { throw 'Android manifest packaging failed.' }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $taskArchive = [IO.Compression.ZipFile]::Open($taskUnsigned, [IO.Compression.ZipArchiveMode]::Update)
    try {
        foreach ($taskAbi in $Abis) {
            $taskLibrary = Join-Path $taskOutput ("libfilmcraft_mobile_app-$taskAbi.so")
            Copy-Item -LiteralPath (Join-Path $taskRoot ("target/" + $taskTargets[$taskAbi] + '/release/libfilmcraft_mobile_app.so')) -Destination $taskLibrary
            & (Join-Path $env:ANDROID_NDK_HOME 'toolchains/llvm/prebuilt/windows-x86_64/bin/llvm-strip.exe') --strip-unneeded $taskLibrary
            if ($LASTEXITCODE -ne 0) { throw 'Stripping native library failed.' }
            $taskNativeText = [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes($taskLibrary))
            if ($taskNativeText.Contains($env:USERPROFILE) -or $taskNativeText.Contains($env:USERPROFILE.Replace('\','/'))) {
                throw 'Native library still contains a local user-profile path.'
            }
            [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($taskArchive, $taskLibrary, "lib/$taskAbi/libfilmcraft_mobile_app.so", [IO.Compression.CompressionLevel]::Optimal) | Out-Null
        }
    } finally { $taskArchive.Dispose() }
    $taskAligned = Join-Path $taskOutput 'aligned.apk'
    & (Join-Path $taskBuildTools 'zipalign.exe') -p -f 4 $taskUnsigned $taskAligned
    if ($LASTEXITCODE -ne 0) { throw 'APK alignment failed.' }

    # Local preview identity persists between builds. Never commit these files.
    $taskKey = Join-Path $taskSigning 'preview.jks'
    $taskPasswordFile = Join-Path $taskSigning 'preview.password'
    if (-not (Test-Path -LiteralPath $taskKey)) {
        if (Test-Path -LiteralPath $taskPasswordFile) { throw 'Password exists without signing key; inspect local signing files.' }
        $taskPasswordBytes = [byte[]]::new(32)
        [Security.Cryptography.RandomNumberGenerator]::Fill($taskPasswordBytes)
        $env:FC_SIGNING_PASSWORD = [Convert]::ToBase64String($taskPasswordBytes)
        [IO.File]::WriteAllText($taskPasswordFile, $env:FC_SIGNING_PASSWORD)
        & keytool -genkeypair -keystore $taskKey -alias preview -keyalg RSA -keysize 3072 -validity 3650 -dname 'CN=FilmCraft Mobile Preview' -storepass:env FC_SIGNING_PASSWORD -keypass:env FC_SIGNING_PASSWORD
        if ($LASTEXITCODE -ne 0) { throw 'Could not create local preview signing identity.' }
    } else {
        $env:FC_SIGNING_PASSWORD = [IO.File]::ReadAllText($taskPasswordFile)
    }
    $taskApk = Join-Path $taskOutput "filmcraft-mobile-$taskVersion-universal.apk"
    & (Join-Path $taskBuildTools 'apksigner.bat') sign --ks $taskKey --ks-key-alias preview --ks-pass env:FC_SIGNING_PASSWORD --key-pass env:FC_SIGNING_PASSWORD --out $taskApk $taskAligned
    if ($LASTEXITCODE -ne 0) { throw 'APK signing failed.' }
    & (Join-Path $taskBuildTools 'apksigner.bat') verify --verbose $taskApk
    if ($LASTEXITCODE -ne 0) { throw 'APK signature verification failed.' }
    $taskHash = (Get-FileHash -LiteralPath $taskApk -Algorithm SHA256).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText((Join-Path $taskOutput 'SHA256SUMS.txt'), "$taskHash  $([IO.Path]::GetFileName($taskApk))`n", [Text.UTF8Encoding]::new($false))
    Copy-Item -LiteralPath $taskNoticesPath -Destination (Join-Path $taskOutput 'THIRD-PARTY-NOTICES.txt')
    Write-Output "APK ready: $taskApk"
} finally {
    Remove-Item Env:FC_SIGNING_PASSWORD -ErrorAction SilentlyContinue
    $env:CARGO_ENCODED_RUSTFLAGS = $taskPreviousEncodedFlags
    Pop-Location
}
