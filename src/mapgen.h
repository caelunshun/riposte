//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_MAPGEN_H
#define RIPOSTE_MAPGEN_H

#include "game.h"
#include "rng.h"

namespace rip {
    class TechTree;

    class MapGenerator {
        Rng rng;
    public:
        Game generate(uint32_t mapWidth, uint32_t mapHeight, std::shared_ptr<Registry> registry, const std::shared_ptr<TechTree> &techTree);
    };
}

#endif //RIPOSTE_MAPGEN_H
