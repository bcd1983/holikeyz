#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"

PACKAGE_DIR=holikeyz-ring-light
APPLET_ID=com.holikeyz.RingLight

if ! command -v kpackagetool6 >/dev/null 2>&1; then
    echo "kpackagetool6 not found — install the KDE Frameworks 6 tooling." >&2
    exit 1
fi

if kpackagetool6 -t Plasma/Applet -l 2>/dev/null | grep -q "^$APPLET_ID$"; then
    echo "Upgrading $APPLET_ID..."
    kpackagetool6 -t Plasma/Applet -u "$PACKAGE_DIR"
else
    echo "Installing $APPLET_ID..."
    kpackagetool6 -t Plasma/Applet -i "$PACKAGE_DIR"
fi

cat <<MSG

Installed. To add it to your panel:
  Right-click the panel → Add or Manage Widgets → search "Ring Light"

If Plasma Shell is acting up after an upgrade, reload it:
  kquitapp6 plasmashell && kstart plasmashell
MSG
