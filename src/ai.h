//
// Created by Caelum van Ispelen on 5/16/21.
//

#ifndef RIPOSTE_AI_H
#define RIPOSTE_AI_H

#include <memory>
#include "ids.h"

namespace rip {
    class Game;

    class AIimpl;

    // Maintains the state for an AI player.
    class AI {
        std::unique_ptr<AIimpl> impl;

    public:
        explicit AI(PlayerId playerID);

        ~AI();

        AI(AI &&other);
        AI(const AI &other) = delete;

        AI &operator=(AI &&other) noexcept;
        AI &operator=(const AI &other) = delete;

        // Performs a turn for this AI player.
        void doTurn(Game &game);
    };
}

#endif //RIPOSTE_AI_H
