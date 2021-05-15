//
// Created by Caelum van Ispelen on 5/11/21.
//

#include "game.h"

namespace rip {
    void Game::advanceTurn() {
        for (auto &unit : units) {
            unit.onTurnEnd();
        }

        ++turn;
    }

    std::optional<UnitId> Game::getNextUnitToMove() const {
        for (const auto &unit : units) {
            if (unit.getMovementLeft() != 0 && unit.getOwner() == thePlayer) {
                return std::make_optional<UnitId>(unit.getID());
            }
        }

        return std::optional<UnitId>();
    }
}
