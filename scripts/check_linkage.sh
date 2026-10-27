#!/usr/bin/env bash
# Fails when the release binary loads netCDF, HDF5 or (Windows) the Visual C++ runtime as
# shared libraries: installers ship the binary alone, so these must be linked in.
set -euo pipefail
cd "$(dirname "$0")/.."

case "$(uname -s)" in
  Darwin) libs=$(otool -L target/release/octant) ;;
  Linux) libs=$(ldd target/release/octant) ;;
  MINGW* | MSYS*)
    vs=$("/c/Program Files (x86)/Microsoft Visual Studio/Installer/vswhere.exe" -latest -property installationPath)
    dumpbin=$(find "$(cygpath "$vs")/VC/Tools/MSVC" -path '*Hostx64/x64/dumpbin.exe' | head -1)
    libs=$("$dumpbin" -dependents target/release/octant.exe)
    ;;
  *) echo "unknown OS $(uname -s)" >&2; exit 1 ;;
esac

echo "$libs"
if grep -iE 'netcdf|hdf5|vcruntime|msvcp' <<<"$libs"; then
  echo "::error::the release binary links the libraries above dynamically" >&2
  exit 1
fi
