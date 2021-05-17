//
// Created by Caelum van Ispelen on 5/12/21.
//

#include "city.h"
#include "unit.h"
#include "game.h"
#include <string>
#include <utility>

namespace rip {
    Yield::Yield(int hammers, int commerce, int food) : hammers(hammers), commerce(commerce), food(food) {}

    Yield Yield::operator+(const Yield &other) const {
        return Yield(hammers + other.hammers, commerce + other.commerce, food + other.food);
    }

    void Yield::operator+=(const Yield &other) {
        hammers += other.hammers;
        commerce += other.commerce;
        food += other.food;
    }

    BuildTask::BuildTask(int cost) : cost(cost) {}

    int BuildTask::getCost() const {
        return cost;
    }

    int BuildTask::getProgress() const {
        return progress;
    }

    bool BuildTask::isFinished() const {
        return (progress >= cost);
    }

    int BuildTask::getOverflow() const {
        return (progress - cost);
    }

    void BuildTask::spendHammers(int hammers) {
        progress += hammers;
    }

    City::City(glm::uvec2 pos, std::string name, PlayerId owner) : pos(pos), name(std::move(name)), owner(owner) {

    }

    void City::setID(CityId id) {
        this->id = id;
    }

    glm::uvec2 City::getPos() const {
        return pos;
    }

    const std::string &City::getName() const {
        return name;
    }

    PlayerId City::getOwner() const {
        return owner;
    }

    CityId City::getID() const {
        return id;
    }

    void City::setName(std::string name) {
        this->name = std::move(name);
    }

    class BfcEntry {
    public:
        Yield yield;
        glm::uvec2 pos;

        BfcEntry(const Yield &yield, const glm::uvec2 &pos) : yield(yield), pos(pos) {}
    };

    void City::updateWorkedTiles(Game &game) {
        // Clear worked tiles and recompute them.
        for (const auto tile : workedTiles) {
            game.setTileWorked(tile, false);
        }
        workedTiles.clear();

        // Priorities:
        // 1. Food
        // 2. Production
        // 3. Commerce
        // Iterate over the BFC and optimize these.
        std::vector<BfcEntry> entries;

        for (const auto bfcPos : getBigFatCross(getPos())) {
            if (!game.containsTile(bfcPos)) continue;
            if (game.isTileWorked(bfcPos)) continue;
            const auto &tile = game.getTile(bfcPos);
            const auto yield = tile.getYield(game, bfcPos);
            entries.emplace_back(yield, bfcPos);
        }

        std::stable_sort(entries.begin(), entries.end(), [] (const BfcEntry &a, const BfcEntry &b) {
            if (a.yield.food < b.yield.food) {
                return false;
            } else if (b.yield.food < a.yield.food) {
                return true;
            } else if (a.yield.hammers < b.yield.hammers) {
                return false;
            } else if (b.yield.hammers < a.yield.hammers) {
                return true;
            } else if (a.yield.commerce < b.yield.commerce) {
                return false;
            } else {
                return true;
            }
        });

        // The city's own tile is always worked.
        entries.emplace(entries.begin(), game.getTile(pos).getYield(game, pos), pos);

        for (int i = 0; i < std::min(population + 1, (int) entries.size()); i++) {
            workedTiles.push_back(entries[i].pos);
            game.setTileWorked(entries[i].pos, true);
        }
    }

    Yield City::computeYield(const Game &game) const {
        Yield yield(0, 0, 0);
        for (const auto workedPos : workedTiles) {
            yield += game.getTile(workedPos).getYield(game, workedPos);
        }
        return yield;
    }

    void City::onTurnEnd(Game &game) {
        auto yield = computeYield(game);
        if (hasBuildTask()) {
            buildTask->spendHammers(yield.hammers);

            if (buildTask->isFinished()) {
                buildTask->onCompleted(game, *this);
                previousBuildTask = buildTask->getName();
                buildTask = std::unique_ptr<BuildTask>();
            }
        }
        updateWorkedTiles(game);
        doGrowth(game);
    }

    bool City::hasBuildTask() const {
        if (buildTask) {
            return true;
        }
        return false;
    }

    void City::setBuildTask(std::unique_ptr<BuildTask> task) {
        buildTask = std::move(task);
    }

    int City::estimateTurnsForCompletion(const BuildTask &task, const Game &game) const {
        auto production = computeYield(game).hammers;
        return (task.getCost() + production - 1) / production;
    }

    const std::string &City::getPreviousBuildTask() const {
        return previousBuildTask;
    }


    UnitBuildTask::UnitBuildTask(std::shared_ptr<UnitKind> unitKind) : BuildTask(unitKind->cost), unitKind(std::move(unitKind)) {}

    void UnitBuildTask::onCompleted(Game &game, City &builder) {
        Unit unit(unitKind, builder.getPos(), builder.getOwner());
        game.addUnit(std::move(unit));
    }

    const std::string &UnitBuildTask::getName() const {
        return unitKind->name;
    }

    std::vector<std::unique_ptr<BuildTask>> City::getPossibleBuildTasks(const Game &game) const {
        std::vector<std::unique_ptr<BuildTask>> tasks;

        for (const auto &unitKind : game.getRegistry().getUnits()) {
            tasks.push_back(std::make_unique<UnitBuildTask>(unitKind));
        }

        return tasks;
    }

    const BuildTask *City::getBuildTask() const {
        if (buildTask) {
            return &*buildTask;
        } else {
            return nullptr;
        }
    }

    int City::getPopulation() const {
        return population;
    }

    void City::doGrowth(Game &game) {
        auto yield = computeYield(game);
        auto consumedFood = population * 2;
        auto excessFood = yield.food - consumedFood;

        auto neededFoodForGrowth = 30 + 3 * population;

        storedFood += excessFood;
        if (storedFood < 0) {
            --population;
            storedFood = 0;
        } else if (storedFood >= neededFoodForGrowth) {
            ++population;
            storedFood -= neededFoodForGrowth;
        }
    }
}

