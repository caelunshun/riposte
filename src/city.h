//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_CITY_H
#define RIPOSTE_CITY_H

#include <memory>
#include <glm/vec2.hpp>
#include <rea.h>
#include <absl/container/flat_hash_set.h>
#include "player.h"
#include "registry.h"
#include "ids.h"
#include "culture.h"
#include "yield.h"

class HappinessEntry;
class UnhappinessEntry;

namespace rip {
    class City;

    // Something a city is producing right now: a unit, a building,
    // etc. This is an abstract class.
    class BuildTask {
        // The number of accumulated hammers needed to finish.
        int cost;
        // The current number of hammers spent.
        int progress = 0;
    public:
        BuildTask(int cost);
        virtual ~BuildTask() = default;

        int getCost() const;
        int getProgress() const;
        bool isFinished() const;

        // If the task is finished, returns the number
        // of extra spent hammers.
        int getOverflow() const;

        void spendHammers(int hammers);

        virtual bool canBuild(const Game &game, const City &builder) = 0;

        // Should be invoked when the task completes to spawn any
        // necessary changes (create a unit, add a building, etc.)
        virtual void onCompleted(Game &game, City &builder) = 0;

        virtual const std::string &getName() const = 0;

        virtual std::vector<std::string> describe() const {
            return {"Cost: " + std::to_string(getCost())};
        }
    };

    struct CultureLevel {
        int value;

        explicit CultureLevel(int value) : value(value) {}

        std::string getName() const {
            switch (value) {
                case 0:
                    return "None";
                case 1:
                    return "Poor";
                case 2:
                    return "Fledgling";
                case 3:
                    return "Developing";
                case 4:
                    return "Refined";
                case 5:
                    return "Influential";
                case 6:
                    return "Legendary";
                default:
                    throw std::string("invalid culture level " + std::to_string(value));
            }
        }
    };

    // A build task to build a unit.
    class UnitBuildTask : public BuildTask {
        std::shared_ptr<UnitKind> unitKind;

    public:
        UnitBuildTask(std::shared_ptr<UnitKind> unitKind);

        ~UnitBuildTask() override = default;

        void onCompleted(Game &game, City &builder) override;

        bool canBuild(const Game &game, const City &builder) override;

        const std::string &getName() const override;

        const std::shared_ptr<UnitKind> &getUnitKind() const {
            return unitKind;
        }

        std::vector<std::string> describe() const override;
    };

    // A build task to build a building.
    class BuildingBuildTask : public BuildTask {
        std::shared_ptr<Building> building;

    public:
        BuildingBuildTask(std::shared_ptr<Building> building);

        ~BuildingBuildTask() override = default;

        bool canBuild(const Game &game, const City &builder) override;

        void onCompleted(Game &game, City &builder) override;

        const std::string &getName() const override;

        const std::shared_ptr<Building> &getBuilding() const {
            return building;
        }

        std::vector<std::string> describe() const override;
    };

    class City {
        glm::uvec2 pos;
        std::string name;
        PlayerId owner;
        CityId id;

        // What the city is building right now. Can be null.
        std::unique_ptr<BuildTask> buildTask;
        std::string previousBuildTask;

        std::vector<glm::uvec2> workedTiles;
        std::vector<glm::uvec2> manualWorkedTiles;

        int population = 1;
        int storedFood = 0;

        // Culture stored in the city for each
        // player. Note that this is not the same as plot culture,
        // which is stored in the CultureMap object.
        Culture culture;

        // Resources accessible to this city.
        absl::flat_hash_set<std::shared_ptr<Resource>, ResourceHash> resources;

        // Buildings present in the city.
        std::vector<std::shared_ptr<Building>> buildings;
        // Sum of the building effects present in the city.
        BuildingEffect buildingEffects;

        // Sources of happiness in the city.
        std::vector<HappinessEntry> happiness;
        // Sources of unhapipiness in the city.
        std::vector<UnhappinessEntry> unhappiness;

        bool coastal = false;

        bool capital = false;

        void doGrowth(Game &game);

        void workTiles(Game &game);

    public:
        City(glm::uvec2 pos, std::string name, PlayerId owner);

        void setID(CityId id);
        void setCapital(Game &game, bool isCapital);

        glm::uvec2 getPos() const;
        const std::string &getName() const;
        PlayerId getOwner() const;
        CityId getID() const;
        bool isCapital() const;

        const Culture &getCulture() const;
        int getCulturePerTurn() const;
        CultureLevel getCultureLevel() const;

        void setName(std::string name);

        // Updates automatically chosen worked tiles.
        // Also, removes manually worked tiles that can
        // no longer be worked.
        void updateWorkedTiles(Game &game);
        // Returns whether this city can work the given tile.
        bool canWorkTile(glm::uvec2 pos, const Game &game) const;
        // Adds a manual worked tile that overrides an automatic one.
        void addManualWorkedTile(const Game &game, glm::uvec2 pos);
        // Removes a manual worked tile.
        void removeManualWorkedTile(glm::uvec2 pos);
        // Gets a list of worked tiles.
        const std::vector<glm::uvec2> &getWorkedTiles() const;
        const std::vector<glm::uvec2> &getManualWorkedTiles() const;

        Yield computeYield(const Game &game) const;

        int getGoldProduced(Game &game) const;

        void onCreated(Game &game);
        void onTurnEnd(Game &game);

        bool hasBuildTask() const;
        void setBuildTask(std::unique_ptr<BuildTask> task);
        const BuildTask *getBuildTask() const;
        int estimateTurnsForCompletion(const BuildTask &task, const Game &game) const;

        const std::string &getPreviousBuildTask() const;

        std::vector<std::unique_ptr<BuildTask>> getPossibleBuildTasks(const Game &game) const;

        int getPopulation() const;

        bool hasResource(const std::shared_ptr<Resource> &resource) const;
        void addResource(std::shared_ptr<Resource> resource);
        void clearResources();

        int getMaintenanceCost(const Game &game) const;

        const std::vector<std::shared_ptr<Building>> &getBuildings() const;
        bool hasBuilding(const std::string &buildingName) const;
        void addBuilding(std::shared_ptr<Building> building);
        const BuildingEffect &getBuildingEffects() const;

        bool isCoastal() const;

        int getStoredFood() const;
        int getFoodNeededForGrowth() const;
        int getConsumedFood() const;

        const std::vector<HappinessEntry> &getHappinessSources() const;
        const std::vector<UnhappinessEntry> &getUnhappinessSources() const;

        uint32_t getHappiness() const;
        uint32_t getUnhappiness() const;

        uint32_t getNumWorkingCitizens() const;

        void updateHappiness(Game &game);

        void transferControlTo(Game &game, PlayerId newOwner);
    };
}

#endif //RIPOSTE_CITY_H
