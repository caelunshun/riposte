//
// Created by Caelum van Ispelen on 5/16/21.
//

#include "ai.h"
#include "game.h"
#include "ripmath.h"
#include "city.h"
#include "rng.h"
#include <glm/glm.hpp>
#include <deque>
#include <iostream>
#include <optional>
#include <absl/container/flat_hash_set.h>
#include <absl/container/flat_hash_map.h>

namespace rip {
    // The AI consists of two different "brains," each with
    // its own role. They are:
    // 1. The long-term strategy. Maintains a "goal" for the next hundred turns
    // or so. (See the Goal class.) Goals include expansion through peace, expansion
    // through war, and economic recovery. This brain never interacts directly with
    // the game state.
    // 2. The short-term strategy. This is responsible for executing the goals of the long-term
    // strategy. In particular, it determines what to build in cities, and it
    // directs the tactical brain toward targets.
    // 3. The tactical brain. This is responsible for actually moving units.
    // It responds to high-level commands from the short-term strategy, like "attack this city"
    // or "settle over there."

    // A goal set by the long term brain.
    struct Goal {
        virtual ~Goal() = default;

        virtual bool requestsSettlerExpansion() const {
            return false;
        }
    };

    // A goal to expand peacefully (through settlement)
    struct ExpandPeacefully: public Goal {
        bool requestsSettlerExpansion() const override {
            return true;
        }
    };

    // A goal to improve our economy.
    struct EconomyGoal : public Goal {};

    class LongTermBrain {
        std::unique_ptr<Goal> goal;
        PlayerId playerID;

    public:
        LongTermBrain(PlayerId playerID) : playerID(playerID) {
            goal = std::make_unique<ExpandPeacefully>();
        }

        void update(Game &game) {
            if (game.getPlayer(playerID).getCities().size() == 4) {
                goal = std::make_unique<EconomyGoal>();
            }
        }

        const Goal &getGoal() const {
            return *goal;
        }
    };

    // A command created by the short term brain.
    struct TacticalCommand {
        virtual ~TacticalCommand() = default;
    };

    // Directs the tactical brain to settle a city at a location
    // using the next possible settler.
    struct SettleCityCommand : public TacticalCommand {
        glm::uvec2 settleLocation;

        SettleCityCommand(const glm::uvec2 &settleLocation) : settleLocation(settleLocation) {}
    };

    class ShortTermBrain {
        std::deque<std::unique_ptr<TacticalCommand>> commands;
        PlayerId playerID;

        std::optional<glm::uvec2> getBestCityLocation(Game &game) {
            // Breadth-first search on the capital.
            const auto &player = game.getPlayer(playerID);
            glm::uvec2 capitalPos;
            if (player.getCities().empty()) {
                for (const auto &unit : game.getUnits()) {
                    if (unit.getOwner() == playerID) {
                        capitalPos = unit.getPos();
                        break;
                    }
                }
            } else {
                capitalPos = game.getCity(player.getCities().at(0)).getPos();
            }

            std::vector<glm::uvec2> cityPositions;
            for (const auto &city : game.getCities()) {
                cityPositions.push_back(city.getPos());
            }

            std::deque<glm::uvec2> queue;
            queue.push_back(capitalPos);

            using Pos = std::pair<uint32_t, uint32_t>;
            absl::flat_hash_set<Pos> visited;

            while (!queue.empty()) {
                auto current = queue[0];
                queue.pop_front();

                bool isTooFar = false;
                for (const auto otherPos : cityPositions) {
                    if (dist(current, otherPos) <= 6) {
                        isTooFar = true;
                        break;
                    }
                }

                if (!isTooFar) {
                    return current;
                }

                // Add neighbors
                for (const auto neighbor : getNeighbors(current)) {
                    if (!game.containsTile(neighbor)) {
                        continue;
                    }

                    const auto &tile = game.getTile(neighbor);
                    if (tile.getTerrain() == Terrain::Ocean) {
                        continue;
                    }

                    if (visited.contains(Pos(neighbor.x, neighbor.y))) {
                        continue;
                    }

                    visited.emplace(neighbor.x, neighbor.y);
                    queue.push_back(neighbor);
                }
            }

            return std::optional<glm::uvec2>();
        }

        void handleGoal(Game &game, const Goal &goal) {
            if (goal.requestsSettlerExpansion()) {
                auto pos = getBestCityLocation(game);
                if (pos.has_value()) {
                    commands.push_back(std::make_unique<SettleCityCommand>(*pos));
                }
            }
        }

