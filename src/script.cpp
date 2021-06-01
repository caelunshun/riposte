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
#include "unit.h"
#include "city.h"

namespace rip {
    static NVGcolor luaColor(sol::table color) {
        if (color.size() == 3) {
            return nvgRGB(color.get<int>(1), color.get<int>(2), color.get<int>(3));
        } else {
            return nvgRGBA(color.get<int>(1), color.get<int>(2), color.get<int>(3), color.get<int>(4));
        }
    }

    struct Canvas {
        NVGcontext *vg;

        Canvas(NVGcontext *vg) : vg(vg) {}
    };

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

            auto combat_type = lua.new_usertype<CombatBonus>("CombatBonus");
            combat_type["whenInCityBonus"] = &CombatBonus::whenInCityBonus;
            combat_type["againstUnitCategoryBonus"] = &CombatBonus::againstUnitCategoryBonus;
            combat_type["againstUnitBonus"] = &CombatBonus::againstUnitBonus;
            combat_type["onlyOnAttack"] = &CombatBonus::onlyOnAttack;
            combat_type["onlyOnDefense"] = &CombatBonus::onlyOnDefense;
            combat_type["unit"] = &CombatBonus::unit;
            combat_type["unitCategory"] = &CombatBonus::unitCategory;

            auto unit_kind_type = lua.new_usertype<UnitKind>("UnitKind");
            unit_kind_type["id"] = &UnitKind::id;
            unit_kind_type["name"] = &UnitKind::name;
            unit_kind_type["strength"] = &UnitKind::strength;
            unit_kind_type["movement"] = &UnitKind::movement;
            unit_kind_type["capabilities"] = &UnitKind::capabilities;
            unit_kind_type["cost"] = &UnitKind::cost;
            unit_kind_type["techs"] = &UnitKind::techs;
            unit_kind_type["resources"] = &UnitKind::resources;
            unit_kind_type["combatBonuses"] = &UnitKind::combatBonuses;
            unit_kind_type["category"] = &UnitKind::category;

            auto unit_type = lua.new_usertype<Unit>("Unit");
            unit_type["getKind"] = &Unit::getKind;
            unit_type["getPos"] = &Unit::getPos;
            unit_type["getOwner"] = [&] (Unit &self) {
                return &game->getPlayer(self.getOwner());
            };
            unit_type["getCombatStrength"] = &Unit::getCombatStrength;
            unit_type["getMovementLeft"] = &Unit::getMovementLeft;
            unit_type["getHealth"] = &Unit::getHealth;
            unit_type["setHealth"] = &Unit::setHealth;
            unit_type["canFight"] = &Unit::canFight;
            unit_type["shouldDie"] = &Unit::shouldDie;
            unit_type["setMovementLeft"] = &Unit::setMovementLeft;
            unit_type["canMove"] = [&] (Unit &self, glm::uvec2 target) {
                return self.canMove(target, *game);
            };
            unit_type["moveTo"] = [&] (Unit &self, glm::uvec2 target, bool allowCombat) {
                self.moveTo(target, *game, allowCombat);
            };
            unit_type["wouldAttack"] = [&] (Unit &self, Unit &other) {
                return self.wouldAttack(*game, other);
            };
            unit_type["hasPath"] = &Unit::hasPath;
            unit_type["setPath"] = &Unit::setPath;
            unit_type["moveAlongCurrentPath"] = &Unit::moveAlongCurrentPath;
            unit_type["isInCombat"] = &Unit::isInCombat;
            unit_type["fortify"] = &Unit::fortify;
            unit_type["isFortified"] = &Unit::isFortified;
            unit_type["fortifyUntilHealed"] = &Unit::fortifyUntilHealed;
            unit_type["skipTurn"] = &Unit::skipTurn;
            unit_type["teleportTo"] = [&] (Unit &self, glm::uvec2 pos) {
                self.teleportTo(pos, *game);
            };

