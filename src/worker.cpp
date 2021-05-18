//
// Created by Caelum van Ispelen on 5/17/21.
//

#include "worker.h"
#include "game.h"
#include <nuklear.h>

namespace rip {
    int WorkerTask::getRemainingTurns() const {
        return remainingTurns;
    }

    bool WorkerTask::isFinished() const {
        return getRemainingTurns() <= 0;
    }

    WorkerCapability::WorkerCapability(UnitId unitID) : Capability(unitID) {}

    void WorkerCapability::onTurnEnd(Game &game) {
        if (currentTask.has_value()) {
            (*currentTask)->onTurnEnd(game);

            if ((*currentTask)->isFinished()) {
                (*currentTask)->onFinished(game);
                currentTask = {};
            }

            auto &unit = game.getUnit(unitID);
            unit.setMovementLeft(0); // hard at work!
        }
    }

    std::vector<std::unique_ptr<WorkerTask>> WorkerCapability::getPossibleTasks(const Game &game) const {
        std::vector<std::unique_ptr<WorkerTask>> tasks;

        const auto &unit = game.getUnit(unitID);

        Cottage cottage(unit.getPos());
        tasks.push_back(std::make_unique<BuildImprovementTask>(cottage.getNumBuildTurns(), unit.getPos(), std::make_unique<Cottage>(std::move(cottage))));
        Mine mine(unit.getPos());
        tasks.push_back(std::make_unique<BuildImprovementTask>(mine.getNumBuildTurns(), unit.getPos(), std::make_unique<Mine>(std::move(mine))));
        Farm farm(unit.getPos());
        tasks.push_back(std::make_unique<BuildImprovementTask>(farm.getNumBuildTurns(), unit.getPos(), std::make_unique<Farm>(std::move(farm))));

        // Remove incompatible improvement tasks.
        for (int i = static_cast<int>(tasks.size()) - 1; i >= 0; i--) {
            auto &task = tasks[i];
            auto downcasted = dynamic_cast<BuildImprovementTask*>(&*task);
            if (downcasted) {
                if (!downcasted->getImprovement().isCompatible(game.getTile(unit.getPos()))) {
                    tasks.erase(tasks.begin() + i);
                }
            }
        }

        return tasks;
    }

    void WorkerCapability::onUnitMoved(Game &game) {
        currentTask = {};
    }

    UnitUIStatus WorkerCapability::paintMainUI(Game &game, nk_context *nk) {
        auto &unit = game.getUnit(unitID);

        if (currentTask.has_value()) {
            auto &task = **currentTask;
            nk_layout_row_push(nk, 150);
            auto text = task.getPresentParticiple() + " (" + std::to_string(task.getRemainingTurns()) + ")";
            nk_label(nk, text.c_str(), NK_TEXT_ALIGN_LEFT);
        }

        if (!game.getCityAtLocation(unit.getPos())) {
            for (auto &possibleTask : getPossibleTasks(game)) {
                nk_layout_row_push(nk, 120);
                auto text = possibleTask->getName();
                if (nk_button_label(nk, text.c_str())) {
                    currentTask = std::move(possibleTask);
                    unit.setMovementLeft(0);
                    return UnitUIStatus::Deselect;
                }
            }
        }

        return UnitUIStatus::None;
    }

    void BuildImprovementTask::onFinished(Game &game) {
        auto &tile = game.getTile(pos);
        tile.addImprovement(std::move(improvement));
        tile.setForested(false);
    }

    std::string BuildImprovementTask::getName() {
        return "Build " + improvement->getName();
    }

    std::string BuildImprovementTask::getPresentParticiple() {
        return "Building " + improvement->getName();
    }

    const Improvement &BuildImprovementTask::getImprovement() const {
        return *improvement;
    }
}