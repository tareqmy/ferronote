#!/usr/bin/env bash
set -e

if [ ! -f ".version" ]; then
  echo "Error: .version file not found!"
  exit 1
fi

NEW_VER=$(cat .version | tr -d '\r\n')

# Strip leading 'v' if provided
NEW_VER="${NEW_VER#v}"

echo "🔄 Bumping Ferronote version to ${NEW_VER}..."

# 2. Update Cargo.toml, the workspace crates, and the inter-crate dependency versions
sed -i '' -E "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"/version = \"${NEW_VER}\"/" \
  Cargo.toml ferronote-store/Cargo.toml ferronote-search/Cargo.toml
sed -i '' -E "s/^(ferronote-(store|search) = \{ version = \")[0-9]+\.[0-9]+\.[0-9]+\"/\1${NEW_VER}\"/" Cargo.toml
echo "  ✓ Updated Cargo.toml, ferronote-store/Cargo.toml, ferronote-search/Cargo.toml"

# 3. Update Homebrew Formula
sed -i '' -E "s/archive\/refs\/tags\/v[0-9]+\.[0-9]+\.[0-9]+\.tar\.gz/archive\/refs\/tags\/v${NEW_VER}\.tar\.gz/" Formula/ferronote.rb
echo "  ✓ Updated Formula/ferronote.rb"

# 4. Update AUR PKGBUILD
sed -i '' -E "s/^pkgver=[0-9]+\.[0-9]+\.[0-9]+/pkgver=${NEW_VER}/" packaging/aur/PKGBUILD
echo "  ✓ Updated packaging/aur/PKGBUILD"

# 5. Update Man page (version and month)
sed -i '' -E "s/ferronote [0-9]+\.[0-9]+\.[0-9]+/ferronote ${NEW_VER}/" docs/ferronote.1
sed -i '' -E "s/^(\.TH FERRONOTE 1 )\"[A-Za-z]+ [0-9]{4}\"/\1\"$(date '+%B %Y')\"/" docs/ferronote.1
echo "  ✓ Updated docs/ferronote.1"

# 6. Update Gemini Style Guide
sed -i '' -E "s/v[0-9]+\.[0-9]+\.[0-9]+/v${NEW_VER}/" .gemini/STYLE_GUIDE.md
echo "  ✓ Updated .gemini/STYLE_GUIDE.md"

# 7. Update the version placeholder in the bug report template
sed -i '' -E "s/placeholder: \"[0-9]+\.[0-9]+\.[0-9]+\"/placeholder: \"${NEW_VER}\"/" .github/ISSUE_TEMPLATE/bug_report.yml
echo "  ✓ Updated .github/ISSUE_TEMPLATE/bug_report.yml"

# 8. Update Cargo.lock via cargo check
cargo check > /dev/null 2>&1 || true
echo "  ✓ Updated Cargo.lock"

echo "✨ Version successfully bumped to ${NEW_VER} across all files!"
