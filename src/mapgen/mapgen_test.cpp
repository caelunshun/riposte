//
// Created by Caelum van Ispelen on 8/30/21.
//

#include "grid.h"
#include "line_mesh.h"
#include "../rng.h"
#include "land.h"

#include <mapgen.pb.h>

#define STB_IMAGE_WRITE_IMPLEMENTATION
#include <stb_image_write.h>

#include <FastNoise/FastNoise.h>

using namespace rip::mapgen;

int main() {
    rip::Rng rng;

    ContinentsSettings settings;
    settings.set_numcontinents(NumContinents::Two);
    ContinentsGenerator generator(settings);

    const auto grid = generator.generateLandGrid(80,
                                                 48, rng);

    std::vector<unsigned char> pixels(grid.getWidth() * grid.getHeight() * 3);

    for (int x = 0; x < grid.getWidth(); x++) {
        for (int y = 0; y < grid.getHeight(); y++) {
            auto cell = grid.get(x, y);
            if (cell == LandCell::Land) {
                pixels[(x + y * grid.getWidth()) * 3 + 1] = 255;
            } else {
                pixels[(x + y * grid.getWidth()) * 3 + 2] = 255;
            }
        }
    }

    stbi_write_png("mapgen.png", grid.getWidth(), grid.getHeight(), 3, (void*) pixels.data(), 3 * grid.getWidth());

    return 0;
}

