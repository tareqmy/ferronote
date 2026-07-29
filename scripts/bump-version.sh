#!/usr/bin/env bash
set -e

NEW_VER="$1"

if [ -z "$NEW_VER" ]; then
  echo "Usage: ./scripts/bump-version.sh <VERSION> (e.g. 1.0.4)"
  exit 1
fi

# Strip leading 'v' if provided
NEW_VER="${NEW_VER#v}"

echo "🔄 Bumping Ferronote version to ${NEW_VER}..."

# 1. Update .version
echo "$NEW_VER" > .version
echo "  ✓ Updated .version"

# 2. Update Cargo.toml
sed -i '' -E "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"/version = \"${NEW_VER}\"/" Cargo.toml
echo "  ✓ Updated Cargo.toml"

# 3. Update Homebrew Formula
sed -i '' -E "s/archive\/refs\/tags\/v[0-9]+\.[0-9]+\.[0-9]+\.tar\.gz/archive\/refs\/tags\/v${NEW_VER}\.tar\.gz/" Formula/ferronote.rb
echo "  ✓ Updated Formula/ferronote.rb"

# 4. Update AUR PKGBUILD
sed -i '' -E "s/^pkgver=[0-9]+\.[0-9]+\.[0-9]+/pkgver=${NEW_VER}/" packaging/aur/PKGBUILD
echo "  ✓ Updated packaging/aur/PKGBUILD"

# 5. Update Man page
sed -i '' -E "s/ferronote [0-9]+\.[0-9]+\.[0-9]+/ferronote ${NEW_VER}/" docs/ferronote.1
echo "  ✓ Updated docs/ferronote.1"

# 6. Update Gemini Style Guide
sed -i '' -E "s/v[0-9]+\.[0-9]+\.[0-9]+/v${NEW_VER}/" .gemini/STYLE_GUIDE.md
echo "  ✓ Updated .gemini/STYLE_GUIDE.md"

# 7. Update Cargo.lock via cargo check
cargo check > /dev/null 2>&1
echo "  ✓ Updated Cargo.lock"

echo "✨ Version successfully bumped to ${NEW_VER} across all files!"
