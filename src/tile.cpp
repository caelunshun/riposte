//
// Created by Caelum van Ispelen on 5/11/21.
//

#include "tile.h"
#include "city.h"
#include "game.h"

namespace rip {
    int Tile::getMovementCost() const {
        return (forested ? 2 : 1);
    }

    bool Tile::isForested() const {
        return forested;
    }

    void Tile::setForested(bool forested) {
        this->forested = forested;
    }

    Yield Tile::getYield(const Game &game, glm::uvec2 pos) const {
        Yield yield(0, 0, 0);

        switch (terrain) {
            case Grassland:
                yield.commerce += 1;
                yield.food += 2;
                break;
            case Plains:
                yield.food += 1;
                yield.hammers += 1;
                break;
            case Ocean:
                yield.food += 2;
                yield.commerce += 2;
                break;
            case Desert:
                break;
        }

        if (forested) {
            yield.hammers += 1;
        }

        if (game.getCityAtLocation(pos)) {
            yield.hammers += 1;
            yield.food += 1;
            yield.commerce += 1;
        }

        return yield;
    }
}
