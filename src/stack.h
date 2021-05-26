//
// Created by Caelum van Ispelen on 5/25/21.
//

#ifndef RIPOSTE_STACK_H
#define RIPOSTE_STACK_H

#include "ids.h"
#include <glm/vec2.hpp>
#include <vector>

namespace rip {
    class Game;

    // A stack of units all on the same tile.
    //
    // All units in a stack have the same owner.
    class Stack {
        PlayerId owner;
        std::vector<UnitId> units;
        glm::uvec2 pos;

    public:
        Stack(PlayerId owner, glm::uvec2 pos);

        void addUnit(UnitId unit);
        void removeUnit(UnitId unit);
        bool containsUnit(UnitId unit) const;
        const std::vector<UnitId> &getUnits() const;

        // Gets the unit with the highest combat strength.
        UnitId getBestUnit(const Game &game) const;

        glm::uvec2 getPos() const;
        PlayerId getOwner() const;
    };
}

#endif //RIPOSTE_STACK_H
