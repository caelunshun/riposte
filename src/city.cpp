//
// Created by Caelum van Ispelen on 5/12/21.
//

#include "city.h"
#include "unit.h"
#include "game.h"
#include <string>
#include <utility>
#include <iostream>

namespace rip {
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

    City::City(glm::uvec2 pos, std::string name, PlayerId owner) : pos(pos), name(std::move(name)), owner(owner), culture() {
        culture.addCultureForPlayer(owner, 1);
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
            if (game.getCultureMap().getTileOwner(bfcPos) != std::make_optional<PlayerId>(getOwner())) continue;
            const auto &tile = game.getTile(bfcPos);
            const auto yield = tile.getYield(game, bfcPos, owner);
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
        entries.emplace(entries.begin(), game.getTile(pos).getYield(game, pos, owner), pos);

        for (int i = 0; i < std::min(population + 1, (int) entries.size()); i++) {
            workedTiles.push_back(entries[i].pos);
            game.setTileWorked(entries[i].pos, true);
        }
    }

    Yield City::computeYield(const Game &game) const {
        Yield yield(0, 0, 0);
        for (const auto workedPos : workedTiles) {
            yield += game.getTile(workedPos).getYield(game, workedPos, owner);
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

        culture.addCultureForPlayer(owner, getCulturePerTurn());
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

    bool UnitBuildTask::canBuild(const Game &game, const City &builder) {
        // Check resources.
        for (const auto &resourceName : unitKind->resources) {
            const auto &resource = game.getRegistry().getResource(resourceName);
            if (!builder.hasResource(resource)) {
                return false;
            }
        }

        bool hasTech = game.getPlayer(builder.getOwner()).getTechs().isUnitUnlocked(*unitKind);
        return hasTech;
    }

    std::vector<std::unique_ptr<BuildTask>> City::getPossibleBuildTasks(const Game &game) const {
        std::vector<std::unique_ptr<BuildTask>> tasks;

        for (const auto &unitKind : game.getRegistry().getUnits()) {
            auto task = std::make_unique<UnitBuildTask>(unitKind);
            if (!task->canBuild(game, *this)) {
                continue;
            }
            tasks.push_back(std::move(task));
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
            updateWorkedTiles(game);
            storedFood = (30 + 3 * (population - 1)) - 1;
        } else if (storedFood >= neededFoodForGrowth) {
            ++population;
            updateWorkedTiles(game);
            storedFood -= neededFoodForGrowth;
        }
    }

    const Culture &City::getCulture() const {
        return culture;
    }

    int City::getCulturePerTurn() const {
        return population; // TODO?
    }

    CultureLevel City::getCultureLevel() const {
        const auto culture = getCulture().getCultureForPlayer(owner);
        if (culture < 10) {
            return CultureLevel(1);
        } else if (culture < 100) {
            return CultureLevel(2);
        } else if (culture < 500) {
            return CultureLevel(3);
        } else if (culture < 5000) {
            return CultureLevel(4);
        } else if (culture < 50000) {
            return CultureLevel(5);
        } else {
            return CultureLevel(6);
        }
    }

    void City::onCreated(Game &game) {
        game.getCultureMap().onCityCreated(game, getID());
        game.getTradeRoutes().onCityCreated(game, *this);
    }

    int City::getGoldProduced(Game &game) const {
        return computeYield(game).commerce;
    }

    bool City::hasResource(const std::shared_ptr<Resource> &resource) const {
        return resources.contains(resource);
    }

    void City::addResource(std::shared_ptr<Resource> resource) {
        std::cout << "city got " << resource->name << std::endl;
        resources.insert(std::move(resource));
    }

    void City::clearResources() {
        resources.clear();
    }
}

