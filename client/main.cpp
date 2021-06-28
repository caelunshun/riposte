#define SOL_ALL_SAFETIES_ON 1
#define SOL_LUAJIT 1

#include <dume.h>
#include <sol/sol.hpp>

#include <assets.h>
#include <registry.h>
#include <tech.h>
#include <audio.h>
#include <bridge.h>
#include <server.h>

#include <memory>
#include <thread>

const int windowWidth = 1920 / 2;
const int windowHeight = 1080 / 2;

namespace rip {
    struct ImageAsset : public Asset {
        uint64_t sprite;
        ImageAsset(uint64_t sprite) : sprite(sprite) {}
    };
    class ImageLoader : public AssetLoader {
        std::shared_ptr<dume::Canvas> canvas;

    public:
        explicit ImageLoader(const std::shared_ptr<dume::Canvas> &canvas) : canvas(canvas) {}

        std::shared_ptr<Asset> loadAsset(const std::string &id, const std::string &data) override {
            const auto sprite = canvas->createSpriteFromEncoded(id, data);
            return std::make_shared<ImageAsset>(sprite);
        }
    };

    struct FontAsset : public Asset {};
    class FontLoader : public AssetLoader {
        std::shared_ptr<dume::Canvas> canvas;

    public:
        explicit FontLoader(const std::shared_ptr<dume::Canvas> &canvas) : canvas(canvas) {}

        std::shared_ptr<Asset> loadAsset(const std::string &id, const std::string &data) override {
            canvas->loadFont(data);
            return std::make_shared<FontAsset>();
        }
    };

    class DataIntoLuaLoader : public AssetLoader {
        sol::function loadFunction;
        std::string luaRegistry;

    public:
        explicit DataIntoLuaLoader(sol::function loadFunction, std::string luaRegistry) : loadFunction(loadFunction), luaRegistry(luaRegistry) {}

        std::shared_ptr<Asset> loadAsset(const std::string &id, const std::string &data) override {
            loadFunction.call<void>(id, luaRegistry, data);
        }
    };

    void makeLuaClientBindings(sol::state &lua, std::shared_ptr<Registry> registry, std::shared_ptr<TechTree> techTree) {
        lua["createSingleplayerGame"] = [=]() {
            auto bridges = newLocalBridgePair();
            Server server(registry, techTree);
            server.addConnection(std::move(bridges.first));

            auto serverThread = std::thread([server = std::move(server)] () mutable {
                server.run();
            });
            serverThread.detach();

            return std::move(bridges.second);
        };

        auto bridge_type = lua.new_usertype<Bridge>("Bridge");
        bridge_type["pollReceivedPacket"] = &Bridge::pollReceivedPacket;
    }
}

int main() {
    glfwInit();
    GLFWwindow *window = glfwCreateWindow(windowWidth, windowHeight, "Riposte", nullptr, nullptr);

    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);

#ifdef __APPLE__
    glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
#endif

    auto canvas = std::make_shared<dume::Canvas>(window);

    auto lua = std::make_shared<sol::state>();
    lua->open_libraries(
            sol::lib::string, sol::lib::coroutine,
            sol::lib::base, sol::lib::bit32,
            sol::lib::count, sol::lib::debug,
            sol::lib::ffi, sol::lib::io,
            sol::lib::jit, sol::lib::math,
            sol::lib::os, sol::lib::package,
            sol::lib::table, sol::lib::utf8
            );

    dume::makeLuaBindings(*lua);
    (*lua)["cv"] = canvas;

    auto registry = std::make_shared<rip::Registry>();

    auto audio = std::make_shared<rip::AudioManager>();

    auto assets = std::make_shared<rip::Assets>();
    assets->addLoader("image", std::make_unique<rip::ImageLoader>(canvas));
    assets->addLoader("font", std::make_unique<rip::FontLoader>(canvas));
    assets->addLoader("civ", std::make_unique<rip::CivLoader>(registry));
    assets->addLoader("unit", std::make_unique<rip::UnitLoader>(registry));
    assets->addLoader("building", std::make_unique<rip::BuildingLoader>(registry));
    assets->addLoader("resource", std::make_unique<rip::ResourceLoader>(registry));
    assets->addLoader("tech", std::make_unique<rip::TechLoader>());
    assets->addLoader("sound", std::make_unique<rip::AudioLoader>(audio));
    assets->loadAssetsDir("assets", false);

    auto techTree = std::make_shared<rip::TechTree>(*assets, *registry);
    rip::makeLuaClientBindings(*lua, registry, techTree);

    lua->script_file("client/main.lua");

    sol::function renderFunction = (*lua)["render"];
    sol::function handleEventFunction = (*lua)["handleEvent"];
    sol::function resizeFunction = (*lua)["resize"];

    canvas->setGlfwCallbacks(canvas, lua, handleEventFunction, resizeFunction);

    // Load registry data into Lua.
    sol::function loadFunction = (*lua)["loadDataFile"];
    auto luaAssets = std::make_shared<rip::Assets>();
    luaAssets->addLoader("civ", std::make_unique<rip::DataIntoLuaLoader>(loadFunction, "civs"));
    luaAssets->addLoader("unit", std::make_unique<rip::DataIntoLuaLoader>(loadFunction, "unitKinds"));
    luaAssets->addLoader("building", std::make_unique<rip::DataIntoLuaLoader>(loadFunction, "buildings"));
    luaAssets->addLoader("resource", std::make_unique<rip::DataIntoLuaLoader>(loadFunction, "resources"));
    luaAssets->addLoader("tech", std::make_unique<rip::DataIntoLuaLoader>(loadFunction, "techs"));
    luaAssets->loadAssetsDir("assets", true); // skip non-data assets like images, etc.

    auto lastTime = glfwGetTime();
    while (!glfwWindowShouldClose(window)) {
        const auto currentTime = glfwGetTime();
        const auto dt = currentTime - lastTime;

        lastTime = currentTime;

        glfwPollEvents();
        renderFunction.call<void>(dt);

        canvas->render();

        int width, height;
        glfwGetWindowSize(window, &width, &height);

        double cursorX, cursorY;
        glfwGetCursorPos(window, &cursorX, &cursorY);

        bool shouldFixCursor = (cursorX < 0 || cursorY < 0 || cursorX > width || cursorY > height);

        cursorX = std::clamp(cursorX, 0.0, (double) width);
        cursorY = std::clamp(cursorY, 0.0, (double) height);

        if (shouldFixCursor) {
            glfwSetCursorPos(window, cursorX, cursorY);
        }

        (*lua)["cursorPos"] = lua->create_table_with("x", cursorX, "y", cursorY);
    }

    // force shutdown with a segfault. a bug in Dume causes
    // the program to hang the entire system
    // on shutdown and resize, so this is a temporary hack.
    int *x = nullptr;
    *x += 1;

    glfwDestroyWindow(window);
    glfwTerminate();
}
