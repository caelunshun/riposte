cmake_minimum_required(VERSION 3.15)
project(riposte)

include(${CMAKE_BINARY_DIR}/conanbuildinfo.cmake)
conan_basic_setup()

set(CMAKE_CXX_STANDARD 20)

set(SOURCES src/main.cpp src/math.h src/tile.cpp src/tile.h src/game.cpp src/game.h src/renderer.cpp src/renderer.h)
set(NANOVG_SOURCES external/nanovg/src/nanovg.c)

add_library(nanovg ${NANOVG_SOURCES})
target_compile_definitions(nanovg PRIVATE FONS_USE_FREETYPE)
target_include_directories(nanovg PUBLIC external/nanovg/src)

add_executable(riposte ${SOURCES})
target_link_libraries(riposte ${CONAN_LIBS} nanovg)
