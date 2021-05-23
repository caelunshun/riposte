//
// Created by Caelum van Ispelen on 5/11/21.
//

#include "game.h"

namespace rip {
    void Game::advanceTurn() {
        for (auto &unit : units) {
            unit.onTurnEnd(*this);
        }

        tradeRoutes.updateResources(*this);

        for (auto &city : cities) {
            city.onTurnEnd(*this);
        }

        for (auto &player : players) {
            player.onTurnEnd(*this);
        }

        cultureMap.onTurnEnd(*this);

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

    Era Game::getEra() const {
        if (turn < 50) {
            return Era::Ancient;
        } else if (turn < 150) {
            return Era::Classical;
        } else if (turn < 250) {
            return Era::Medieval;
        } else if (turn < 300) {
            return Era::Renaissance;
        } else if (turn < 400) {
            return Era::Industrial;
        } else if (turn < 450) {
            return Era::Modern;
        } else {
            return Era::Future;
        }
    }
}
