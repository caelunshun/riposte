//
// Created by Caelum van Ispelen on 5/11/21.
//

#include "tile.h"

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
}
