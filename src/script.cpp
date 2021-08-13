//
// Created by Caelum van Ispelen on 5/31/21.
//

#include "renderer.h"

#include "script.h"

#include <sol/sol.hpp>
#include <absl/container/flat_hash_map.h>
#include <iostream>
#include <utility>

#include "game.h"
#include "hud.h"

namespace rip {
    // A HUD window written in Lua.
    class LuaWindow : public Window {
        sol::table luaTable;

    public:
        ~LuaWindow() override = default;

        LuaWindow(sol::table luaTable) : luaTable(std::move(luaTable)) {}

        void paint(Game &game, nk_context *nk, NVGcontext *vg) override {
            sol::function paintFn = luaTable.get<sol::function>("paint");
            paintFn.call<void>(luaTable, nk, Canvas(vg));
        }

        bool shouldClose() override {
            sol::function fn = luaTable.get<sol::function>("shouldClose");
            return fn.call<bool>(luaTable);
        }
    };

    struct ScriptImpl {
        sol::state lua;

        absl::flat_hash_map<std::string, std::vector<sol::function>> eventHandlers;

        std::shared_ptr<Game*> game;

        void registerEventHandler(const std::string &event, sol::function handler) {
            if (!eventHandlers.contains(event)) eventHandlers[event] = {};

            eventHandlers[event].push_back(std::move(handler));

            std::cout << "[script] registered event handler for '" << event << "'" << std::endl;
        }

        // Generates Lua bindings.
        ScriptImpl() :game(std::make_shared<Game*>(nullptr)) {
            lua.open_libraries(sol::lib::base, sol::lib::coroutine, sol::lib::count, sol::lib::ffi, sol::lib::debug,
                               sol::lib::io, sol::lib::math, sol::lib::os, sol::lib::package, sol::lib::string,
                               sol::lib::table, sol::lib::utf8);

            auto engine_type = lua.new_usertype<ScriptImpl>("Engine");
            engine_type["registerEventHandler"] = &ScriptImpl::registerEventHandler;
            lua["engine"] = std::ref(*this);
        }

        void forEachHandler(const std::string &event, std::function<void(sol::function &)> callback) {
            if (!eventHandlers.contains(event)) return;
            for (auto &handler : eventHandlers[event]) {
                callback(handler);
            }
        }
    };

    ScriptEngine::ScriptEngine() {
        impl = std::make_unique<ScriptImpl>();
    }

    void ScriptEngine::registerHudBindings(std::shared_ptr<Hud> hud) {
        auto &lua = impl->lua;

        auto hud_type = lua.new_usertype<Hud>("Hud");
        hud_type["openWindow"] = [=] (Hud &hud, sol::table window) {
            hud.openWindow(std::make_shared<LuaWindow>(std::move(window)));
        };
        hud_type["takeFullControl"] = &Hud::takeFullControl;

        lua["hud"] = hud;
    }

    ScriptEngine::~ScriptEngine() = default;

    ScriptEngine::ScriptEngine(ScriptEngine &&other) = default;

    void ScriptEngine::setGame(Game *game) {
        (*impl->game) = game;
        impl->lua["game"] = game;
    }

    void ScriptEngine::onWarDeclared(Player &declarer, Player &declared) {
        impl->forEachHandler("onWarDeclared", [&] (sol::function &handler) {
            handler.call<void>(declarer, declared);
        });
    }

    void ScriptEngine::onDialogueOpened(Player &with) {
        impl->forEachHandler("onDialogueOpened", [&] (sol::function &handler) {
            handler.call<void>(with);
        });
    }

    void ScriptEngine::onPosClicked(glm::uvec2 pos) {
        impl->forEachHandler("onPosClicked", [=] (sol::function handler) {
            handler.call<void>(pos);
        });
    }

    void ScriptEngine::onKeyPressed(int key) {
        impl->forEachHandler("onKeyPressed", [=] (sol::function handler) {
            handler.call<void>(key);
        });
    }

    void ScriptEngine::onTurnEnd() {
        impl->forEachHandler("onTurnEnd", [=] (sol::function handler) {
            handler.call<void>();
        });
    }

    std::shared_ptr<Asset> ScriptLoader::loadAsset(const std::string &id, const std::string &data) {
        engine->impl->lua.script(data);
        return std::make_shared<ScriptAsset>();
    }

    ScriptLoader::ScriptLoader(const std::shared_ptr<ScriptEngine> &engine) : engine(engine) {}
}
