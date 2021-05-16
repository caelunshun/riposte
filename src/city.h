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

namespace rip {
    struct Yield {
        int hammers;
        int commerce;
        int food;

        Yield(int hammers, int commerce, int food);
    };

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

        // Should be invoked when the task completes to spawn any
        // necessary changes (create a unit, add a building, etc.)
        virtual void onCompleted(Game &game, City &builder) = 0;

        virtual const std::string &getName() const = 0;
    };

    // A build task to build a unit.
    class UnitBuildTask : public BuildTask {
        std::shared_ptr<UnitKind> unitKind;

    public:
        UnitBuildTask(std::shared_ptr<UnitKind> unitKind);

        ~UnitBuildTask() override = default;

        void onCompleted(Game &game, City &builder) override;

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

    public:
        City(glm::uvec2 pos, std::string name, PlayerId owner);

        void setID(CityId id);

        glm::uvec2 getPos() const;
        const std::string &getName() const;
        PlayerId getOwner() const;
        CityId getID() const;

        void setName(std::string name);

        Yield computeYield(const Game &game) const;

        void onTurnEnd(Game &game);

        bool hasBuildTask() const;
        void setBuildTask(std::unique_ptr<BuildTask> task);
        const BuildTask *getBuildTask() const;
        int estimateTurnsForCompletion(const BuildTask &task, const Game &game) const;

        const std::string &getPreviousBuildTask() const;

        std::vector<std::unique_ptr<BuildTask>> getPossibleBuildTasks(const Game &game) const;
    };
}

#endif //RIPOSTE_CITY_H
