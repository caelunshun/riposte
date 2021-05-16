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

    City::City(glm::uvec2 pos, std::string name, PlayerId owner) : pos(pos), name(std::move(name)), owner(owner) {}

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

    Yield City::computeYield(const Game &game) const {
        return Yield(10, 2, 2); // TODO
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

    // A BuildTask to build a unit.
    class UnitBuildTask : public BuildTask {
        std::shared_ptr<UnitKind> unitKind;

    public:
        UnitBuildTask(std::shared_ptr<UnitKind> unitKind) : BuildTask(unitKind->cost), unitKind(std::move(unitKind)) {}

        ~UnitBuildTask() override = default;

        void onCompleted(Game &game, City &builder) override {
            Unit unit(unitKind, builder.getPos(), builder.getOwner());
            game.addUnit(std::move(unit));
        }

        const std::string &getName() const override {
            return unitKind->name;
        }
    };

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
}

