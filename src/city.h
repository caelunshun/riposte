//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_CITY_H
#define RIPOSTE_CITY_H

#include <memory>
#include <glm/vec2.hpp>
#include <rea.h>
#include "player.h"
#include "registry.h"
#include "ids.h"
#include "culture.h"
#include "yield.h"

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

        int population = 1;
        int storedFood = 0;

        // Culture stored in the city for each
        // player. Note that this is not the same as plot culture,
        // which is stored in the CultureMap object.
        Culture culture;

        void doGrowth(Game &game);

    public:
        City(glm::uvec2 pos, std::string name, PlayerId owner);

        void setID(CityId id);

        glm::uvec2 getPos() const;
        const std::string &getName() const;
        PlayerId getOwner() const;
        CityId getID() const;

        const Culture &getCulture() const;
        int getCulturePerTurn() const;
        CultureLevel getCultureLevel() const;

        void setName(std::string name);

        void updateWorkedTiles(Game &game);
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
    };
}

#endif //RIPOSTE_CITY_H
