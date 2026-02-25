#!/usr/bin/env python3
"""Create or update the Scoop manifest for the fotos desktop app."""
import json
import os
import sys

manifest_path = sys.argv[1]
version = sys.argv[2]
tag = sys.argv[3]
msi_sha = sys.argv[4]

url = f"https://github.com/charly-vibes/fotos/releases/download/{tag}/fotos_{version}_x64_en-US.msi"

manifest = {
    "version": version,
    "description": "AI-powered screenshot capture and analysis tool",
    "homepage": "https://github.com/charly-vibes/fotos",
    "license": "MIT",
    "url": url,
    "hash": f"sha256:{msi_sha}",
    "installer": {
        "file": f"fotos_{version}_x64_en-US.msi"
    },
    "checkver": {
        "github": "https://github.com/charly-vibes/fotos"
    },
    "autoupdate": {
        "url": "https://github.com/charly-vibes/fotos/releases/download/v$version/fotos_$version_x64_en-US.msi"
    }
}

os.makedirs(os.path.dirname(manifest_path), exist_ok=True)
with open(manifest_path, "w") as f:
    json.dump(manifest, f, indent=4)
    f.write("\n")

print(f"Wrote {manifest_path} (version {version})")
print(f"  url: {url}")
print(f"  sha256: {msi_sha[:16]}...")
