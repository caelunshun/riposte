//
// Created by Caelum van Ispelen on 9/1/21.
//

#include <glm/vec2.hpp>
#include <glm/geometric.hpp>
#include <FastNoise/FastNoise.h>

#include "land.h"

namespace rip::mapgen {
    ContinentsGenerator::ContinentsGenerator(ContinentsSettings settings) : settings(std::move(settings)) {

    }

    Grid<LandCell> ContinentsGenerator::generateSingleContinent(uint32_t width, uint32_t height, Rng &rng) {
        Grid<LandCell> grid(width, height, LandCell::Ocean);

        auto noiseFnA = FastNoise::New<FastNoise::FractalFBm>();
        noiseFnA->SetSource(FastNoise::New<FastNoise::Simplex>());
        noiseFnA->SetOctaveCount(8);
        std::vector<float> noiseA(width * height);
        float scale = static_cast<float>(width) / 32.0;
        float frequency = 0.06 / scale;
        noiseFnA->GenUniformGrid2D(noiseA.data(), 0, 0, width, height, frequency, rng.u32(0, std::numeric_limits<uint32_t>::max()));

        float baseRadius = 12 * scale;
        glm::vec2 center(width / 2, height / 2);

        // Circle distorted by 2D Simplex noise.
        for (int x = 0; x < grid.getWidth(); x++) {
            for (int y = 0; y < grid.getHeight(); y++) {
                float distance = glm::length(glm::vec2(x, y) - center);

                float noiseValueA = noiseA[x + y * width];

                float modifiedRadius = baseRadius + noiseValueA * 12 * scale;

                if (distance <= modifiedRadius) {
                    grid.set(x, y, LandCell::Land);
                }
            }
        }

        return grid;
    }

    int getNumContinents(const ContinentsSettings &settings) {
        return static_cast<int>(settings.numcontinents()) + 1;
    }

    void stampLand(LandCell &existing, const LandCell &stamp) {
        if (stamp == LandCell::Land) existing = LandCell::Land;
    }

    Grid<LandCell> ContinentsGenerator::generateLandGrid(uint32_t mapWidth, uint32_t mapHeight, Rng &rng) {
        Grid<LandCell> grid(mapWidth, mapHeight, LandCell::Ocean);

        int continentWidth = (mapWidth - 2) / getNumContinents(settings);

        for (int i = 0; i < getNumContinents(settings); i++) {
            auto continent = generateSingleContinent(continentWidth, mapHeight - 2, rng);
            grid.stamp(continent, 1 + i * continentWidth, 1, continentWidth, mapHeight - 2, stampLand);
        }

        return grid;
    }
}
