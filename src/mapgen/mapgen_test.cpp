//
// Created by Caelum van Ispelen on 8/30/21.
//

#include "grid.h"
#include "line_mesh.h"

#define STB_IMAGE_WRITE_IMPLEMENTATION
#include <stb_image_write.h>

using namespace rip::mapgen;

enum class Cell {
    Foreground,
    Background,
};

int main() {
    LineMesh mesh;



    // Circle
    for (float theta = 0; theta <= 2 * 3.142; theta += 0.1) {
        float x = std::cos(theta) + 5;
        float y = std::sin(theta) + 5;
        mesh.addPoint(glm::vec2(x, y));
    }

    mesh.scale(100);

    const Grid<Cell> grid = mesh.rasterizeToGrid(Cell::Background, Cell::Foreground);

    std::vector<unsigned char> pixels(grid.getWidth() * grid.getHeight() * 3);

    for (int x = 0; x < grid.getWidth(); x++) {
        for (int y = 0; y < grid.getHeight(); y++) {
            auto cell = grid.get(x, y);
            if (cell == Cell::Foreground) {
                pixels[(x + y * grid.getWidth()) * 3] = 255;
            }
        }
    }

    stbi_write_png("mapgen.png", grid.getWidth(), grid.getHeight(), 3, (void*) pixels.data(), 3 * grid.getWidth());

    return 0;
}

