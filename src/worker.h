//
// Created by Caelum van Ispelen on 5/17/21.
//

#ifndef RIPOSTE_WORKER_H
#define RIPOSTE_WORKER_H

#include <glm/vec2.hpp>
#include <optional>
#include "unit.h"

namespace rip {
    class Game;
    class Improvement;

    // A task being completed by a worker.
    class WorkerTask {
    protected:
        // Turns left until the task is complete.
        int remainingTurns;
        // Position of the worker.
        glm::uvec2 pos;
        WorkerTask(int numberOfTurns, glm::uvec2 pos) : remainingTurns(numberOfTurns), pos(pos) {}

    public:
        virtual void onTurnEnd(Game &game) {
            --remainingTurns;
        }

        glm::uvec2 getPos() const {
            return pos;
        }
        int getRemainingTurns() const;
        bool isFinished() const;

        virtual void onFinished(Game &game) = 0;

        virtual std::string getName() = 0;
        virtual std::string getPresentParticiple() = 0;

        virtual ~WorkerTask() = default;
    };

    // A task to build an improvement.
    class BuildImprovementTask : public WorkerTask {
        std::unique_ptr<Improvement> improvement;

    public:
        BuildImprovementTask(int numberOfTurns, glm::uvec2 pos, std::unique_ptr<Improvement> improvement) : WorkerTask(numberOfTurns, pos), improvement(std::move(improvement)) {}

        void onFinished(Game &game) override;

        std::string getName() override;

        std::string getPresentParticiple() override;

        const Improvement &getImprovement() const;
    };

    // Capability attached to workers.
    class WorkerCapability : public Capability {
        std::optional<std::unique_ptr<WorkerTask>> currentTask;

    public:
        explicit WorkerCapability(UnitId unitID);

        void onTurnEnd(Game &game) override;

        UnitUIStatus paintMainUI(Game &game, Hud &hud, nk_context *nk) override;

        std::vector<std::unique_ptr<WorkerTask>> getPossibleTasks(const Game &game) const;

        void setTask(std::unique_ptr<WorkerTask> task);

        const WorkerTask *getTask() const;

        void onUnitMoved(Game &game, glm::uvec2 oldPos) override;
    };
}

#endif //RIPOSTE_WORKER_H
