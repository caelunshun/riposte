//
// Created by Caelum van Ispelen on 5/11/21.
//

#include "game.h"

namespace rip {
    void Game::advanceTurn() {
        for (auto &unit : units) {
            unit.onTurnEnd();
        }

        for (auto &city : cities) {
            city.onTurnEnd(*this);
        }

        ++turn;
    }

    std::optional<UnitId> Game::getNextUnitToMove() {
        for (auto &unit : units) {
            if (unit.getMovementLeft() != 0 && unit.getOwner() == thePlayer) {
                if (unit.hasPath()) {
                    unit.moveAlongCurrentPath(*this);
                } else {
                    return std::make_optional<UnitId>(unit.getID());
                }
            }
        }

        return std::optional<UnitId>();
    }
}
