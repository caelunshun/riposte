cmake_minimum_required(VERSION 3.14)
project(riposte)

include(${CMAKE_BINARY_DIR}/conanbuildinfo.cmake)
conan_basic_setup()

set(CMAKE_CXX_STANDARD 20)

include(FindProtobuf)
find_package(Protobuf REQUIRED)

include(FetchContent)

FetchContent_Declare(
        Corrosion
        GIT_REPOSITORY https://github.com/AndrewGaspar/corrosion.git
        GIT_TAG origin/master # Optionally specify a version tag or branch here
)

FetchContent_MakeAvailable(Corrosion)

corrosion_import_crate(MANIFEST_PATH external/dume/renderer/Cargo.toml)

add_library(pb SHARED external/lua-protobuf/pb.c)
target_link_libraries(pb ${CONAN_LIBS})

set(DUME_SOURCES external/dume/src/dume.cpp)
add_library(dume ${DUME_SOURCES})
target_include_directories(dume PUBLIC external/dume/include/ external/dume/renderer/include/)
target_include_directories(dume PRIVATE external/sol2/include)
target_link_libraries(dume ${CONAN_LIBS} dume-renderer)

set(NANOVG_SOURCES external/nanovg/src/nanovg.c)
set(SOURCES ${NANOVG_SOURCES} proto/riposte.pb.cc client/main.cpp src/ripmath.h src/tile.cpp src/tile.h src/game.cpp src/game.h src/renderer.cpp src/renderer.h src/assets.cpp src/assets.h src/view.cpp src/view.h src/cursor.cpp src/cursor.h src/mapgen.cpp src/mapgen.h src/rng.cpp src/rng.h src/city.cpp src/city.h src/player.cpp src/player.h src/registry.cpp src/registry.h src/unit.cpp src/unit.h src/ripmath.cpp src/hud.cpp src/hud.h src/ui.cpp src/ui.h src/path.cpp src/path.h src/ai.cpp src/ai.h src/worker.cpp src/traversal.h src/culture.cpp src/culture.h src/tech.cpp src/yield.cpp src/yield.h src/trade.cpp src/trade.h src/audio.cpp src/audio.h src/era.cpp src/era.h src/combat.cpp src/combat.h src/stack.cpp src/stack.h src/event.cpp src/event.h src/script.cpp src/script.h src/script_bindings/unit.cpp src/script_bindings/player.cpp src/script_bindings/game.cpp src/script_bindings/registry.cpp src/script_bindings/other.cpp src/script_bindings/city.cpp src/ship.cpp src/ship.h src/bridge.cpp src/server.cpp src/server.h)

add_executable(riposte ${SOURCES})

add_subdirectory(external/FastNoise2)

corrosion_import_crate(MANIFEST_PATH crodio/Cargo.toml)

target_compile_definitions(riposte PRIVATE FONS_USE_FREETYPE SOL_ALL_SAFETIES_ON)
target_include_directories(riposte PRIVATE
        src/
        proto/
        external/nanovg/src
        external/FastNoise2/include
        external/Rea/Rea
        external/Nuklear
        external/sol2/include
        crodio/include
        ${PROTOBUF_INCLUDE_DIR})

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

target_link_libraries(riposte dume ${CONAN_LIBS} ${PROTOBUF_LIBRARY} FastNoise ${NATIVE_AUDIO_LIBS} crodio pb)