            auto building_type = lua.new_usertype<Building>("Building");
            building_type["name"] = &Building::name;
            building_type["cost"] = &Building::cost;
            building_type["prerequisites"] = &Building::prerequisites;
            building_type["techs"] = &Building::techs;
            building_type["onlyCoastal"] = &Building::onlyCoastal;

            auto build_type = lua.new_usertype<BuildTask>("BuildTask");
            build_type["getCost"] = &BuildTask::getCost;
            build_type["getProgress"] = &BuildTask::getProgress;
            build_type["isFinished"] = &BuildTask::isFinished;
            build_type["getOverflow"] = &BuildTask::getOverflow;
            build_type["spendHammers"] = &BuildTask::spendHammers;
            build_type["getName"] = &BuildTask::getName;

            auto yield_type = lua.new_usertype<Yield>("Yield");
            yield_type["hammers"] = &Yield::hammers;
            yield_type["commerce"] = &Yield::commerce;
            yield_type["food"] = &Yield::food;

            auto city_type = lua.new_usertype<City>("City");
            city_type["getPos"] = &City::getPos;
            city_type["getName"] = &City::getName;
            city_type["getOwner"] = [&] (City &self) {
                return &game->getPlayer(self.getOwner());
            };
            city_type["isCapital"] = &City::isCapital;
            city_type["getCulture"] = [&] (City &self) {
                return self.getCulture().getCultureForPlayer(self.getOwner());
            };
            city_type["getCulturePerTurn"] = &City::getCulturePerTurn;
            city_type["getCultureLevel"] = [&] (City &self) {
                return self.getCultureLevel().value;
            };
            city_type["setName"] = &City::setName;
            city_type["getPopulation"] = &City::getPopulation;
            city_type["isCoastal"] = &City::isCoastal;
            city_type["hasBuildTask"] = &City::hasBuildTask;
            city_type["getBuildTask"] = &City::getBuildTask;
            city_type["estimateTurnsForCompletion"] = [&] (City &self, const BuildTask &task) {
                return self.estimateTurnsForCompletion(task, *game);
            };
            city_type["getBuildings"] = &City::getBuildings;
            city_type["hasBuilding"] = &City::hasBuilding;
            city_type["computeYield"] = [&] (City &self) {
                return self.computeYield(*game);
            };
            city_type["getWorkedTiles"] = &City::getWorkedTiles;
            city_type["updateWorkedTiles"] = [&] (City &self) {
                self.updateWorkedTiles(*game);
            };
            city_type["addManualWorkedTile"] = &City::addManualWorkedTile;
            city_type["removeManualWorkedTile"] = &City::removeManualWorkedTile;
            city_type["getManualWorkedTiles"] = &City::getManualWorkedTiles;
            city_type["canWorkTile"] = &City::canWorkTile;

            auto player_type = lua.new_usertype<Player>("Player");
            player_type["getLeader"] = &Player::getLeader;
            player_type["getName"] = &Player::getUsername;
            player_type["hasAI"] = &Player::hasAI;
            player_type["getCiv"] = &Player::getCiv;
            player_type["declareWarOn"] = [&] (Player &self, Player &opponent) {
                self.declareWarOn(opponent.getID(), *game);
            };
            player_type["isAtWarWith"] = [=] (Player &self, Player &opponent) {
                return self.isAtWarWith(opponent.getID());
            };
            player_type["isDead"] = &Player::isDead;
            player_type["getBaseRevenue"] = &Player::getBaseRevenue;
            player_type["getGoldRevenue"] = &Player::getGoldRevenue;
            player_type["getBeakerExpenses"] = &Player::getExpenses;
            player_type["getNetGold"] = &Player::getNetGold;
            player_type["getGold"] = &Player::getGold;
            player_type["getSciencePercent"] = &Player::getSciencePercent;
            player_type["setSciencePercent"] = [&] (Player &self, int sciencePercent) {
                self.setSciencePercent(sciencePercent, *game);
            };
            player_type["getScore"] = &Player::getScore;
            player_type["recomputeScore"] = [&] (Player &self) {
                self.recomputeScore(*game);
            };
            player_type["die"] = [&] (Player &self) {
                self.die(*game);
            };

