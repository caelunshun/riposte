# Try to find LuaJIT
# LuaJIT_FOUND - System has LuaJIT
# LuaJIT_LIBRARY - The libraries needed to use LuaJIT
# LuaJIT_INCLUDE_DIR - The LuaJIT include directories

# Prefer static linking
set(orig_CMAKE_FIND_LIBRARY_SUFFIXES ${CMAKE_FIND_LIBRARY_SUFFIXES})
set(CMAKE_FIND_LIBRARY_SUFFIXES .lib .a ${CMAKE_FIND_LIBRARY_SUFFIXES})

find_library(LuaJIT_LIBRARY
        NAMES
        luajit luajit_64 luajit-5.1 libluajit libluajit_64
        PATHS
        ${PROJECT_SOURCE_DIR}/ext/lua/${libpath}
        /usr/
        /usr/bin/
        /usr/include/
        /usr/lib/
        /usr/local/
        /usr/local/bin/
        /opt/)

find_path(LuaJIT_INCLUDE_DIR
        NAMES
        lua.h
        PATHS
        ${PROJECT_SOURCE_DIR}/ext/lua/include/) # Only look internally

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(LuaJIT DEFAULT_MSG LuaJIT_LIBRARY LuaJIT_INCLUDE_DIR)

message(STATUS "LuaJIT_FOUND: ${LuaJIT_FOUND}")
message(STATUS "LuaJIT_LIBRARY: ${LuaJIT_LIBRARY}")
message(STATUS "LuaJIT_INCLUDE_DIR: ${LuaJIT_INCLUDE_DIR}")

if (${LuaJIT_FOUND})
    link_libraries(${LuaJIT_LIBRARY})
    include_directories(${LuaJIT_INCLUDE_DIR})
    include_directories(${LuaJIT_INCLUDE_DIR}/../)
endif()

set(CMAKE_FIND_LIBRARY_SUFFIXES ${orig_CMAKE_FIND_LIBRARY_SUFFIXES})
