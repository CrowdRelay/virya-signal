#!/usr/bin/env bash
set -euo pipefail

ndk_version="${NDK_VERSION:?NDK_VERSION is required}"
sdk_root="${ANDROID_SDK_ROOT:?ANDROID_SDK_ROOT is required}"
packages=()

[[ -d "$sdk_root/platforms/android-36" ]] || packages+=("platforms;android-36")
[[ -d "$sdk_root/build-tools/36.0.0" ]] || packages+=("build-tools;36.0.0")
[[ -d "$sdk_root/ndk/$ndk_version" ]] || packages+=("ndk;$ndk_version")

if ((${#packages[@]})); then
  yes | sdkmanager --licenses >/dev/null || true
  sdkmanager "${packages[@]}"
else
  echo "Android API 36, Build Tools 36.0.0 and NDK $ndk_version are already installed."
fi

{
  echo "ANDROID_NDK_HOME=$sdk_root/ndk/$ndk_version"
  echo "NDK_HOME=$sdk_root/ndk/$ndk_version"
} >> "${GITHUB_ENV:?GITHUB_ENV is required}"
