# - Try to find luajit
# Once done this will define
#  LUAJIT_FOUND - System has luajit
#  LUAJIT_INCLUDE_DIR - The luajit include directories
#  LUAJIT_LIBRARY - The libraries needed to use luajit

find_package(PkgConfig)
if (PKG_CONFIG_FOUND)
  pkg_check_modules(PC_LUAJIT QUIET luajit)
endif()

set(LUAJIT_DEFINITIONS ${PC_LUAJIT_CFLAGS_OTHER})

find_path(LuaJIT_INCLUDE_DIR luajit.h
          PATHS ${PC_LUAJIT_INCLUDEDIR} ${PC_LUAJIT_INCLUDE_DIRS}
          PATH_SUFFIXES luajit-2.0 luajit-2.1)

if(MSVC)
  list(APPEND LuaJIT_NAMES lua51)
elseif(MINGW)
  list(APPEND LuaJIT_NAMES libluajit libluajit-5.1)
else()
  list(APPEND LuaJIT_NAMES luajit-5.1)
endif()

find_library(LuaJIT_LIBRARY NAMES ${LuaJIT_NAMES}
             PATHS ${PC_LUAJIT_LIBDIR} ${PC_LUAJIT_LIBRARY_DIRS})

set(LuaJIT_LIBRARY ${LuaJIT_LIBRARY})
set(LuaJIT_INCLUDE_DIR ${LuaJIT_INCLUDE_DIR})

include(FindPackageHandleStandardArgs)
# handle the QUIETLY and REQUIRED arguments and set LUAJIT_FOUND to TRUE
# if all listed variables are TRUE
find_package_handle_standard_args(LuaJit DEFAULT_MSG
                                  LuaJIT_LIBRARY LuaJIT_INCLUDE_DIR)

mark_as_advanced(LuaJIT_INCLUDE_DIR LuaJIT_LIBRARY)
