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

        virtual void doTurn(Game &game, AIimpl &ai, Player &player, Unit &unit) = 0;

        virtual void onCreated(Game &game, AIimpl &ai, Player &player, Unit &unit) {}

        virtual void onDeath(Game &game, AIimpl &ai, Player &player) {}

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

    // EMPIRE

    // A global goal for the empire in the next 50-100 turns.
    enum class Goal {
        // Expand with settlers
        ExpandPeacefully,
        // Expand with the sword
        ExpandWar,
        // Improve economy
        Thrive,
    };

    class AIimpl {
        // A unit AI for each unit.
        std::vector<std::unique_ptr<UnitAI>> unitAIs;
        absl::flat_hash_set<UnitId> unitAISet;

        // A city AI for each city.
        std::vector<CityAI> cityAIs;
        absl::flat_hash_set<CityId> cityAISet;

        std::string playerName;

        // The current long-term goal.
        Goal goal = Goal::ExpandPeacefully;

        PlayerId thePlayerID;

    public:
        // ID of the controlled player.
        PlayerId playerID;
        Rng rng;

        absl::flat_hash_set<glm::uvec2, PosHash> claimedWorkerTiles;
        absl::flat_hash_set<glm::uvec2, PosHash> claimedSettlerTiles;

        bool isPeacefulExpansionExhausted = false;

        // The number of settlers we own. *This
        // includes settlers currently being built.*
        int settlerCount = 1; // starts at 1 because of starting settler

        explicit AIimpl(PlayerId playerId) : playerID(playerId) {}

        Goal getGoal() const {
            return goal;
        }

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
                    unit.moveAlongCurrentPath(game, true);
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

        bool isEconomyReadyForWar(Game &game, Player &player) const {
            return (static_cast<double>(player.getBaseRevenue()) / player.getExpenses() >= 1.5 && player.getBeakerRevenue() >= 10);
        }

        bool hasBaseDesiredCities(Game &game, Player &player) const {
            const auto numCities = getCities().size();

            const auto baseDesiredCities = round(4 + 5 * (player.getLeader().expansiveness / 10 - 0.2));

            return (numCities >= baseDesiredCities);
        }

        bool needsExpansion(Game &game, Player &player) const {
            if (!hasBaseDesiredCities(game, player)) return true;

            // We have the base desired cities - but more expansion might still
            // be good if we're thriving economically.
            return isEconomyReadyForWar(game, player);
        }

        void setGoal(Goal newGoal) {
            if (goal != newGoal) {
                const char *goalNames[3] = {"ExpandPeacefully", "ExpandWar", "Thrive"};
                log("NEW GOAL: " + std::string(goalNames[static_cast<int>(newGoal)]));
            }
            goal = newGoal;
        }

        void updateGoal(Game &game, Player &player) {
            if (isPeacefulExpansionExhausted && goal == Goal::ExpandPeacefully && needsExpansion(game, player)) {
                setGoal(Goal::ExpandWar);
            }

            if (hasBaseDesiredCities(game, player) && isEconomyReadyForWar(game, player)) {
                setGoal(Goal::ExpandWar);
            }

            if (goal == Goal::ExpandPeacefully && !isEconomyReadyForWar(game, player) && hasBaseDesiredCities(game, player)) {
                setGoal(Goal::Thrive);
            }
        }

        void updateResearch(Game &game, Player &player);

        void log(std::string message) const {
            if (playerID == thePlayerID) {
                std::cout << "[ai-" << playerName << "] " << message << std::endl;
            }
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
            playerName = player.getLeader().name;
            thePlayerID = game.getThePlayerID();
            updateGoal(game, player);
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
                } else {
                    ai.isPeacefulExpansionExhausted = true;
                }
            }
        }

        void onDeath(Game &game, AIimpl &ai, Player &player) override {
            --ai.settlerCount;
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
                suitabilityFactor = 2;
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

        std::shared_ptr<UnitKind> bestMilitaryUnit;
        for (const auto &unitKind : game.getRegistry().getUnits()) {
            UnitBuildTask task(unitKind);
            if (task.canBuild(game, city) &&
                (!bestMilitaryUnit || unitKind->strength > bestMilitaryUnit->strength)) {
                bestMilitaryUnit = task.getUnitKind();
            }
        }

        std::shared_ptr<UnitKind> unitToBuild;

        const auto &registry = game.getRegistry();
        if (ai.getGoal() == Goal::ExpandPeacefully && ai.settlerCount == 0 && game.getTurn() != 0) {
            ++ai.settlerCount;
            unitToBuild = registry.getUnit("settler");
        } else if (ai.getGoal() == Goal::Thrive) {
            unitToBuild = registry.getUnit("worker");
        } else if (ai.getGoal() == Goal::ExpandWar) {
            unitToBuild = bestMilitaryUnit;
        } else {
            // pick between worker or best military unit
            if (game.getTurn() == 0 || buildIndex % 3 >= 1) {
                unitToBuild = registry.getUnit("worker");
            } else {
                unitToBuild = bestMilitaryUnit;
            }
        }

        if (unitToBuild) {
            ai.log("city building " + unitToBuild->name + " (settlers=" + std::to_string(ai.settlerCount) + ", goal="
                + std::to_string(static_cast<int>(ai.getGoal())) + ")");
            city.setBuildTask(std::make_unique<UnitBuildTask>(unitToBuild));
            ++buildIndex;
        }
    }

    void CityAI::doTurn(Game &game, AIimpl &ai, Player &player, City &city) {
        updateTask(game, ai, player, city);
    }

    // RESEARCH

    // Predefined techs that are important to research.
    static const char *researchOrder[5] = {
            "Agriculture",
            "Pottery",
            "Mining",
            "The Wheel",
            "Bronze Working",
    };

    void AIimpl::updateResearch(Game &game, Player &player) {
        if (player.getResearchingTech().has_value()) return;

        auto options = player.getTechs().getPossibleResearches();
        if (options.empty()) {
            log("teched out on turn " + std::to_string(game.getTurn()));
            return;
        }

        std::shared_ptr<Tech> choice;
        bool finished = false;
        // Check prioritized techs.
        for (const auto prioritized : researchOrder) {
            if (finished) break;
            for (const auto &option : options) {
                if (option->name == prioritized) {
                    choice = option;
                    finished = true;
                    break;
                }
            }
        }

        if (!choice) {
            // Choose by fair die roll.
            choice = options[rng.u32(0, options.size())];
        }

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
