//
// Created by Caelum van Ispelen on 8/30/21.
//

#include "grid.h"
#include "line_mesh.h"
#include "../rng.h"

#define STB_IMAGE_WRITE_IMPLEMENTATION
#include <stb_image_write.h>

#include <FastNoise/FastNoise.h>

using namespace rip::mapgen;

enum class Cell {
    Land,
    Ocean,
};

#define PI 3.14159265

template<class T>
std::vector<float> generateTileableNoise(FastNoise::SmartNode<T> &noise, uint32_t length, int seed, float frequency) {
    std::vector<float> result(length);

    float circumference = length;
    float radius = circumference / (2 * PI);

    // 2D circle to ensure tileability
    for (int i = 0; i < length; i++) {
        float theta = (static_cast<float>(i) / length) * 2 * PI;
        float x = std::cos(theta) * radius * frequency;
        float y = std::sin(theta) * radius * frequency;

        result[i] = noise->GenSingle2D(x, y, seed);
    }

    return result;
}

int main() {
    rip::Rng rng;

    int scale = 64;

    int initialGridSize = 32 * scale;
    Grid<Cell> grid(initialGridSize, initialGridSize, Cell::Ocean);

    auto noiseFnA = FastNoise::New<FastNoise::FractalFBm>();
    noiseFnA->SetSource(FastNoise::New<FastNoise::Simplex>());
    noiseFnA->SetOctaveCount(8);
    std::vector<float> noiseA(grid.getWidth() * grid.getHeight());
    noiseFnA->GenUniformGrid2D(noiseA.data(), 0, 0, grid.getWidth(), grid.getHeight(), 0.06 / scale, rng.u32(0, 10000));

    for (int x = 0; x < grid.getWidth(); x++) {
        for (int y = 0; y < grid.getHeight(); y++) {
            float distance = glm::length(glm::vec2(x, y) - glm::vec2(grid.getWidth() / 2, grid.getHeight() / 2));

            float radius = 8 * scale;
            float noiseValueA = noiseA[x + y * grid.getWidth()];
            radius += noiseValueA * 12 * scale;

            if (distance <= radius) {
                grid.set(x, y, Cell::Land);
            }
        }
    }

    std::vector<unsigned char> pixels(grid.getWidth() * grid.getHeight() * 3);

    for (int x = 0; x < grid.getWidth(); x++) {
        for (int y = 0; y < grid.getHeight(); y++) {
            auto cell = grid.get(x, y);
            if (cell == Cell::Land) {
                pixels[(x + y * grid.getWidth()) * 3 + 1] = 255;
            } else {
                pixels[(x + y * grid.getWidth()) * 3 + 2] = 255;
            }
        }
    }

    stbi_write_png("mapgen.png", grid.getWidth(), grid.getHeight(), 3, (void*) pixels.data(), 3 * grid.getWidth());

    return 0;
}

