//
// Created by Caelum van Ispelen on 9/2/21.
//

#ifndef RIPOSTE_STARTING_LOCATIONS_H
#define RIPOSTE_STARTING_LOCATIONS_H

#include "grid.h"
#include "land.h"
#include "../tile.h"
#include "../rng.h"

namespace rip::mapgen {
    // Responsible for determining where each player starts
    // (i.e., capital city locations)
    class StartingLocationsGenerator {
        std::vector<uint32_t> assignContinents(const std::vector<std::vector<glm::uvec2>> &continents, Rng &rng, int numPlayers);

        float scoreTile(const Grid<Tile> &tileGrid, glm::uvec2 pos, const std::vector<glm::uvec2> &otherStartingLocations);

    public:
        std::vector<glm::uvec2> generateStartingLocations(const Grid<LandCell> &landGrid, const Grid<Tile> &tileGrid, Rng &rng, int numPlayers);
    };
}

#endif //RIPOSTE_STARTING_LOCATIONS_H
