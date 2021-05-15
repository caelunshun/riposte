//
// Created by Caelum van Ispelen on 5/13/21.
//

#ifndef RIPOSTE_UNIT_H
#define RIPOSTE_UNIT_H

#include <glm/vec2.hpp>
#include "registry.h"
#include "ids.h"

namespace rip {
    class Game;

    /**
     * A unit on the map.
     */
    class Unit {
        // What kind the unit is - determines name, strength, movement, etc
        std::shared_ptr<UnitKind> kind;
        // The position of the unit on the map
        glm::uvec2 pos;
        // The unit's ID in the Game class
        UnitId id;
        // The player that owns the unit
        PlayerId owner;
        // The unit's current health, between 0 and 1 inclusive.
        // The actual combat strength is the unit's strength times its health.
        double health;
        // How many tiles the unit has left to move on this turn.
        // Resets to kind.movement at the start of every turn.
        int movementLeft;

        void resetMovement();

    public:
        // Used by the renderer to animate the unit's position between two tiles.
        float moveTime = -1;
        glm::uvec2 moveFrom = glm::uvec2(0);

        Unit(std::shared_ptr<UnitKind> kind, glm::uvec2 pos, PlayerId owner);

        void setID(UnitId id);

        const UnitKind &getKind() const;
        glm::uvec2 getPos() const;
        UnitId getID() const;
        PlayerId getOwner() const;
        double getCombatStrength() const;
        int getMovementLeft() const;

        // Determines whether the unit can move to the given target
        // this turn.
        bool canMove(glm::uvec2 target, const Game &game) const;
        // Attempts to move the unit to a target position.
        // Does nothing if canMove(target) is false.
        void moveTo(glm::uvec2 target, Game &game);

        void onTurnEnd();
    };
}

#endif //RIPOSTE_UNIT_H
