//
// Created by Caelum van Ispelen on 5/31/21.
//

#include "script.h"

#include <sol/sol.hpp>
#include <absl/container/flat_hash_map.h>
#include <iostream>
#include <utility>

#include "game.h"
#include "hud.h"
#include "cursor.h"

namespace rip {
    // A HUD window written in Lua.
    class LuaWindow : public Window {
        sol::table luaTable;

    public:
        ~LuaWindow() override = default;

        LuaWindow(sol::table luaTable) : luaTable(std::move(luaTable)) {}

        void paint(Game &game, nk_context *nk) override {
            sol::function paintFn = luaTable.get<sol::function>("paint");
            paintFn.call<void>(luaTable, nk);
        }

        bool shouldClose() override {
            sol::function fn = luaTable.get<sol::function>("shouldClose");
            return fn.call<bool>(luaTable);
        }
    };

    struct ScriptImpl {
        sol::state lua;

        absl::flat_hash_map<std::string, std::vector<sol::function>> eventHandlers;

        Game *game = nullptr;

        void registerEventHandler(const std::string &event, sol::function handler) {
            if (!eventHandlers.contains(event)) eventHandlers[event] = {};

            eventHandlers[event].push_back(std::move(handler));

            std::cout << "[script] registered event handler for '" << event << "'" << std::endl;
        }

        // Generates Lua bindings.
        ScriptImpl() {
            lua.open_libraries(sol::lib::base, sol::lib::coroutine, sol::lib::count, sol::lib::ffi, sol::lib::debug,
                               sol::lib::io, sol::lib::math, sol::lib::os, sol::lib::package, sol::lib::string,
                               sol::lib::table, sol::lib::utf8);

            auto engine_type = lua.new_usertype<ScriptImpl>("Engine");
            engine_type["registerEventHandler"] = &ScriptImpl::registerEventHandler;
            lua["engine"] = std::ref(*this);

            auto nk_type = lua.new_usertype<nk_context>("NuklearContext");
            nk_type["beginWindow"] = [=](nk_context *nk, std::string title, float posX, float posY, float sizeX, float sizeY) {
                nk_begin(nk, title.c_str(), nk_rect(posX, posY, sizeX, sizeY), 0);
            };
            nk_type["endWindow"] = [=](nk_context *nk) {
                nk_end(nk);
            };
            nk_type["layoutDynamic"] = [=](nk_context *nk, float height, int cols) {
                nk_layout_row_dynamic(nk, height, cols);
            };
            nk_type["spacing"] = [=](nk_context *nk, int cols) {
                nk_spacing(nk, cols);
            };
            nk_type["label"] = [=](nk_context *nk, const std::string &text) {
                nk_label(nk, text.c_str(), NK_TEXT_ALIGN_LEFT);
            };
            nk_type["labelWrap"] = [=](nk_context *nk, const std::string &text) {
                nk_label_wrap(nk, text.c_str());
            };
            nk_type["buttonLabel"] = [=](nk_context *nk, const std::string &text) {
                return nk_button_label(nk, text.c_str()) != 0;
            };

            auto leader_type = lua.new_usertype<Leader>("Leader");
            leader_type["name"] = &Leader::name;
            leader_type["aggressive"] = &Leader::aggressive;
            leader_type["nukemonger"] = &Leader::nukemonger;
            leader_type["submissive"] = &Leader::submissive;
            leader_type["paranoia"] = &Leader::paranoia;
            leader_type["religious"] = &Leader::religious;

            auto civ_type = lua.new_usertype<CivKind>("CivKind");
            civ_type["id"] = &CivKind::id;
            civ_type["name"] = &CivKind::name;
            civ_type["adjective"] = &CivKind::adjective;
            civ_type["color"] = &CivKind::color;
            civ_type["leaders"] = &CivKind::leaders;
            civ_type["cities"] = &CivKind::cities;
            civ_type["startingTechs"] = &CivKind::startingTechs;

            auto player_type = lua.new_usertype<Player>("Player");
            player_type["getLeader"] = &Player::getLeader;
            player_type["getName"] = &Player::getUsername;
            player_type["hasAI"] = &Player::hasAI;
            player_type["getCiv"] = &Player::getCiv;
            player_type["declareWarOn"] = [=] (Player &self, Player &opponent) {
                self.declareWarOn(opponent.getID(), *game);
            };
            player_type["isAtWarWith"] = [=] (Player &self, Player &opponent) {
                return self.isAtWarWith(opponent.getID());
            };

            auto vec2_type = lua.new_usertype<glm::vec2>("Vec2");
            vec2_type["x"] = &glm::vec2::x;
            vec2_type["y"] = &glm::vec2::y;

            auto uvec2_type = lua.new_usertype<glm::uvec2>("UVec2");
            uvec2_type["x"] = &glm::uvec2::x;
            uvec2_type["y"] = &glm::uvec2::y;

            auto cursor_type = lua.new_usertype<Cursor>("Cursor");
            cursor_type["getWindowSize"] = &Cursor::getWindowSize;
            cursor_type["getPos"] = &Cursor::getPos;

            auto game_type = lua.new_usertype<Game>("Game");
            game_type["getThePlayer"] = [=](Game &game) {
                return &game.getThePlayer();
            };
            game_type["getCursor"] = [=](Game &game) {
                return &game.getCursor();
            };
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
            hud.openWindow(std::make_unique<LuaWindow>(std::move(window)));
        };

        lua["hud"] = hud;
    }

    ScriptEngine::~ScriptEngine() = default;

    ScriptEngine::ScriptEngine(ScriptEngine &&other) = default;

    void ScriptEngine::setGame(Game *game) {
        impl->game = game;
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

    std::shared_ptr<Asset> ScriptLoader::loadAsset(const std::string &data) {
        engine->impl->lua.script(data);
        return std::make_shared<ScriptAsset>();
    }

    ScriptLoader::ScriptLoader(const std::shared_ptr<ScriptEngine> &engine) : engine(engine) {}
}
