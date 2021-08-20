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
#include "stack.h"
#include <riposte.pb.h>
#include "saveload.h"
#include "protocol.h"

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

    City::City(UpdateCity &packet, const Registry &registry, const IdConverter &playerIDs) {
        pos = glm::uvec2(packet.pos().x(), packet.pos().y());
        name = packet.name();
        owner = playerIDs.get(packet.ownerid());

        if (packet.has_buildtask()) {
            const auto &buildTaskProto = packet.buildtask().kind();
            if (buildTaskProto.has_unit()) {
                buildTask = std::make_unique<UnitBuildTask>(registry.getUnit(buildTaskProto.unit().unitkindid()));
            } else if (buildTaskProto.has_building()) {
                buildTask = std::make_unique<BuildingBuildTask>(registry.getBuilding(buildTaskProto.building().buildingname()));
            }
            buildTask->spendHammers(packet.buildtask().progress());
        }

        culture = getCultureFromProto(packet.culturevalues(), playerIDs);

        for (const auto &buildingName : packet.buildingnames()) {
            buildings.push_back(registry.getBuilding(buildingName));
        }

        population = packet.population();
        storedFood = packet.storedfood();
        capital = packet.iscapital();

        for (const auto &p : packet.workedtiles()) {
            workedTiles.emplace_back(p.x(), p.y());
        }
        for (const auto &p : packet.manualworkedtiles()) {
            manualWorkedTiles.emplace_back(p.x(), p.y());
        }

        cultureDefenseBonus = packet.culturedefensebonus();
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

        for (int i = 0; i < std::min((int) getNumWorkingCitizens() + 1, (int) entries.size()); i++) {
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

    void City::addManualWorkedTile(const Game &game, glm::uvec2 pos) {
        if (!canWorkTile(pos, game)) return;

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
        if (!buildTask) {
            game.getServer().sendBuildTaskFinished(id, nullptr);
        }
        auto yield = computeYield(game);
        if (hasBuildTask()) {
            buildTask->spendHammers(yield.hammers);

            if (buildTask->isFinished()) {
                buildTask->onCompleted(game, *this);
                game.getServer().sendBuildTaskFinished(id, &*buildTask);
                buildTask = {};
            } else if (!buildTask->canBuild(game, *this)) {
                // We can no longer build - e.g. because we don't have the necessary resources
                // anymore.
                game.getServer().sendBuildTaskFailed(id, *buildTask);
                buildTask = {};
            }
        }

        regrowCultureDefense();

        doGrowth(game);
        updateHappiness(game);
        updateHealth(game);

        updateWorkedTiles(game);
        workTiles(game);

        const auto oldCultureLevel = getCultureLevel();
        culture.addCultureForPlayer(owner, getCulturePerTurn());
        if (getCultureLevel().value > oldCultureLevel.value) {
            game.getServer().broadcastBordersExpanded(id);
        }

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

        bool uniqueUnit = (unitKind->onlyForCivs.empty() || unitKind->onlyForCivs.contains(
                game.getPlayer(builder.getOwner()).getCiv().id
                ));

        return hasTech && isCoastal && uniqueUnit;
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
        return population * 2 + std::max((int) (getSickness() - getHealth()), 0);
    }

    const Culture &City::getCulture() const {
        return culture;
    }

    int City::getCulturePerTurn() const {
        int culture = 0;
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

    void City::onCreated(Game &game, bool isLoading) {
        game.getCultureMap().onCityCreated(game, getID());
        game.getTradeRoutes().onCityCreated(game, *this);
        updateHappiness(game);
        updateHealth(game);
        updateWorkedTiles(game);

        // Cause the client to prompt for a new build task
        if (!isLoading) {
            game.getServer().markCityDirty(id);
            game.getServer().sendBuildTaskFinished(id, nullptr);
        }

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

    const absl::flat_hash_set<std::shared_ptr<Resource>, ResourceHash> &City::getResources() const {
        return resources;
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

        game.getServer().broadcastCityCaptured(id, newOwnerID);
        game.getServer().markCityDirty(id);
        game.getServer().markTileDirty(pos);

        game.getServer().sendBuildTaskFinished(id, nullptr);
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
                && game.getPlayer(builder.getOwner()).getTechs().isBuildingUnlocked(*building)
                && (building->onlyForCivs.empty()
                    || building->onlyForCivs.contains(game.getPlayer(builder.getOwner()).getCiv().id));
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

    const std::vector<HappinessEntry> &City::getHappinessSources() const {
        return happiness;
    }

    const std::vector<UnhappinessEntry> &City::getUnhappinessSources() const {
        return unhappiness;
    }

    uint32_t City::getHappiness() const {
        uint32_t sum = 0;
        for (const auto &entry : happiness) sum += entry.count();
        return sum;
    }

    uint32_t City::getUnhappiness() const {
        uint32_t sum = 0;
        for (const auto &entry : unhappiness) sum += entry.count();
        return sum;
    }

    uint32_t City::getNumWorkingCitizens() const {
        auto unhappyCitizens = std::max((int) 0, (int) (getUnhappiness() - getHappiness()));
        return getPopulation() - unhappyCitizens;
    }

    void City::updateHappiness(Game &game) {
        happiness.clear();
        unhappiness.clear();

        HappinessEntry baseHappiness;
        baseHappiness.set_source(HappinessSource::DifficultyBonus);
        baseHappiness.set_count(5);
        happiness.emplace_back(std::move(baseHappiness));

        uint32_t resourceHappy = 0;
        for (const auto &resource : resources) {
            resourceHappy += resource->happyBonus;
        }
        if (resourceHappy != 0) {
            HappinessEntry bonus;
            bonus.set_source(HappinessSource::Resources);
            bonus.set_count(resourceHappy);
            happiness.emplace_back(std::move(bonus));
        }

        HappinessEntry buildingHappiness;
        buildingHappiness.set_source(HappinessSource::Buildings);
        buildingHappiness.set_count(buildingEffects.happiness);
        if (buildingHappiness.count() > 0) {
            happiness.emplace_back(std::move(buildingHappiness));
        }

        UnhappinessEntry populationUnhappiness;
        populationUnhappiness.set_source(UnhappinessSource::Population);
        populationUnhappiness.set_count(getPopulation());
        unhappiness.emplace_back(std::move(populationUnhappiness));

        auto ourUnits = game.getStackByKey(owner, pos);
        uint32_t undefendedCount = 0;
        if (!ourUnits.has_value() || game.getStack(*ourUnits).getUnits().empty()) {
            ++undefendedCount;
        }

        auto allStacks = game.getStacksAtPos(pos);
        if (!ourUnits.has_value() && !allStacks.empty()) {
            ++undefendedCount;
        }

        if (undefendedCount != 0) {
            UnhappinessEntry undefended;
            undefended.set_source(UnhappinessSource::Undefended);
            undefended.set_count(undefendedCount);
            unhappiness.emplace_back(std::move(undefended));
        }
    }

    int City::getMaxCultureDefenseBonus() const {
        auto level = getCultureLevel();
        switch (level.value) {
            case 0:
            case 1:
                return 0;
            case 2:
                return 20;
            case 3:
                return 40;
            case 4:
                return 60;
            case 5:
                return 80;
            case 6:
                return 100;
        }
    }

    void City::regrowCultureDefense() {
        const auto growthRate = 5;
        if (getHappiness() > getUnhappiness()) {
            cultureDefenseBonus += growthRate;
            cultureDefenseBonus = std::clamp(cultureDefenseBonus, 0, getMaxCultureDefenseBonus());
        }
    }

    int City::getCultureDefenseBonus() const {
        return cultureDefenseBonus;
    }

    void City::bombardCultureDefenses(Game &game, int maxPercent) {
        cultureDefenseBonus -= maxPercent;
        cultureDefenseBonus = std::max(cultureDefenseBonus, 0);
        game.getServer().markCityDirty(id);
    }

    uint32_t City::getHealth() const {
        uint32_t sum = 0;
        for (const auto &entry : health) {
            sum += entry.count();
        }
        return sum;
    }

    uint32_t City::getSickness() const {
        uint32_t sum = 0;
        for (const auto &entry : sickness) {
            sum += entry.count();
        }
        return sum;
    }

    const std::vector<HealthEntry> &City::getHealthSources() const {
        return health;
    }

    const std::vector<SicknessEntry> &City::getSicknessSources() const {
        return sickness;
    }

    void City::updateHealth(Game &game) {
        health.clear();
        sickness.clear();

        SicknessEntry population;
        population.set_source(SicknessSource::PopulationSickness);
        population.set_count(getPopulation());
        sickness.emplace_back(std::move(population));

        HealthEntry difficultyBonus;
        difficultyBonus.set_source(HealthSource::BaseHealth);
        difficultyBonus.set_count(5);
        health.emplace_back(std::move(difficultyBonus));

        uint32_t resourceHealth = 0;
        for (const auto &resource : resources) {
            resourceHealth += resource->healthBonus;
        }
        if (resourceHealth != 0) {
            HealthEntry bonus;
            bonus.set_source(HealthSource::ResourceHealth);
            bonus.set_count(resourceHealth);
            health.emplace_back(std::move(bonus));
        }

        // Forest health
        double forestHealth = 0;
        for (const auto tilePos : getBigFatCross(pos)) {
            if (!game.containsTile(tilePos)) continue;
            if (game.getTile(tilePos).isForested() && game.getCultureMap().getTileOwner(tilePos) == owner) {
                forestHealth += 0.5;
            }
        }
        if (forestHealth >= 1) {
            HealthEntry bonus;
            bonus.set_source(HealthSource::ForestHealth);
            bonus.set_count(static_cast<uint32_t>(std::floor(forestHealth)));
            health.emplace_back(std::move(bonus));
        }
    }
}

