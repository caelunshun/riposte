//
// Created by Caelum van Ispelen on 5/16/21.
//

// The AI powerhouse.

#include "ai.h"
#include "game.h"
#include "ripmath.h"
#include "city.h"
#include "rng.h"
#include "tile.h"
#include "unit.h"
#include "path.h"
#include "player.h"
#include "worker.h"
#include "stack.h"
#include "traversal.h"
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

        virtual void onDeath(Game &game, AIimpl &ai, Player &player) {}

        virtual void doTurn(Game &game, AIimpl &ai, Player &player, Unit &unit) = 0;

        UnitId getUnitID() const {
            return unitID;
        }
    };

    // An AI that controls a single city.
    class CityAI {
        CityId cityID;
        int buildIndex = 0;
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

        absl::flat_hash_set<glm::uvec2, PosHash> claimedWorkerTiles;
        absl::flat_hash_set<glm::uvec2, PosHash> claimedSettlerTiles;

        explicit AIimpl(PlayerId playerId) : playerID(playerId) {}

        std::unique_ptr<UnitAI> makeUnitAI(Unit &unit);

        const absl::flat_hash_set<CityId> &getCities() const {
            return cityAISet;
        }

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

            auto &player = game.getPlayer(playerID);
            if (!unitAIs.empty()) {
                for (int i = unitAIs.size() - 1; i >= 0; i--) {
                    auto &unitAI = unitAIs.at(i);
                    const auto unitID = unitAI->getUnitID();

                    if (!game.getUnits().id_is_valid(unitID)) {
                        // Unit died - delete its AI.
                        unitAI->onDeath(game, *this, player);
                        unitAIs.erase(unitAIs.begin() + i);
                        unitAISet.erase(unitID);
                        continue;
                    }

                    auto &unit = game.getUnit(unitID);
                    unit.moveAlongCurrentPath(game);
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
                    cityAI.doTurn(game, *this, player, city);
                }
            }
        }

        void updateResearch(Game &game, Player &player);

        void log(std::string message) const {
            std::cout << "[ai-" << playerName << "] " << message << std::endl;
        }

        // Gets the closest city (owned by anyone) to the given position.
        std::pair<double, CityId> getDistanceToNearestCity(const Game &game, glm::uvec2 pos) {
            double bestDist = 1000000;
            CityId bestCity;

            for (const auto &city : game.getCities()) {
                const auto d = dist(pos, city.getPos());
                if (d < bestDist) {
                    bestDist = d;
                    bestCity = city.getID();
                }
            }

            return {bestDist, bestCity};
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
        std::optional<glm::uvec2> targetPos;
        absl::flat_hash_set<glm::uvec2, PosHash> blacklist;

    public:
        SettlerAI(const UnitId &unitId) : UnitAI(unitId) {}

        ~SettlerAI() override = default;

        // Returns a rating for a tile.
        double rateCityLocation(Game &game, AIimpl &ai, Unit &unit, const Tile &tile, glm::uvec2 tilePos) {
            const double optimalDist = 6;
            const auto minDist = 3;
            double distanceFactor = 2 * -pow(ai.getDistanceToNearestCity(game, tilePos).first - optimalDist, 2) + 5;

            double tileFactor = 0;
            if (tile.getTerrain() == Terrain::Desert) {
                tileFactor = -10;
            }

            double resourceFactor = 0;
            for (const auto bfcTile : getBigFatCross(tilePos)) {
                if (!game.containsTile(bfcTile)) continue;

                if (game.getTile(bfcTile).hasResource()) {
                    resourceFactor += 3;
                }
            }

            double existingCityFactor = 0;
            if (ai.getDistanceToNearestCity(game, tilePos).first < minDist) {
                existingCityFactor = -100000;
            }

            double blacklistFactor = 0;
            if (blacklist.contains(tilePos)) {
                blacklistFactor = -100000;
            }

            return distanceFactor + tileFactor + resourceFactor + existingCityFactor + blacklistFactor;
        }

        // Finds the best location to build a city based on rateCityLocation.
        std::optional<glm::uvec2> findBestCityLocation(Game &game, AIimpl &ai, Player &player, Unit &unit) {
            std::optional<glm::uvec2> result;
            double bestRating;

            const auto maxDistFromBorder = 10;

            // Custom breadth-first search to track the distance from our cultural borders.
            struct Entry {
                glm::uvec2 pos;
                int distFromBorder;

                Entry(const glm::uvec2 &pos, int distFromBorder) : pos(pos), distFromBorder(distFromBorder) {}
            };

            std::deque<Entry> entries;
            entries.emplace_back(game.getCity(player.getCapital()).getPos(), 0);

            absl::flat_hash_set<glm::uvec2, PosHash> visited;

            while (!entries.empty()) {
                auto entry = entries[0];
                entries.pop_front();

                const double rating = rateCityLocation(game, ai, unit, game.getTile(entry.pos), entry.pos);
                if (rating >= -100 && (!result.has_value() || rating > bestRating)) {
                    result = entry.pos;
                    bestRating = rating;
                }

                for (const auto neighborPos : getSideNeighbors(entry.pos)) {
                    if (visited.contains(neighborPos)) continue;

                    if (!game.containsTile(neighborPos)) continue;

                    const auto &neighbor = game.getTile(neighborPos);
                    if (neighbor.getTerrain() == Terrain::Ocean) continue;

                    const auto owner = game.getCultureMap().getTileOwner(neighborPos);
                    if (owner.has_value() && owner != ai.playerID) {
                        // can't settle into opponent land
                        continue;
                    }

                    Entry newEntry(neighborPos, entry.distFromBorder + 1);
                    if (owner == ai.playerID) {
                        newEntry.distFromBorder = 0;
                    }

                    if (newEntry.distFromBorder > maxDistFromBorder) continue;

                    entries.push_back(newEntry);
                    visited.insert(newEntry.pos);
                }
            }

            return result;
        }

        void doTurn(Game &game, AIimpl &ai, Player &player, Unit &unit) override {
            auto &foundCityCap = *unit.getCapability<FoundCityCapability>();
            if (player.getCities().empty() || (targetPos.has_value() && unit.getPos() == targetPos)) {
                // Settle NOW.
                if (foundCityCap.foundCity(game)) {
                    ai.log(" founded city");
                    return;
                } else {
                    targetPos = {};
                }
            }

            if (!targetPos.has_value()) {
                targetPos = findBestCityLocation(game, ai, player, unit);
                if (targetPos.has_value()) {
                    auto path = computeShortestPath(game, unit.getPos(), *targetPos, {});
                    if (path.has_value()) {
                        unit.setPath(std::move(*path));
                        ai.log("settler pathfinded to new city location");
                    } else {
                        blacklist.insert(*targetPos);
                        targetPos = {};
                    }
                }
            }
        }
    };

    class WorkerAI : public UnitAI {
        glm::uvec2 targetPos;
        std::optional<BuildImprovementTask> targetTask;

    public:
        WorkerAI(const UnitId &unitId) : UnitAI(unitId) {}

        ~WorkerAI() override = default;

        double rateTask(Game &game, AIimpl &ai, Unit &unit, glm::uvec2 pos, const BuildImprovementTask &task) {
            double distFactor = -dist(unit.getPos(), pos);

            double resourceFactor = 0;
            const auto &tile = game.getTile(pos);
            if (tile.hasImproveableResource(task.getImprovement().getName())) {
                resourceFactor = 10;
            }

            double suitabilityFactor = -2;
            if (tile.getTerrain() == Terrain::Plains && task.getImprovement().getName() == "Farm") {
                suitabilityFactor = 2;
            } else if (tile.getTerrain() == Terrain::Grassland && task.getImprovement().getName() == "Cottage") {
                suitabilityFactor = 4;
            }

            return distFactor + resourceFactor + suitabilityFactor;
        }

        void onDeath(Game &game, AIimpl &ai, Player &player) override {
            ai.claimedWorkerTiles.erase(targetPos);
        }

        void doTurn(Game &game, AIimpl &ai, Player &player, Unit &unit) override {
            auto &workCap = *unit.getCapability<WorkerCapability>();

            if (unit.getPos() == targetPos && targetTask.has_value()) {
                ai.log("worker started building " + targetTask->getImprovement().getName());
                workCap.setTask(std::make_unique<BuildImprovementTask>(std::move(*targetTask)));
                targetTask = {};
            }

            if (workCap.getTask() != nullptr) return;

            ai.claimedWorkerTiles.erase(targetPos);

            // Find the best task to complete, based on the values of rateTask().
            std::optional<BuildImprovementTask> bestTask;
            double bestRating;

            for (const auto cityID : ai.getCities()) {
                const auto &city = game.getCity(cityID);
                for (const auto tilePos : getBigFatCross(city.getPos())) {
                    if (!game.containsTile(tilePos)) continue;
                    if (ai.claimedWorkerTiles.contains(tilePos)) continue;
                    if (game.getCultureMap().getTileOwner(tilePos) != ai.playerID) continue;

                    const auto &tile = game.getTile(tilePos);
                    for (auto &improvement : tile.getPossibleImprovements(game, tilePos)) {
                        if (!player.getTechs().isImprovementUnlocked(improvement->getName())) continue;

                        const auto buildTurns = improvement->getNumBuildTurns();
                        BuildImprovementTask task(buildTurns, tilePos, std::move(improvement));
                        double rating = rateTask(game, ai, unit, tilePos, task);

                        if (!bestTask.has_value() || rating > bestRating) {
                            bestTask = std::move(task);
                            bestRating = rating;
                        }
                    }
                }
            }

            if (bestTask.has_value()) {
                auto path = computeShortestPath(game, unit.getPos(), bestTask->getPos(), {});
                if (path.has_value()) {
                    ai.log("worker chose to build " + bestTask->getImprovement().getName());
                    ai.claimedWorkerTiles.insert(bestTask->getPos());

                    targetPos = bestTask->getPos();
                    targetTask = std::move(bestTask);
                    unit.setPath(std::move(*path));
                }
            }
        }
    };

    class ReconUnitAI : public UnitAI {
    public:
        ReconUnitAI(const UnitId &unitId) : UnitAI(unitId) {}

        ~ReconUnitAI() override = default;

        void doTurn(Game &game, AIimpl &ai, Player &player, Unit &unit) override {
            // Stay in the city if it needs protection.
            const auto minCityUnits = 6;
            auto *city = game.getCityAtLocation(unit.getPos());
            if (city && city->getOwner() == ai.playerID) {
                auto stackID = game.getStackByKey(ai.playerID, unit.getPos());
                if (stackID.has_value() && game.getStack(*stackID).getUnits().size() <= minCityUnits) {
                    unit.fortify();
                    return;
                }
            }

            if (unit.isFortified()) return;

            // Scout randomly.
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
        } else if (unit.hasCapability<WorkerCapability>()) {
            return std::make_unique<WorkerAI>(unit.getID());
        } else {
            return std::make_unique<ReconUnitAI>(unit.getID());
        }
    }

    // CITY

    void CityAI::updateTask(Game &game, AIimpl &ai, Player &player, City &city) {
        if (city.getBuildTask()) return;

        int currentProtectionCount = 0;
        auto stackID = game.getStackByKey(player.getID(), city.getPos());
        if (stackID) {
            currentProtectionCount = game.getStack(*stackID).getUnits().size();
        }

        std::shared_ptr<UnitKind> unitToBuild;
        if (buildIndex != 0 && (currentProtectionCount < 2 || (buildIndex + 3) % 5 < 3)) {
            // best military unit we can build
            std::optional<UnitBuildTask> bestTask;
            for (const auto &unitKind : game.getRegistry().getUnits()) {
                UnitBuildTask task(unitKind);
                if (task.canBuild(game, city) &&
                        (!bestTask.has_value() || unitKind->strength > bestTask->getUnitKind()->strength)) {
                    bestTask = std::move(task);
                }
            }

            if (bestTask.has_value()) {
                unitToBuild = bestTask->getUnitKind();
            }
        } else if ((buildIndex + 3) % 5 < 4) {
            // worker
            unitToBuild = game.getRegistry().getUnits().at(2);
        } else {
            // settler
            unitToBuild = game.getRegistry().getUnits().at(0);
        }

        if (unitToBuild) {
            city.setBuildTask(std::make_unique<UnitBuildTask>(unitToBuild));
            ++buildIndex;
        }
    }

    void CityAI::doTurn(Game &game, AIimpl &ai, Player &player, City &city) {
        updateTask(game, ai, player, city);
    }

    // RESEARCH

    void AIimpl::updateResearch(Game &game, Player &player) {
        if (player.getResearchingTech().has_value()) return;

        auto options = player.getTechs().getPossibleResearches();
        if (options.empty()) {
            log("teched out on turn " + std::to_string(game.getTurn()));
            return;
        }

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
