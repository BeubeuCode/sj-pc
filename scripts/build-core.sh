#!/usr/bin/env sh
# Builds the melonDS DS libretro core with our patches (patches/, see docs/BUILDING.md).
# The patch files are the source of truth: local edits under extern/melonDS and extern/melonDS-ds
# are discarded on every run.
set -eu
cd "$(dirname "$0")/.."

MELONDS_URL="https://github.com/JesseTG/melonDS"
# The commit melonDS DS v1.4.0 fetches (cmake/FetchDependencies.cmake, branch jtg/fix-uninitialized-opengl)
MELONDS_COMMIT="16f127dbb587a73371827ce79689c5d237c4b59b"

apply_patches() {
    repo="$1"
    patch_dir="$2"
    git -C "$repo" checkout -q -- .
    for patch in "$patch_dir"/*.patch; do
        [ -e "$patch" ] || continue
        git -C "$repo" apply "$PWD/$patch"
    done
}

if [ ! -d extern/melonDS/.git ]; then
    git clone -q "$MELONDS_URL" extern/melonDS
fi
git -C extern/melonDS fetch -q origin "$MELONDS_COMMIT" 2>/dev/null || true
git -C extern/melonDS checkout -q "$MELONDS_COMMIT"
apply_patches extern/melonDS patches/melonDS
apply_patches extern/melonDS-ds patches/melonDS-ds

cmake -S extern/melonDS-ds -B extern/build -DCMAKE_BUILD_TYPE=Release \
    -DFETCHCONTENT_SOURCE_DIR_MELONDS="$PWD/extern/melonDS"
cmake --build extern/build --parallel

# Replace the core by renaming a new file over it: a running game keeps its loaded copy intact.
mkdir -p cores
for core in $(find extern/build -maxdepth 4 \( -name 'melondsds_libretro.dylib' -o -name 'melondsds_libretro.so' -o -name 'melondsds_libretro.dll' \)); do
    name=$(basename "$core")
    cp "$core" "cores/.$name.new"
    mv -f "cores/.$name.new" "cores/$name"
done
ls cores
