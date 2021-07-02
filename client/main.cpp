#define SOL_ALL_SAFETIES_ON 1
#define SOL_LUAJIT 1

#include <server.h>

#include <dume.h>
#include <sol/sol.hpp>

#include <assets.h>
#include <registry.h>
#include <tech.h>
#include <audio.h>
#include <bridge.h>

#include <memory>
#include <thread>
#include <network.h>

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

    // Creates bindings to the NetworkConnection class.
    void makeLuaNetworkBindings(sol::state &lua) {
        auto conn_type = lua.new_usertype<NetworkConnection>("NetworkConnection",
                                                             sol::constructors<NetworkConnection(const std::string&, uint16_t)>());

        conn_type["getError"] = &NetworkConnection::getError;
        conn_type["sendMessage"] = &NetworkConnection::sendMessage;
        conn_type["recvMessage"] = &NetworkConnection::recvMessage;
    }

    void makeLuaClientBindings(sol::state &lua, std::shared_ptr<Assets> assets,
                               std::shared_ptr<Registry> registry,
                               std::shared_ptr<TechTree> techTree, std::shared_ptr<AudioManager> audio) {
        makeLuaNetworkBindings(lua);

        lua["createSingleplayerGame"] = [=]() {
            auto bridges = newLocalBridgePair();
            auto server = std::make_shared<Server>(registry, techTree);
            server->addConnection(std::move(bridges.first));

            auto serverThread = std::thread([server = std::move(server)] () mutable {
                server->run();
            });
            serverThread.detach();

            return std::move(bridges.second);
        };

        auto bridge_type = lua.new_usertype<Bridge>("Bridge");
        bridge_type["pollReceivedPacket"] = &Bridge::pollReceivedPacket;
        bridge_type["sendPacket"] = &Bridge::sendPacket;

        lua["playSound"] = [=](const std::string &soundID) {
            auto *sound = audio->playSound(soundID);
            return sol::light<InstanceHandle>(sound);
        };
        lua["isSoundPlaying"] = [=](sol::light<InstanceHandle> handle) {
            return audio->isSoundPlaying(&*handle);
        };
        lua["stopSound"] = [=](sol::light<InstanceHandle> handle) {
            rodio_stop_sound(&*handle);
        };

        lua["getAssetIDsWithPrefix"] = [=](const std::string &prefix) {
            auto list = assets->getAllIDs();
            std::vector<std::string> result;
            for (const auto &id : list) {
                if (id.rfind(prefix, 0) == 0) {
                    result.push_back(id);
                }
            }
            return result;
        };
    }
}

CControlFlow invokeFunction(void *function, Event event) {
    auto &f = *((std::function<CControlFlow(Event)>*) function);
    return f(event);
}

int main() {
    EventLoop *eventLoop = winit_event_loop_new();
    WindowOptions options = {
            .title = "Riposte",
            .width = windowWidth,
            .height = windowHeight,
    };
    Window *window = winit_window_new(&options, eventLoop);

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

    audio->setAssets(assets);

    auto techTree = std::make_shared<rip::TechTree>(*assets, *registry);
    rip::makeLuaClientBindings(*lua, assets, registry, techTree, audio);

    lua->script_file("client/main.lua");

    sol::function renderFunction = (*lua)["render"];
    sol::function handleEventFunction = (*lua)["handleEvent"];
    sol::function resizeFunction = (*lua)["resize"];

    // Load registry data into Lua.
    sol::function loadFunction = (*lua)["loadDataFile"];
    auto luaAssets = std::make_shared<rip::Assets>();
    luaAssets->addLoader("civ", std::make_unique<rip::DataIntoLuaLoader>(loadFunction, "civs"));
    luaAssets->addLoader("unit", std::make_unique<rip::DataIntoLuaLoader>(loadFunction, "unitKinds"));
    luaAssets->addLoader("building", std::make_unique<rip::DataIntoLuaLoader>(loadFunction, "buildings"));
    luaAssets->addLoader("resource", std::make_unique<rip::DataIntoLuaLoader>(loadFunction, "resources"));
    luaAssets->addLoader("tech", std::make_unique<rip::DataIntoLuaLoader>(loadFunction, "techs"));
    luaAssets->loadAssetsDir("assets", true); // skip non-data assets like images, etc.

    auto lastTime = winit_get_time();

    int width = windowWidth, height = windowHeight;
    double cursorX = 0, cursorY = 0;

    winit_window_grab_cursor(window, true);

    std::function<CControlFlow(Event)> callbackFunction([&](Event event) {
        audio->update();

        if (event.kind == EventKind::RedrawRequested) {
            const auto currentTime = winit_get_time();
            const auto dt = currentTime - lastTime;

            lastTime = currentTime;

            renderFunction.call<void>(dt);
            canvas->render();
        } else {
            canvas->handleEvent(event, *lua, handleEventFunction);
        }

        if (event.kind == EventKind::CursorMove) {
            cursorX = event.data.cursor_pos[0];
            cursorY = event.data.cursor_pos[1];
        } else if (event.kind == EventKind::Resized) {
            int oldWidth = width, oldHeight = height;
            width = event.data.new_size[0];
            height = event.data.new_size[1];
            resizeFunction.call<void>(lua->create_table_with("x", oldWidth, "y", oldHeight), lua->create_table_with("x", width, "y", height));
        }

        (*lua)["cursorPos"] = lua->create_table_with("x", cursorX, "y", cursorY);

        if (event.kind == EventKind::CloseRequested) {
            return CControlFlow::Exit;
        } else {
            return CControlFlow::Poll;
        }
    });

    winit_event_loop_run(eventLoop, &invokeFunction, (void *) (&callbackFunction));
}