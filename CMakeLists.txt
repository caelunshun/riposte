cmake_minimum_required(VERSION 3.14)
project(riposte)

set(CONAN_DISABLE_CHECK_COMPILER ON)

include(${CMAKE_BINARY_DIR}/conanbuildinfo.cmake)
conan_basic_setup()

set(CMAKE_CXX_STANDARD 17)

set(Protobuf_USE_STATIC_LIBS ON)
include(FindProtobuf)
find_package(Protobuf REQUIRED)

include(FetchContent)

FetchContent_Declare(
        Corrosion
        GIT_REPOSITORY https://github.com/AndrewGaspar/corrosion.git
        GIT_TAG origin/master # Optionally specify a version tag or branch here
)

FetchContent_MakeAvailable(Corrosion)

corrosion_import_crate(MANIFEST_PATH c-bindings/Cargo.toml CRATES riposte-c-bindings)

protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS proto/riposte.proto proto/mapgen.proto)

set(NANOVG_SOURCES external/nanovg/src/nanovg.c)
set(SOURCES
        ${PROTO_SRCS}
        ${NANOVG_SOURCES}
        src/ai.cpp
        src/assets.cpp
        src/city.cpp
        src/combat.cpp
        src/culture.cpp
        src/cursor.cpp
        src/era.cpp
        src/event.cpp
        src/game.cpp
        src/hud.cpp
        src/lobby.cpp
        src/mapgen.cpp
        src/network.cpp
        src/path.cpp
        src/player.cpp
        src/protocol.cpp
        src/registry.cpp
        src/renderer.cpp
        src/ripmath.cpp
        src/rng.cpp
        src/saveload.cpp
        src/server.cpp
        src/ship.cpp
        src/slot_map.cpp
        src/stack.cpp
        src/tech.cpp
        src/tile.cpp
        src/trade.cpp
        src/ui.cpp
        src/unit.cpp
        src/view.cpp
        src/worker.cpp
        src/yield.cpp

        src/mapgen/grid.cpp
        src/mapgen/line_mesh.cpp
        src/mapgen/land.cpp
        src/mapgen/terrain.cpp
        src/mapgen/starting_locations.cpp)

add_library(riposte-lib ${SOURCES})

set(FASTNOISE2_NOISETOOL OFF CACHE BOOL "" FORCE)
add_subdirectory(external/FastNoise2)

target_include_directories(riposte-lib PUBLIC
        src/
        external/nanovg/src
        external/FastNoise2/include
        external/Nuklear
        external/filesystem/include
        c-bindings/include
        ${CMAKE_CURRENT_BINARY_DIR} // for protobuf files
        ${PROTOBUF_INCLUDE_DIR})
target_compile_definitions(riposte-lib PRIVATE FONS_USE_FREETYPE)

#set(CMAKE_C_COMPILER "/usr/bin/clang-11")
#set(CMAKE_CXX_COMPILER "/usr/bin/clang++-11")

# needed for audio
if(APPLE)
    find_library(CORE_AUDIO_LIBRARY CoreAudio)
    find_library(AUDIO_TOOLBOX_LIBRARY AudioToolbox)
    find_library(AUDIO_UNIT_LIBRARY AudioUnit)
    find_library(CARBON_LIBRARY Carbon)
    SET(NATIVE_AUDIO_LIBS ${CORE_AUDIO_LIBRARY} ${AUDIO_TOOLBOX_LIBRARY} ${AUDIO_UNIT_LIBRARY} ${CARBON_LIBRARY})
elseif(UNIX)
    set(THREADS_PREFER_PTHREAD_FLAG ON)
    find_package(ALSA REQUIRED)
    find_package(Threads REQUIRED)
    SET(NATIVE_AUDIO_LIBS rt ${ALSA_LIBRARIES} Threads::Threads)
endif()

target_link_libraries(riposte-lib ${CONAN_LIBS} ${PROTOBUF_LIBRARY} FastNoise ${NATIVE_AUDIO_LIBS} riposte-c-bindings)

add_executable(riposte src/main.cpp)
target_link_libraries(riposte riposte-lib)

add_executable(mapgen-test src/mapgen/mapgen_test.cpp)
target_link_libraries(mapgen-test riposte-lib)
