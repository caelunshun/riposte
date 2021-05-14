//
// Created by Caelum van Ispelen on 5/13/21.
//

#include "unit.h"
#include "game.h"
#include "ripmath.h"

namespace rip {
    void Unit::resetMovement() {
        movementLeft = kind->movement;
    }

    Unit::Unit(std::shared_ptr<UnitKind> kind, glm::uvec2 pos, PlayerId owner) : kind(std::move(kind)), pos(pos), owner(owner) {
        health = 1;
        resetMovement();
    }

    void Unit::setID(UnitId id) {
        this->id = id;
    }

    const UnitKind &Unit::getKind() const {
        return *kind;
    }

    glm::uvec2 Unit::getPos() const {
        return pos;
    }

    UnitId Unit::getID() const {
        return id;
    }

    PlayerId Unit::getOwner() const {
        return owner;
    }

    double Unit::getCombatStrength() const {
        return health * kind->strength;
    }

    int Unit::getMovementLeft() const {
        return movementLeft;
    }

    bool Unit::canMove(glm::uvec2 target, const Game &game) const {
        if (!game.containsTile(target)) {
            return false;
        }

        if (dist(target, pos) > movementLeft) {
            return false;
        }

        if (game.getTile(target).getTerrain() == Terrain::Ocean) {
            return false;
        }

        return true;
    }

    void Unit::moveTo(glm::uvec2 target, Game &game) {
        if (!canMove(target, game)) return;

        auto d = dist(target, pos);
        movementLeft -= ceil(d);
        pos = target;

        // Unit has moved; update visibility
        game.getPlayer(owner).recomputeVisibility(game);
    }
}
