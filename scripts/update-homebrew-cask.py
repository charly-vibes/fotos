#!/usr/bin/env python3
"""Create or update the Homebrew cask for the fotos desktop app."""
import sys
import os

cask_path = sys.argv[1]
version = sys.argv[2]
tag = sys.argv[3]
appimage_sha = sys.argv[4]
dmg_sha = sys.argv[5] if len(sys.argv) > 5 else None

base = f"https://github.com/charly-vibes/fotos/releases/download/{tag}"

dmg_sha_line = f'sha256 "{dmg_sha}"' if dmg_sha else 'sha256 :no_check'

cask = f"""\
cask "fotos" do
  version "{version}"

  on_linux do
    url "{base}/Fotos_{version}_amd64.AppImage"
    sha256 "{appimage_sha}"
  end

  on_macos do
    url "{base}/Fotos_{version}_aarch64.dmg"
    {dmg_sha_line}
  end

  name "Fotos"
  desc "AI-powered screenshot capture and analysis tool"
  homepage "https://github.com/charly-vibes/fotos"

  on_linux do
    binary "Fotos_\#{{version}}_amd64.AppImage", target: "fotos"
  end

  on_macos do
    app "Fotos.app"
  end

  zap trash: [
    "~/.config/fotos",
    "~/.local/share/fotos",
    "~/Library/Application Support/fotos",
    "~/Library/Caches/fotos",
    "~/Library/Preferences/io.github.charly-vibes.fotos.plist",
  ]
end
"""

os.makedirs(os.path.dirname(cask_path), exist_ok=True)
with open(cask_path, "w") as f:
    f.write(cask)

print(f"Wrote {cask_path} (version {version})")
print(f"  AppImage sha256: {appimage_sha[:16]}...")
if dmg_sha:
    print(f"  DMG sha256: {dmg_sha[:16]}...")
