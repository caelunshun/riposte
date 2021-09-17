//
// Created by Caelum van Ispelen on 9/1/21.
//

#ifndef RIPOSTE_TERRAIN_H
#define RIPOSTE_TERRAIN_H

#include "../tile.h"
#include "../rng.h"
#include "grid.h"
#include "land.h"

namespace rip::mapgen {
    // Given a land map indicating which tiles are land or ocean,
    // a terrain generator is responsible for setting tile terrains -
    // deciding whether to use grassland/plains/desert, determining
    // where to put hills, etc.
    class TerrainGenerator {
    public:
        virtual ~TerrainGenerator() = default;

        virtual Grid<Tile> generateTerrain(const Grid<LandCell> &landGrid, Rng &rng) = 0;
    };

    // The default terrain generator, which uses a series
    // of noises to create semi-realistic terrain.
    class DefaultTerrainGenerator : public TerrainGenerator {
    public:
        Grid<Tile> generateTerrain(const Grid<LandCell> &landGrid, Rng &rng) override;
    };
}

#endif //RIPOSTE_TERRAIN_H
