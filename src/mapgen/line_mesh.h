//
// Created by Caelum van Ispelen on 8/30/21.
//

#ifndef RIPOSTE_LINE_MESH_H
#define RIPOSTE_LINE_MESH_H

#include <glm/vec2.hpp>
#include <glm/geometric.hpp>
#include <iostream>

#include <riposte_networking.h>

#include "grid.h"

namespace rip::mapgen {
    struct MeshBounds {
        glm::vec2 origin;
        glm::vec2 size;
    };

    // A mesh of lines. Can be rasterized into a Grid.
    class LineMesh {
        std::vector<glm::vec2> points;

    public:
        void addPoint(glm::vec2 point) noexcept;

        const std::vector<glm::vec2> &getPoints() const noexcept;
        std::vector<glm::vec2> &getPoints() noexcept;

        MeshBounds getBounds() const noexcept;

        void scale(float factor) noexcept;

        template<class T>
        Grid<T> rasterizeToGrid(T backgroundValue, T foregroundValue) {
            RipRasterizedMask *raster = zeno_rasterize_lines((float*) points.data(), points.size());

            const auto gridWidth = zeno_mask_get_width(raster);
            const auto gridHeight = zeno_mask_get_height(raster);

            Grid<T> targetGrid(gridWidth, gridHeight, backgroundValue);

            for (int y = 0; y < gridHeight; y++) {
                for (int x = 0; x < gridWidth; x++) {
                    uint8_t rasterValue = zeno_mask_get_value(raster, x, y);
                    if (rasterValue > 0) {
                        targetGrid.set(x, y, foregroundValue);
                    }
                }
            }

            return targetGrid;
        }
    };
}

#endif //RIPOSTE_LINE_MESH_H