            auto vec2_type = lua.new_usertype<glm::vec2>("Vec2", sol::constructors<glm::vec2(float, float)>());
            vec2_type["x"] = &glm::vec2::x;
            vec2_type["y"] = &glm::vec2::y;

            auto uvec2_type = lua.new_usertype<glm::uvec2>("UVec2", sol::constructors<glm::uvec2(uint32_t, uint32_t)>());
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
            game_type["getCityAtLocation"] = [=](Game &game, glm::uvec2 pos) {
                auto *city = game.getCityAtLocation(pos);
                return city;
            };
            game_type["getScreenOffset"] = &Game::getScreenOffset;
            game_type["getPosFromScreenOffset"] = &Game::getPosFromScreenOffset;

            auto cv_type = lua.new_usertype<Canvas>("Canvas");
            cv_type["beginPath"] = [] (Canvas &cv) {
                nvgBeginPath(cv.vg);
            };
            cv_type["rect"] = [] (Canvas &cv, float posX, float posY, float sizeX, float sizeY) {
                nvgRect(cv.vg, posX, posY, sizeX, sizeY);
            };
            cv_type["circle"] = [] (Canvas &cv, float cx, float cy, float radius) {
                nvgCircle(cv.vg, cx, cy, radius);
            };
            cv_type["lineTo"] = [] (Canvas &cv, float x, float y) {
                nvgLineTo(cv.vg, x, y);
            };
            cv_type["fillColor"] = [] (Canvas &cv, sol::table color) {
                nvgFillColor(cv.vg, luaColor(color));
            };
            cv_type["strokeColor"] = [] (Canvas &cv, sol::table color) {
                nvgStrokeColor(cv.vg, luaColor(color));
            };
            cv_type["strokeWidth"] = [] (Canvas &cv, float width) {
                nvgStrokeWidth(cv.vg, width);
            };
            cv_type["fill"] = [] (Canvas &cv) {
                nvgFill(cv.vg);
            };
            cv_type["stroke"] = [] (Canvas &cv) {
                nvgStroke(cv.vg);
            };
            cv_type["textFormat"] = [] (Canvas &cv, int baseline, int align) {
                nvgTextAlign(cv.vg, baseline | align);
            };
            cv_type["fontSize"] = [] (Canvas &cv, float size) {
                nvgFontSize(cv.vg, size);
            };
            cv_type["text"] = [] (Canvas &cv, float x, float y, const std::string &text) {
                nvgText(cv.vg, x, y, text.c_str(), nullptr);
            };

            lua["TextBaseline"] = lua.create_table_with(
                    "Alphabetic", NVG_ALIGN_BASELINE,
                    "Top", NVG_ALIGN_TOP,
                    "Bottom", NVG_ALIGN_BOTTOM,
                    "Middle", NVG_ALIGN_MIDDLE
                    );
            lua["TextAlign"] = lua.create_table_with(
                    "Left", NVG_ALIGN_LEFT,
                    "Center", NVG_ALIGN_CENTER,
                    "Right", NVG_ALIGN_RIGHT
                    );

            lua["Key"] = lua.create_table_with(
                    "Escape", GLFW_KEY_ESCAPE,
                    "Enter", GLFW_KEY_ENTER,
                    "LeftShift", GLFW_KEY_LEFT_SHIFT,
                    "RightShift", GLFW_KEY_RIGHT_SHIFT,
                    "Control", GLFW_KEY_LEFT_CONTROL,
                    "Alt", GLFW_KEY_LEFT_ALT,
                    "Tab", GLFW_KEY_TAB
                    );
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

    std::shared_ptr<Asset> ScriptLoader::loadAsset(const std::string &data) {
        engine->impl->lua.script(data);
        return std::make_shared<ScriptAsset>();
    }

    ScriptLoader::ScriptLoader(const std::shared_ptr<ScriptEngine> &engine) : engine(engine) {}
}
