cmake_minimum_required(VERSION 3.14)
project(riposte)

include(${CMAKE_BINARY_DIR}/conanbuildinfo.cmake)
conan_basic_setup()

set(CMAKE_CXX_STANDARD 20)

set(NANOVG_SOURCES external/nanovg/src/nanovg.c)
set(SOURCES ${NANOVG_SOURCES} src/main.cpp src/ripmath.h src/tile.cpp src/tile.h src/game.cpp src/game.h src/renderer.cpp src/renderer.h src/assets.cpp src/assets.h src/view.cpp src/view.h src/cursor.cpp src/cursor.h src/mapgen.cpp src/mapgen.h src/rng.cpp src/rng.h src/city.cpp src/city.h src/player.cpp src/player.h src/registry.cpp src/registry.h src/unit.cpp src/unit.h src/ripmath.cpp src/hud.cpp src/hud.h src/ui.cpp src/ui.h src/path.cpp src/path.h src/ai.cpp src/ai.h src/worker.cpp src/traversal.h src/culture.cpp src/culture.h src/tech.cpp src/yield.cpp src/yield.h)

add_executable(riposte ${SOURCES})

add_subdirectory(external/FastNoise2)

target_compile_definitions(riposte PRIVATE FONS_USE_FREETYPE)
target_include_directories(riposte PRIVATE
        external/nanovg/src
        external/FastNoise2/include
        external/Rea/Rea
        external/Nuklear)

#set(CMAKE_C_COMPILER "/usr/bin/clang-11")
#set(CMAKE_CXX_COMPILER "/usr/bin/clang++-11")

target_link_libraries(riposte ${CONAN_LIBS} FastNoise)
