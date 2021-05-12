cmake_minimum_required(VERSION 3.14)
project(riposte)

#set(CMAKE_C_COMPILER clang-11)
#set(CMAKE_CXX_COMPILER clang++-11)

include(${CMAKE_BINARY_DIR}/conanbuildinfo.cmake)
conan_basic_setup()

set(CMAKE_CXX_STANDARD 20)

set(NANOVG_SOURCES external/nanovg/src/nanovg.c)
set(SOURCES ${NANOVG_SOURCES} src/main.cpp src/math.h src/tile.cpp src/tile.h src/game.cpp src/game.h src/renderer.cpp src/renderer.h src/assets.cpp src/assets.h src/view.cpp src/view.h src/cursor.cpp src/cursor.h src/mapgen.cpp src/mapgen.h src/rng.cpp src/rng.h)

add_executable(riposte ${SOURCES})

add_subdirectory(external/FastNoise2)

target_compile_definitions(riposte PRIVATE FONS_USE_FREETYPE)
target_include_directories(riposte PRIVATE external/nanovg/src external/FastNoise2/include)

target_link_libraries(riposte ${CONAN_LIBS} FastNoise)
