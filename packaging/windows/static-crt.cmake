# Handed to every CMake build of the Windows installer (release.yml), so the C libraries
# (netCDF, HDF5) link the static C runtime like the crt-static Rust code around them.
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded")
