//
// Created by Caelum van Ispelen on 5/31/21.
//

#include "script.h"

#include <sol/sol.hpp>
#include <absl/container/flat_hash_map.h>
#include <iostream>

#include "hud.h"

namespace rip {
    struct ScriptImpl {
        sol::state lua;

        absl::flat_hash_map<std::string, std::vector<sol::function>> eventHandlers;

        Game *game = nullptr;

        void registerEventHandler(const std::string &event, sol::function handler) {
            if (!eventHandlers.contains(event)) eventHandlers[event] = {};

            eventHandlers[event].push_back(std::move(handler));

            std::cout << "[script] registered event handler for '" << event << "'" << std::endl;
        }

        ScriptImpl() {
            lua.open_libraries(sol::lib::base, sol::lib::coroutine, sol::lib::count, sol::lib::ffi, sol::lib::debug,
                               sol::lib::io, sol::lib::math, sol::lib::os, sol::lib::package, sol::lib::string,
                               sol::lib::table, sol::lib::utf8);

            auto engine_type = lua.new_usertype<ScriptImpl>("Engine");
            engine_type["registerEventHandler"] = &ScriptImpl::registerEventHandler;
            lua["engine"] = std::ref(*this);

            auto hud_type = lua.new_usertype<Hud>("Hud");

        }
    };

    ScriptEngine::ScriptEngine() {
        impl = std::make_unique<ScriptImpl>();
    }

    ScriptEngine::~ScriptEngine() = default;

    ScriptEngine::ScriptEngine(ScriptEngine &&other) = default;

    void ScriptEngine::setGame(Game *game) {
        impl->game = game;
    }

    void ScriptEngine::onWarDeclared(Player &declarer, Player &declared) {

    }

    std::shared_ptr<Asset> ScriptLoader::loadAsset(const std::string &data) {
        engine->impl->lua.script(data);
        return std::make_shared<ScriptAsset>();
    }

    ScriptLoader::ScriptLoader(const std::shared_ptr<ScriptEngine> &engine) : engine(engine) {}
}
