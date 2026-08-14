#!/bin/bash
# Ensure the tbd CLI is available and run `tbd prime`.
# Installed by: tbd setup --auto. Runs on SessionStart and PreCompact.
#
# Local-first, then a VERSION-PINNED zero-install fallback. Pinning is both a
# supply-chain control (an unpinned runner re-resolves to latest on every run
# and bypasses any cool-off) and a consistency control (every teammate and agent
# runs the same tbd version).

# Prefer common local bin locations.
export PATH="$HOME/.local/bin:$HOME/bin:/usr/local/bin:$PATH"

tbd_version_at_least() {
  local version_pattern='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$'
  local installed_major installed_minor installed_patch
  local required_major required_minor required_patch
  local installed_prerelease required_prerelease

  if [[ $1 =~ $version_pattern ]]; then
    installed_major=${BASH_REMATCH[1]}
    installed_minor=${BASH_REMATCH[2]}
    installed_patch=${BASH_REMATCH[3]}
    installed_prerelease=${BASH_REMATCH[4]}
  else
    return 1
  fi
  if [[ $2 =~ $version_pattern ]]; then
    required_major=${BASH_REMATCH[1]}
    required_minor=${BASH_REMATCH[2]}
    required_patch=${BASH_REMATCH[3]}
    required_prerelease=${BASH_REMATCH[4]}
  else
    return 1
  fi

  if (( installed_major != required_major )); then
    (( installed_major > required_major ))
    return
  fi
  if (( installed_minor != required_minor )); then
    (( installed_minor > required_minor ))
    return
  fi
  if (( installed_patch != required_patch )); then
    (( installed_patch > required_patch ))
    return
  fi
  # A dirty or development build with the same core version is older than the stable
  # repository pin and must take the exact-package fallback.
  if [ -z "$required_prerelease" ]; then
    [ -z "$installed_prerelease" ]
  else
    [ -z "$installed_prerelease" ] || [ "$installed_prerelease" = "$required_prerelease" ]
  fi
}

# Local-first when the installed CLI satisfies this repository's pin. An older CLI may
# be unable to read a newer tbd_format, so let the exact pinned fallback handle upgrades.
installed_tbd_version=""
if command -v tbd &> /dev/null; then
    installed_tbd_version=$(tbd --version 2>/dev/null || true)
fi
if [ -n "$installed_tbd_version" ] && tbd_version_at_least "$installed_tbd_version" "0.6.2"; then
    tbd prime "$@"
    exit $?
fi

# Pinned zero-install fallback. Never use an unpinned runner here.
if command -v npx &> /dev/null; then
    if [ -n "$installed_tbd_version" ]; then
        echo "[tbd] Local tbd $installed_tbd_version does not satisfy repository pin 0.6.2; using pinned fallback." >&2
    fi
    # The exact first-party pin is approved in supply-chain-policy.json. npm's `before`
    # setting has no per-package exception, so clear it only for this verified bootstrap.
    env -u NPM_CONFIG_BEFORE npx --yes get-tbd@0.6.2 prime "$@"
    exit $?
fi

if [ -n "$installed_tbd_version" ]; then
    echo "[tbd] Local tbd $installed_tbd_version does not satisfy repository pin 0.6.2, and npx is unavailable."
else
    echo "[tbd] tbd CLI not found and npx is unavailable."
fi
echo "[tbd] Install it with: npm install -g get-tbd@0.6.2"
exit 1