        std::unique_ptr<BuildTask> getBuildTask(Game &game, const Goal &goal) {
            const auto &registry = game.getRegistry();
            if (goal.requestsSettlerExpansion()) {
                auto &settler = registry.getUnits().at(0);
                return std::make_unique<UnitBuildTask>(settler);
            } else {
                auto &warrior = registry.getUnits().at(1);
                return std::make_unique<UnitBuildTask>(warrior);
            }
        }

        void setCityBuildTasks(Game &game, const Goal &goal) {
            const auto &player = game.getPlayer(playerID);
            for (const auto cityID : player.getCities()) {
                auto &city = game.getCity(cityID);
                if (!city.hasBuildTask()) {
                    auto task = getBuildTask(game, goal);
                    city.setBuildTask(std::move(task));
                }
            }
        }

    public:
        ShortTermBrain(const PlayerId &playerId) : playerID(playerId) {}

        void update(Game &game, const Goal &goal) {
            handleGoal(game, goal);
            setCityBuildTasks(game, goal);
        }

        bool hasCommand() {
            return !commands.empty();
        }

        std::unique_ptr<TacticalCommand> popCommand() {
            auto cmd = std::move(commands.at(0));
            commands.pop_front();
            return cmd;
        }
    };

    class TacticalBrain {
        PlayerId playerID;

        absl::flat_hash_map<UnitId, glm::uvec2> settlersDeploying;

        void moveScouts(Game &game) {
            Rng rng;
            for (auto &unit : game.getUnits()) {
                if (unit.getOwner() == playerID && unit.getKind().id != "settler" && !unit.hasPath()) {
                    std::optional<Path> path;
                    while (!path.has_value()) {
                        glm::uvec2 target(rng.u32(unit.getPos().x - 10, unit.getPos().x + 10),
                                          rng.u32(unit.getPos().y - 10, unit.getPos().y + 10));
                        path = computeShortestPath(game, unit.getPos(), target, std::optional<VisibilityMap>());
                    }
                    unit.setPath(std::move(*path));
                }
            }
        }

    public:
        TacticalBrain(const PlayerId &playerId) : playerID(playerId) {}

        void handleCommand(Game &game, const TacticalCommand &command) {
            auto asSettle = dynamic_cast<const SettleCityCommand*>(&command);
            if (asSettle) {
                for (auto &unit : game.getUnits()) {
                    if (unit.getOwner() != playerID) {
                        continue;
                    }
                    if (settlersDeploying.contains(unit.getID())) {
                        continue;
                    }
                    if (std::find(unit.getKind().capabilities.begin(), unit.getKind().capabilities.end(), "found_city")
                        != unit.getKind().capabilities.end()) {
                        auto path = computeShortestPath(game, unit.getPos(), asSettle->settleLocation, std::optional<VisibilityMap>());
                        if (path.has_value()) {
                            unit.setPath(std::move(*path));
                            settlersDeploying[unit.getID()] = asSettle->settleLocation;
                            break;
                        }
                    }
                }
            }
        }

        void update(Game &game) {
            moveScouts(game);
            for (auto &unit : game.getUnits()) {
                if (unit.getOwner() == playerID) {
                    unit.moveAlongCurrentPath(game);
                }
            }

            std::vector<UnitId> toRemove;
            for (auto &pair : settlersDeploying) {
                auto &unit = game.getUnit(pair.first);
                if (unit.getPos() == pair.second && !game.getCityAtLocation(unit.getPos())) {
                    game.getPlayer(playerID).createCity(unit.getPos(), game);
                    game.killUnit(unit.getID());
                    toRemove.push_back(unit.getID());
                    std::cout << "[ai] founded city" << std::endl;
                }
            }

            for (auto id : toRemove) {
                settlersDeploying.erase(id);
            }
        }
    };

    AI::AI(PlayerId playerID) : playerID(playerID),
        longTermBrain(std::make_unique<LongTermBrain>(playerID)),
        shortTermBrain(std::make_unique<ShortTermBrain>(playerID)),
        tacticalBrain(std::make_unique<TacticalBrain>(playerID)) {

    }

    AI::~AI() = default;

    AI::AI(AI &&other) = default;

    AI &AI::operator=(AI &&other) noexcept = default;

    void AI::doTurn(Game &game) {
        longTermBrain->update(game);
        const auto &goal = longTermBrain->getGoal();
        shortTermBrain->update(game, goal);
        while (shortTermBrain->hasCommand()) {
            tacticalBrain->handleCommand(game, *shortTermBrain->popCommand());
        }
        tacticalBrain->update(game);
    }
}
