//
// Created by Caelum van Ispelen on 8/30/21.
//

#include "grid.h"
#include "land.h"
#include "terrain.h"
#include "starting_locations.h"
#include "../rng.h"

#include <mapgen.pb.h>

#define STB_IMAGE_WRITE_IMPLEMENTATION
#include <stb_image_write.h>

#include <FastNoise/FastNoise.h>

using namespace rip::mapgen;

int main() {
    rip::Rng rng;

    ContinentsSettings settings;
    settings.set_numcontinents(NumContinents::Two);
    ContinentsGenerator continentGen(settings);

    const auto landGrid = continentGen.generateLandGrid(80,
                                                        48, rng);

    DefaultTerrainGenerator terrainGen;
    const auto tileGrid = terrainGen.generateTerrain(landGrid, rng);

    StartingLocationsGenerator startingLocGen;
    const auto startingLocations = startingLocGen.generateStartingLocations(landGrid, tileGrid, rng, 7);

    std::vector<unsigned char> pixels(landGrid.getWidth() * landGrid.getHeight() * 3);

    for (int x = 0; x < landGrid.getWidth(); x++) {
        for (int y = 0; y < landGrid.getHeight(); y++) {
            const auto &tile = tileGrid.get(x, y);

            std::array<uint8_t, 3> color;

            switch (tile.getTerrain()) {
                case rip::Ocean:
                    color = {30, 40, 220};
                    break;
                case rip::Plains:
                    color = {250, 224, 83};
                    break;
                case rip::Grassland:
                    color = {30, 220, 70};
                    break;
                case rip::Desert:
                    color = {255, 255, 255};
                    break;
            }

            if (std::find(startingLocations.begin(), startingLocations.end(), glm::uvec2(x, y)) != startingLocations.end()) {
                color = {0, 0, 0};
            }

            int baseIndex = (x + y * landGrid.getWidth()) * 3;
            pixels[baseIndex] = color[0];
            pixels[baseIndex + 1] = color[1];
            pixels[baseIndex + 2] = color[2];
        }
    }

    stbi_write_png("mapgen.png", landGrid.getWidth(), landGrid.getHeight(), 3, (void*) pixels.data(), 3 * landGrid.getWidth());

    return 0;
}

