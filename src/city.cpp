//
// Created by Caelum van Ispelen on 5/12/21.
//

#include "city.h"
#include "unit.h"
#include "game.h"
#include "tile.h"
#include "trade.h"
#include "event.h"
#include <string>
#include <utility>
#include <iostream>
#include "server.h"

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

    bool City::isCapital() const {
        return capital;
    }

    void City::setName(std::string name) {
        this->name = std::move(name);
    }

    class BfcEntry {
    public:
        Yield yield;
        glm::uvec2 pos;
        bool forced;

        BfcEntry(const Yield &yield, const glm::uvec2 &pos, bool forced) : yield(yield), pos(pos), forced(forced) {}
    };

    void City::updateWorkedTiles(Game &game) {
        // Clear worked tiles and recompute them.
        for (const auto tile : workedTiles) {
            game.setTileWorked(tile, false, id);
        }
        workedTiles.clear();

        // Priorities:
        // 1. Food
        // 2. Production
        // 3. Commerce
        // Iterate over the BFC and optimize these.
        std::vector<BfcEntry> entries;

        // Account for manual tiles
        for (int i = 0; i < manualWorkedTiles.size(); i++) {
            auto tilePos = manualWorkedTiles[i];
            if (!canWorkTile(tilePos, game)) {
                removeManualWorkedTile(tilePos);
                --i;
                continue;
            }

            entries.emplace_back(game.getTile(tilePos).getYield(game, tilePos, owner), tilePos, true);
        }

        for (const auto bfcPos : getBigFatCross(getPos())) {
            if (!canWorkTile(bfcPos, game)) continue;
            const auto &tile = game.getTile(bfcPos);
            const auto yield = tile.getYield(game, bfcPos, owner);
            entries.emplace_back(yield, bfcPos, false);
        }

        std::stable_sort(entries.begin(), entries.end(), [&] (const BfcEntry &a, const BfcEntry &b) {
            if (b.forced) return false;
            else if (a.forced) return true;
            else if (a.yield.food < b.yield.food) {
                return false;
            } else if (b.yield.food < a.yield.food) {
                return true;
            } else if (a.yield.hammers + a.yield.commerce < b.yield.hammers + b.yield.commerce) {
                return false;
            } else if (a.yield.hammers + a.yield.commerce == b.yield.hammers + b.yield.commerce) {
                if (game.getTile(a.pos).hasNonRoadImprovements()) {
                    return true;
                } else if (game.getTile(b.pos).hasNonRoadImprovements()) {
                    return false;
                }
            }

            return true;
        });

        // The city's own tile is always worked.
        entries.emplace(entries.begin(), game.getTile(pos).getYield(game, pos, owner), pos, true);

        for (int i = 0; i < std::min(population + 1, (int) entries.size()); i++) {
            workedTiles.push_back(entries[i].pos);
            game.setTileWorked(entries[i].pos, true, id);
        }

        // Remove manual worked tiles that are no longer worked.
        if (!manualWorkedTiles.empty()) {
            for (int i = manualWorkedTiles.size() - 1; i >= 0; i--) {
                auto p = manualWorkedTiles[i];
                if (std::find(workedTiles.begin(), workedTiles.end(), p) == workedTiles.end()) {
                    manualWorkedTiles.erase(manualWorkedTiles.begin() + i);
                }
            }
        }
    }

    bool City::canWorkTile(glm::uvec2 tilePos, const Game &game) const {
        if (!game.containsTile(tilePos)) return false;
        if (dist(tilePos, pos) >= 2.5) return false;
        auto worker = game.isTileWorked(tilePos);
        if (!(!worker.has_value() || worker == id)) return false;
        if (game.getCultureMap().getTileOwner(tilePos) != std::make_optional<PlayerId>(getOwner())) return false;
        return true;
    }

    void City::addManualWorkedTile(glm::uvec2 pos) {
        removeManualWorkedTile(pos);
        manualWorkedTiles.push_back(pos);

        if (manualWorkedTiles.size() > population) {
            manualWorkedTiles.erase(manualWorkedTiles.begin());
        }
    }

    void City::removeManualWorkedTile(glm::uvec2 pos) {
        auto it = std::find(manualWorkedTiles.begin(), manualWorkedTiles.end(), pos);
        if (it != manualWorkedTiles.end()) {
            manualWorkedTiles.erase(it);
        }
    }

    const std::vector<glm::uvec2> &City::getWorkedTiles() const {
        return workedTiles;
    }

    const std::vector<glm::uvec2> &City::getManualWorkedTiles() const {
        return manualWorkedTiles;
    }

    Yield City::computeYield(const Game &game) const {
        Yield yield(0, 0, 0);
        for (const auto workedPos : workedTiles) {
            const auto &tile = game.getTile(workedPos);

            yield += tile.getYield(game, workedPos, owner);

            if (tile.getTerrain() == Terrain::Ocean) {
                yield += Yield(0, 0, buildingEffects.oceanFoodBonus);
            }
        }

        // Apply building effects
        yield.hammers += buildingEffects.bonusHammers;
        yield.commerce += buildingEffects.bonusCommerce;
        yield.food += buildingEffects.bonusFood;

        yield.hammers += percentOf(yield.hammers, buildingEffects.bonusHammerPercent);
        yield.commerce += percentOf(yield.commerce, buildingEffects.bonusCommercePercent);
        yield.food += percentOf(yield.food, buildingEffects.bonusFoodPercent);

        return yield;
    }

    void City::workTiles(Game &game) {
        for (const auto tilePos : workedTiles) {
            auto &tile = game.getTile(tilePos);
            for (auto &improvement : tile.getImprovements()) {
                improvement->onWorked(game, *this);
            }
        }
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
        workTiles(game);
        doGrowth(game);

        culture.addCultureForPlayer(owner, getCulturePerTurn());

        game.getServer().markCityDirty(id);
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
        return (task.getCost() - task.getProgress() + production - 1) / production;
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
        bool isCoastal = !unitKind->ship || builder.isCoastal();

        return hasTech && isCoastal;
    }

    std::vector<std::string> UnitBuildTask::describe() const {
        std::vector<std::string> result = {
             "Cost: " + std::to_string(getCost()),
             "Type: " + unitKind->category + " unit",
             "Strength: " + std::to_string(static_cast<int>(unitKind->strength)),
             "Movement: " + std::to_string(static_cast<int>(unitKind->movement)),
        };

        if (unitKind->carryUnitCapacity != 0) {
            result.push_back("Can carry " + std::to_string(unitKind->carryUnitCapacity) + " units");
        }

        for (const auto &bonus : unitKind->combatBonuses) {
            int amount = 0;
            std::string text;
            if (bonus.againstUnitBonus != 0) {
                amount = bonus.againstUnitBonus;
                text = " against " + bonus.unit;
            } else if (bonus.againstUnitCategoryBonus != 0) {
                amount = bonus.againstUnitCategoryBonus;
                text = " against " + bonus.unitCategory + " units";
            } else if (bonus.whenInCityBonus != 0) {
                amount = bonus.whenInCityBonus;
                text = " when in city";
            }

            if (bonus.onlyOnAttack) {
                text = " attack" + text;
            } else if (bonus.onlyOnDefense) {
                text = " defense" + text;
            }

            result.push_back("+" + std::to_string(amount) + "%" + text);
        }

        return result;
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

        for (const auto &building : game.getRegistry().getBuildings()) {
            auto task = std::make_unique<BuildingBuildTask>(building);
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
        auto consumedFood = getConsumedFood();
        auto excessFood = yield.food - consumedFood;

        auto neededFoodForGrowth = getFoodNeededForGrowth();

        storedFood += excessFood;
        if (storedFood < 0) {
            --population;
            if (population < 1) population = 1;
            updateWorkedTiles(game);
            storedFood = (30 + 3 * (population - 1)) - 1;
        } else if (storedFood >= neededFoodForGrowth) {
            ++population;
            updateWorkedTiles(game);
            storedFood -= neededFoodForGrowth;

            if (buildingEffects.hasGranaryFoodStore) {
                storedFood += neededFoodForGrowth / 2;
            }
        }
    }

    int City::getStoredFood() const {
        return storedFood;
    }

    int City::getFoodNeededForGrowth() const {
        return 30 + 3 * population;
    }

    int City::getConsumedFood() const {
        return population * 2;
    }

    const Culture &City::getCulture() const {
        return culture;
    }

    int City::getCulturePerTurn() const {
        int culture = 1;
        if (isCapital()) {
            culture += 1;
        }

        culture += buildingEffects.bonusCulture;
        culture += percentOf(culture, buildingEffects.bonusCulturePercent);

        return culture;
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
        updateWorkedTiles(game);

        // Check coastal status.
        for (const auto neighborPos : getNeighbors(pos)) {
            if (!game.containsTile(neighborPos)) continue;
            const auto &tile = game.getTile(neighborPos);
            if (tile.getTerrain() == Terrain::Ocean) {
                coastal = true;
                break;
            }
        }
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

    void City::setCapital(Game &game, bool isCapital) {
        this->capital = isCapital;
        if (isCapital) {
            game.getPlayer(owner).setCapital(id);
        }
    }

    int City::getMaintenanceCost(const Game &game) const {
        const auto &capitalCity = game.getCity(game.getPlayer(owner).getCapital());
        float baseDistanceCost = dist(pos, capitalCity.getPos()) * 0.25;
        int distanceFromPalaceCost = static_cast<int>((7 + population) * (baseDistanceCost / 8));

        int numberOfCitiesCost = static_cast<int>(0.6 + 0.033 * population * game.getPlayer(owner).getCities().size() / 2);

        int total = distanceFromPalaceCost + numberOfCitiesCost;

        total -= percentOf(total, buildingEffects.minusMaintenancePercent);

        return total;
    }

    void City::transferControlTo(Game &game, PlayerId newOwnerID) {
        if (newOwnerID == owner) return;

        capital = false;

        game.getCultureMap().onCityDestroyed(game, id);
        auto &oldOwner = game.getPlayer(owner);
        oldOwner.removeCity(id, game);
        auto &newOwner = game.getPlayer(newOwnerID);
        newOwner.registerCity(id);
        if (population > 1) --population;
        buildTask = {};
        previousBuildTask = "";
        owner = newOwnerID;
        game.getCultureMap().onCityCreated(game, id);

        game.addEvent(std::make_unique<CityCapturedEvent>(
                name,
                newOwner.getCiv().name
                ));

        newOwner.recomputeScore(game);
        oldOwner.recomputeScore(game);

        game.getServer().markCityDirty(id);
    }

    const std::vector<std::shared_ptr<Building>> &City::getBuildings() const {
        return buildings;
    }

    bool City::hasBuilding(const std::string &buildingName) const {
        for (const auto &building : getBuildings()) {
            if (building->name == buildingName) {
                return true;
            }
        }
        return false;
    }

    void City::addBuilding(std::shared_ptr<Building> building) {
        if (hasBuilding(building->name)) return;
        for (const auto &effect : building->effects) {
            buildingEffects += effect;
        }
        buildings.emplace_back(std::move(building));
    }

    const BuildingEffect &City::getBuildingEffects() const {
        return buildingEffects;
    }

    bool City::isCoastal() const {
        return coastal;
    }

    bool BuildingBuildTask::canBuild(const Game &game, const City &builder) {
        return
                !builder.hasBuilding(building->name)
                && (!building->onlyCoastal || builder.isCoastal())
                && game.getPlayer(builder.getOwner()).getTechs().isBuildingUnlocked(*building);
    }

    void BuildingBuildTask::onCompleted(Game &game, City &builder) {
        builder.addBuilding(building);
    }

    const std::string &BuildingBuildTask::getName() const {
        return building->name;
    }

    BuildingBuildTask::BuildingBuildTask(std::shared_ptr<Building> building)
        : BuildTask(building->cost), building(building) {

    }

    std::vector<std::string> BuildingBuildTask::describe() const {
        return BuildTask::describe();
    }
}

