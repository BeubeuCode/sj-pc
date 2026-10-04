#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/.."
cmake -S extern/melonDS-ds -B extern/build -DCMAKE_BUILD_TYPE=Release
cmake --build extern/build --parallel
mkdir -p cores
find extern/build -maxdepth 3 \( -name 'melondsds_libretro.dylib' -o -name 'melondsds_libretro.so' -o -name 'melondsds_libretro.dll' \) -exec cp {} cores/ \;
ls cores
