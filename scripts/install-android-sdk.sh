#!/usr/bin/env bash
set -euo pipefail

ndk_version="${NDK_VERSION:?NDK_VERSION is required}"
build_tools_version="${ANDROID_BUILD_TOOLS_VERSION:-36.0.0}"
sdk_root="${ANDROID_SDK_ROOT:?ANDROID_SDK_ROOT is required}"
build_tools_dir="$sdk_root/build-tools/$build_tools_version"
zipalign_path="$build_tools_dir/zipalign"
packages=()

[[ -d "$sdk_root/platforms/android-36" ]] || packages+=("platforms;android-36")
[[ -d "$build_tools_dir" ]] || packages+=("build-tools;$build_tools_version")
[[ -d "$sdk_root/ndk/$ndk_version" ]] || packages+=("ndk;$ndk_version")

if ((${#packages[@]})); then
  yes | sdkmanager --licenses >/dev/null || true
  sdkmanager "${packages[@]}"
else
  echo "Android API 36, Build Tools 36.0.0 and NDK $ndk_version are already installed."
fi

if [[ ! -x "$zipalign_path" ]]; then
  echo "zipalign is missing or not executable: $zipalign_path" >&2
  exit 1
fi

{
  echo "ANDROID_NDK_HOME=$sdk_root/ndk/$ndk_version"
  echo "NDK_HOME=$sdk_root/ndk/$ndk_version"
} >> "${GITHUB_ENV:?GITHUB_ENV is required}"

echo "$build_tools_dir" >> "${GITHUB_PATH:?GITHUB_PATH is required}"
