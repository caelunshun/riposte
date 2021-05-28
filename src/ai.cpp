//
// Created by Caelum van Ispelen on 5/16/21.
//

#include "ai.h"
#include "game.h"
#include "ripmath.h"
#include "city.h"
#include "rng.h"
#include "tile.h"
#include "unit.h"
#include "path.h"
#include "player.h"
#include <glm/glm.hpp>
#include <deque>
#include <iostream>
#include <optional>
#include <absl/container/flat_hash_set.h>
#include <absl/container/flat_hash_map.h>

namespace rip {
    // An AI that controls a single unit.
    class UnitAI {
    protected:
        // The unit being controlled.
        UnitId unitID;

    public:
        explicit UnitAI(UnitId unitID) : unitID(unitID) {}

        virtual ~UnitAI() = default;

        virtual void doTurn(Game &game, AIimpl &ai, Player &player, Unit &unit) = 0;

        UnitId getUnitID() const {
            return unitID;
        }
    };

    // An AI that controls a single city.
    class CityAI {
        CityId cityID;
        void updateTask(Game &game, AIimpl &ai, Player &player, City &city);
    public:
        CityAI(CityId cityID) : cityID(cityID) {}

        CityId getCityID() const {
            return cityID;
        }

        void doTurn(Game &game, AIimpl &ai, Player &player, City &city);
    };

    class AIimpl {
        // A unit AI for each unit.
        std::vector<std::unique_ptr<UnitAI>> unitAIs;
        absl::flat_hash_set<UnitId> unitAISet;

        // A city AI for each city.
        std::vector<CityAI> cityAIs;
        absl::flat_hash_set<CityId> cityAISet;

        std::string playerName;

    public:
        // ID of the controlled player.
        PlayerId playerID;
        Rng rng;

        explicit AIimpl(PlayerId playerId) : playerID(playerId) {}

        std::unique_ptr<UnitAI> makeUnitAI(Unit &unit);

        void updateUnits(Game &game) {
            // Add new unit AIs for newly created units.
            for (auto &unit : game.getUnits()) {
                if (unit.getOwner() != playerID) continue;

                if (!unitAISet.contains(unit.getID())) {
                    auto ai = makeUnitAI(unit);
                    unitAIs.push_back(std::move(ai));
                    unitAISet.insert(unit.getID());
                }
            }

            if (!unitAIs.empty()) {
                for (int i = unitAIs.size() - 1; i >= 0; i--) {
                    auto &unitAI = unitAIs.at(i);
                    const auto unitID = unitAI->getUnitID();

                    if (!game.getUnits().id_is_valid(unitID)) {
                        // Unit died - delete its AI.
                        unitAIs.erase(unitAIs.begin() + i);
                        unitAISet.erase(unitID);
                        continue;
                    }

                    auto &unit = game.getUnit(unitID);
                    unit.moveAlongCurrentPath(game);
                    auto &player = game.getPlayer(playerID);
                    unitAI->doTurn(game, *this, player, unit);
                }
            }
        }

        void updateCities(Game &game, Player &player) {
            // Add new city AIs for newly created cities.
            for (const auto cityID : player.getCities()) {
                if (!cityAISet.contains(cityID)) {
                    cityAIs.emplace_back(cityID);
                    cityAISet.insert(cityID);
                }
            }

            // Update each city.
            if (!cityAIs.empty()) {
                for (int i = cityAIs.size() - 1; i >= 0; i--) {
                    auto &cityAI = cityAIs.at(i);
                    const auto cityID = cityAI.getCityID();

                    if (!game.getCities().id_is_valid(cityID) || game.getCity(cityID).getOwner() != playerID) {
                        // Lost the city - delete its AI.
                        cityAIs.erase(cityAIs.begin() + i);
                        cityAISet.erase(cityID);
                        continue;
                    }

                    auto &city = game.getCity(cityID);
                    auto &player = game.getPlayer(playerID);
                    cityAI.doTurn(game, *this, player, city);
                }
            }
        }

        void updateResearch(Game &game, Player &player);

        void log(std::string message) const {
            std::cout << "[ai-" << playerName << "] " << message << std::endl;
        }

        void doTurn(Game &game) {
            auto &player = game.getPlayer(playerID);
            playerName = player.getCiv().leader;
            updateUnits(game);
            updateCities(game, player);
            updateResearch(game, player);
        }
    };

    // UNIT

    class SettlerAI : public UnitAI {
    public:
        SettlerAI(const UnitId &unitId) : UnitAI(unitId) {}

        ~SettlerAI() override = default;

        void doTurn(Game &game, AIimpl &ai, Player &player, Unit &unit) override {
            auto &foundCityCap = *unit.getCapability<FoundCityCapability>();
            if (player.getCities().empty()) {
                // Settle NOW.
                foundCityCap.foundCity(game);
                ai.log(" founded city");
            }
        }
    };

    class ReconUnitAI : public UnitAI {
    public:
        ReconUnitAI(const UnitId &unitId) : UnitAI(unitId) {}

        ~ReconUnitAI() override = default;

        void doTurn(Game &game, AIimpl &ai, Player &player, Unit &unit) override {
            int attempts = 0;
            while (!unit.hasPath() && attempts < 10) {
                glm::uvec2 target(unit.getPos().x + static_cast<int>(ai.rng.u32(0, 20)) - 10,
                                  unit.getPos().y + static_cast<int>(ai.rng.u32(0, 20)) - 10);
                auto path = computeShortestPath(game, unit.getPos(), target, {});
                if (path.has_value()) {
                    unit.setPath(std::move(*path));
                }

                ++attempts;
            }
        }
    };

    std::unique_ptr<UnitAI> AIimpl::makeUnitAI(Unit &unit) {
        if (unit.hasCapability<FoundCityCapability>()) {
            return std::make_unique<SettlerAI>(unit.getID());
        } else {
            return std::make_unique<ReconUnitAI>(unit.getID());
        }
    }

    // CITY

    void CityAI::updateTask(Game &game, AIimpl &ai, Player &player, City &city) {
        if (city.getBuildTask()) return;

        auto worker = game.getRegistry().getUnits().at(2);
        city.setBuildTask(std::make_unique<UnitBuildTask>(worker));
    }

    void CityAI::doTurn(Game &game, AIimpl &ai, Player &player, City &city) {
        updateTask(game, ai, player, city);
    }

    // RESEARCH

    void AIimpl::updateResearch(Game &game, Player &player) {
        if (player.getResearchingTech().has_value()) return;

        auto options = player.getTechs().getPossibleResearches();
        if (options.empty()) return;

        auto &choice = options[rng.u32(0, options.size())];
        player.setResearchingTech(choice);
        log("researching " + choice->name);
    }

    AI::AI(PlayerId playerID) : impl(std::make_unique<AIimpl>(playerID)) {}

    void AI::doTurn(Game &game) {
        impl->doTurn(game);
    }

    AI::~AI() = default;

    AI::AI(AI &&other) = default;

    AI &AI::operator=(AI &&other) noexcept = default;
}
