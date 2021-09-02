//
// Created by Caelum van Ispelen on 9/1/21.
//

#ifndef RIPOSTE_LAND_H
#define RIPOSTE_LAND_H

#include <mapgen.pb.h>

#include "grid.h"
#include "../rng.h"

namespace rip::mapgen {
    enum class LandCell {
        Land,
        Ocean,
    };

    // Responsible for generating a grid indicating
    // which tiles are land and which are ocean.
    class LandGenerator {
    public:
        virtual ~LandGenerator() = default;
        virtual Grid<LandCell> generateLandGrid(uint32_t mapWidth, uint32_t mapHeight, Rng &rng) = 0;
    };

    class ContinentsGenerator : public LandGenerator {
        ContinentsSettings settings;

    public:
        explicit ContinentsGenerator(ContinentsSettings settings);

        Grid<LandCell> generateSingleContinent(uint32_t width, uint32_t height, Rng &rng);

        Grid<LandCell> generateLandGrid(uint32_t mapWidth, uint32_t mapHeight, Rng &rng) override;

        ~ContinentsGenerator() override = default;
    };
}

#endif //RIPOSTE_LAND_H
