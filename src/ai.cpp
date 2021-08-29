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

    // An AI's plan for an upcoming or ongoing war.
    struct WarPlan {
        // The player to attack.
        PlayerId opponent;

        // The city to target.
        CityId targetCityID;

        // The location to gather troops for the attack.
        CityId gatherCityID;

        // Whether troops are already en route from gatherCity to targetCity.
        bool enRoute = false;

        // Whether troops are to attack the city this turn.
        bool shouldAttack = false;

        // The units that are ready to attack (i.e., in position in gatherCity)
        absl::flat_hash_set<UnitId> readyUnits;

        // The units that are next to the targetCity and can attack on the next turn
        absl::flat_hash_set<UnitId> attackingUnits;

        std::optional<CityId> findNewTargetCity(Game &game, AIimpl &ai, Player &player);

        void updateGatherCity(Game &game, AIimpl &ai, Player &player);

        // Updates the war plan. Returns whether the opponent is still valid.
        bool update(Game &game, AIimpl &ai, Player &player);

        void setTargetCity(Game &game, AIimpl &ai, Player &player, CityId newTarget);

        int getNeededUnitCount(Game &game);

        PlayerId findBestOpponent(Game &game, AIimpl &ai, Player &player);
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

        // The current war plan.
        // Only applicable if goal == Goal::ExpandWar.
        WarPlan warPlan;

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

        WarPlan &getWarPlan() {
            return warPlan;
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

                    if (!game.getUnits().contains(unitID)) {
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

                    if (!game.getCities().contains(cityID) || game.getCity(cityID).getOwner() != playerID) {
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
            return (static_cast<double>(player.getBaseRevenue()) / player.getExpenses() >= 1.2 && player.getBeakerRevenue() >= 10);
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

        void setGoal(Goal newGoal, Game &game, Player &player) {
            if (goal != newGoal) {
                const char *goalNames[3] = {"ExpandPeacefully", "ExpandWar", "Thrive"};
                log("NEW GOAL: " + std::string(goalNames[static_cast<int>(newGoal)]));

                if (newGoal == Goal::ExpandWar) {
                    warPlan.opponent = warPlan.findBestOpponent(game, *this, player);
                    warPlan.setTargetCity(game, *this, player, *warPlan.findNewTargetCity(game, *this, player));
                    log("PLOTTING WAR against " + game.getPlayer(warPlan.opponent).getLeader().name);
                }
            }
            goal = newGoal;
        }

        void updateGoal(Game &game, Player &player) {
            if (isPeacefulExpansionExhausted && goal == Goal::ExpandPeacefully && needsExpansion(game, player)) {
                setGoal(Goal::ExpandWar, game, player);
            }

            if (hasBaseDesiredCities(game, player) && isEconomyReadyForWar(game, player)) {
                setGoal(Goal::ExpandWar, game, player);
            }

            if (goal == Goal::ExpandPeacefully && !isEconomyReadyForWar(game, player) && hasBaseDesiredCities(game, player)) {
                setGoal(Goal::Thrive, game, player);
            }
        }

        void updateResearch(Game &game, Player &player);

        void log(std::string message) const {
            //if (playerID == thePlayerID) {
                std::cerr << "[ai-" << playerName << "] " << message << std::endl;
            //}
        }

        // Gets the closest city (owned by anyone) to the given position.
        std::pair<double, CityId> getDistanceToNearestCity(const Game &game, glm::uvec2 pos, bool onlyOurs) {
            double bestDist = 1000000;
            CityId bestCity;

            for (const auto &city : game.getCities()) {
                if (onlyOurs && city.getOwner() != playerID) continue;
                const auto d = dist(pos, city.getPos());
                if (d < bestDist) {
                    bestDist = d;
                    bestCity = city.getID();
                }
            }

            return {bestDist, bestCity};
        }

        void updateWarPlan(Game &game, Player &player) {
            if (goal != Goal::ExpandWar) return;

            if (warPlan.update(game, *this, player)) {
                // We finished this player off. Switch to thriving
                setGoal(Goal::Thrive, game, player);
            }
        }

        void doTurn(Game &game) {
            auto &player = game.getPlayer(playerID);
            playerName = player.getLeader().name;
            thePlayerID = game.getThePlayerID();
            updateWarPlan(game, player);
            updateGoal(game, player);
            updateUnits(game);
            updateCities(game, player);
            updateResearch(game, player);
        }
    };

    // WAR
    PlayerId WarPlan::findBestOpponent(Game &game, AIimpl &ai, Player &player) {
        // Find opponent with closest capital.
        std::optional<PlayerId> closest;
        double closestDist;
        for (const auto &player : game.getPlayers()) {
            if (player.getID() == ai.playerID) continue;
            if (player.isDead()) continue;
            if (player.getCities().empty()) continue;

            double dist = ai.getDistanceToNearestCity(game, game.getCity(player.getCapital()).getPos(), true).first;
            if (!closest.has_value() || dist < closestDist) {
                closestDist = dist;
                closest = player.getID();
            }
        }

        return *closest;
    }

    std::optional<CityId> WarPlan::findNewTargetCity(Game &game, AIimpl &ai, Player &player) {
        // Find closest city to our land.
        std::optional<CityId> result;
        double resultDist;
        for (const auto cityID : game.getPlayer(opponent).getCities()) {
            const auto &city = game.getCity(cityID);
            const auto dist = ai.getDistanceToNearestCity(game, city.getPos(), true).first;
            if (!result.has_value() || dist < resultDist) {
                resultDist = dist;
                result = cityID;
            }
        }
        return result;
    }

    void WarPlan::updateGatherCity(Game &game, AIimpl &ai, Player &player) {
        gatherCityID = ai.getDistanceToNearestCity(game, game.getCity(targetCityID).getPos(), true).second;
    }

    void WarPlan::setTargetCity(Game &game, AIimpl &ai, Player &player, CityId newTarget) {
        if (newTarget != targetCityID) {
            ai.log("WAR: targeting " + game.getCity(newTarget).getName());
            targetCityID = newTarget;
            updateGatherCity(game, ai, player);
            enRoute = false;
            shouldAttack = false;
            attackingUnits.clear();
            readyUnits.clear();
        }
    }

    bool WarPlan::update(Game &game, AIimpl &ai, Player &player) {
        // Check that opponent isn't dead.
        if (game.getPlayer(opponent).isDead()) return true;

        ai.log("war plan: ready = " + std::to_string(readyUnits.size()) + ", attacking = " + std::to_string(attackingUnits.size())
            + ", enRoute = " + std::to_string(enRoute) + ", shouldAttack = " + std::to_string(shouldAttack));
        // Check that the target city is still owned by the opponent.
        const auto &targetCity = game.getCity(targetCityID);
        if (targetCity.getOwner() != opponent) {
            auto newTarget = findNewTargetCity(game, ai, player);
            if (newTarget.has_value()) {
                setTargetCity(game, ai, player, *newTarget);
            } else {
                return true;
            }
        }

        // Check that the target city is still the best city.
        auto bestCity = findNewTargetCity(game, ai, player);
        if (bestCity.has_value() && *bestCity != targetCityID) {
            setTargetCity(game, ai, player, *bestCity);
        }

        if (readyUnits.size() >= getNeededUnitCount(game)
            && !player.isAtWarWith(opponent)) {
            // We're ready for war.
            player.declareWarOn(opponent, game);
        }

        if (readyUnits.size() >= getNeededUnitCount(game)) {
            enRoute = true;
        } else {
            enRoute = false;
        }

        if (attackingUnits.size() >= getNeededUnitCount(game)
            && player.isAtWarWith(opponent)) {
            shouldAttack = true;
        } else {
            shouldAttack = false;
        }

        return false;
    }

    // Gets the number of units we need to take the city.
    int WarPlan::getNeededUnitCount(Game &game) {
        const auto targetStack = game.getStackByKey(opponent, game.getCity(targetCityID).getPos());
        if (targetStack.has_value()) {
            return game.getStack(*targetStack).getUnits().size();
        } else {
            return 2;
        }
    }

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
            double distanceFactor = 2 * -pow(ai.getDistanceToNearestCity(game, tilePos, true).first - optimalDist, 2) + 5;

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
            if (ai.getDistanceToNearestCity(game, tilePos, false).first < minDist) {
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
                    auto path = computeShortestPath(game, unit.getPos(), *targetPos, {}, unit.getKind(), player.getID());
                    if (path.has_value()) {
                        unit.setPath(std::move(*path));
                        ai.log("settler pathfinded to new city location");
                    } else {
                        ai.isPeacefulExpansionExhausted = true;
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
            if (tile.isForested() && task.getImprovement().getName() == "Mine") {
                suitabilityFactor = 2;
            } else if (tile.getTerrain() == Terrain::Plains && task.getImprovement().getName() == "Farm") {
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
                auto path = computeShortestPath(game, unit.getPos(), bestTask->getPos(), {}, unit.getKind(), player.getID());
                if (path.has_value()) {
                    ai.log("worker chose to build " + bestTask->getImprovement().getName());
                    ai.claimedWorkerTiles.insert(bestTask->getPos());

                    targetPos = bestTask->getPos();
                    targetTask = std::move(bestTask);
                    unit.setPath(std::move(*path));
                }
            } else {
                auto capitalPos = game.getCity(player.getCapital()).getPos();
                if (unit.getPos() != capitalPos) {
                    auto path = computeShortestPath(game, unit.getPos(), capitalPos, {}, unit.getKind(), player.getID());
                    if (path) {
                        unit.setPath(std::move(*path));
                    }
                }
            }
        }
    };

    class MilitaryGroundUnitAI : public UnitAI {
    public:
        MilitaryGroundUnitAI(const UnitId &unitId) : UnitAI(unitId) {}

        ~MilitaryGroundUnitAI() override = default;

        void doTurn(Game &game, AIimpl &ai, Player &player, Unit &unit) override {
            // Stay in the city if it needs protection.
            const auto minCityUnits = 2;
            auto *city = game.getCityAtLocation(unit.getPos());
            if (city && city->getOwner() == ai.playerID) {
                auto stackID = game.getStackByKey(ai.playerID, unit.getPos());
                if (stackID.has_value() && game.getStack(*stackID).getUnits().size() <= minCityUnits) {
                    unit.fortify();
                    return;
                }
            }

            if (unit.isFortified()) return;

            if (ai.getGoal() == Goal::ExpandWar) {
                // Follow the war plan.
                auto &plan = ai.getWarPlan();
                const auto targetCityPos = game.getCity(plan.targetCityID).getPos();
                const auto gatherCityPos = game.getCity(plan.gatherCityID).getPos();

                glm::uvec2 targetPos;
                if (plan.enRoute) {
                    targetPos = targetCityPos;
                } else {
                    targetPos = gatherCityPos;
                }

                if (!unit.hasPath() || unit.getPath().getDestination() != targetPos) {
                    auto path = computeShortestPath(game, unit.getPos(), targetPos, {}, unit.getKind(), player.getID());
                    if (path.has_value()) {
                        unit.setPath(std::move(*path));
                    } else {
                        ai.log("can't pathfind to gather location");
                    }
                }

                if (!plan.enRoute && unit.getPos() == gatherCityPos) {
                    plan.readyUnits.insert(unit.getID());
                } else if (!plan.enRoute) {
                    plan.readyUnits.erase(unit.getID());
                }

                if (plan.enRoute && isAdjacent(unit.getPos(), targetCityPos)) {
                    plan.attackingUnits.insert(unit.getID());
                    plan.readyUnits.insert(unit.getID());

                    // Attack if we can.
                    if (plan.shouldAttack) {
                        ai.log("unit ATTACKING city");
                        unit.moveTo(targetCityPos, game, true);
                    } else {
                        ai.log("unit READY but NOT attacking");
                    }
                } else {
                    plan.attackingUnits.erase(unit.getID());
                }
            } else {
                // Scout randomly.
                int attempts = 0;
                while (!unit.hasPath() && attempts < 10) {
                    glm::uvec2 target(unit.getPos().x + static_cast<int>(ai.rng.u32(0, 20)) - 10,
                                      unit.getPos().y + static_cast<int>(ai.rng.u32(0, 20)) - 10);
                    auto path = computeShortestPath(game, unit.getPos(), target, {}, unit.getKind(), player.getID());
                    if (path.has_value()) {
                        unit.setPath(std::move(*path));
                    }

                    ++attempts;
                }
            }
        }

        void onDeath(Game &game, AIimpl &ai, Player &player) override {
            ai.getWarPlan().readyUnits.erase(unitID);
            ai.getWarPlan().attackingUnits.erase(unitID);
        }
    };

    std::unique_ptr<UnitAI> AIimpl::makeUnitAI(Unit &unit) {
        if (unit.hasCapability<FoundCityCapability>()) {
            return std::make_unique<SettlerAI>(unit.getID());
        } else if (unit.hasCapability<WorkerCapability>()) {
            return std::make_unique<WorkerAI>(unit.getID());
        } else {
            return std::make_unique<MilitaryGroundUnitAI>(unit.getID());
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
        std::optional<BuildingBuildTask> building;

        const auto &registry = game.getRegistry();
        if (ai.getGoal() == Goal::ExpandPeacefully && ai.settlerCount == 0 && game.getTurn() != 0) {
            ++ai.settlerCount;
            unitToBuild = registry.getUnit("settler");
        } else if (ai.getGoal() == Goal::Thrive) {
            BuildingBuildTask buildMarket(registry.getBuilding("Market"));
            BuildingBuildTask buildLibrary(registry.getBuilding("Library"));

            const auto commerce = city.computeYield(game).commerce;
            if (commerce >= 8 && buildMarket.canBuild(game, city)) {
                building = std::move(buildMarket);
            } else if (commerce >= 8 && buildLibrary.canBuild(game, city)) {
                building = std::move(buildLibrary);
            } else {
                unitToBuild = registry.getUnit("worker");
            }
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

        BuildingBuildTask granary(registry.getBuilding("Granary"));
        if (ai.getGoal() != Goal::ExpandWar && granary.canBuild(game, city) && game.getTurn() > 60) {
            building = std::move(granary);
        }

        if (building.has_value()) {
            ai.log("city building " + building->getBuilding()->name);
            city.setBuildTask(std::make_unique<BuildingBuildTask>(std::move(*building)));
            ++buildIndex;
        } else if (unitToBuild) {
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
    static const char *researchOrder[9] = {
            "Agriculture",
            "Pottery",
            "Mining",
            "The Wheel",
            "Bronze Working",
            "Writing",
            "Alphabet",
            "Mathematics",
            "Currency"
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
