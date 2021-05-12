//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_MAPGEN_H
#define RIPOSTE_MAPGEN_H

#include "game.h"
#include "rng.h"

namespace rip {
    class MapGenerator {
        Rng rng;
    public:
        void generate(Game &game);
    };
}

#endif //RIPOSTE_MAPGEN_H
