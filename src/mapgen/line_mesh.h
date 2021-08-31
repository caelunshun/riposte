//
// Created by Caelum van Ispelen on 8/30/21.
//

#ifndef RIPOSTE_LINE_MESH_H
#define RIPOSTE_LINE_MESH_H

#include <glm/vec2.hpp>
#include <glm/geometric.hpp>
#include <iostream>
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
            const auto bounds = getBounds();
            const auto gridWidth = static_cast<uint32_t>(std::ceil(bounds.size.x));
            const auto gridHeight = static_cast<uint32_t>(std::ceil(bounds.size.y));
            Grid<T> targetGrid(gridWidth, gridHeight, backgroundValue);

            // Rasterize edges with a line marching algorithm.
            for (int i = 0; i < points.size() - 1; i++) {
                const auto pointA = points[i];
                const auto pointB = points[i + 1];

                glm::vec2 ray = pointB - pointA;
                float posAlongRay = 0;
                const float rayLength = glm::length(ray);
                const float slope = (pointB.y - pointA.y) / (pointB.x - pointA.x);

                ray = glm::normalize(ray);

                double tmp;

                while (posAlongRay < rayLength) {
                    const glm::vec2 currentPoint = pointA + ray * posAlongRay;
                    std::cout << currentPoint.x << ", " << currentPoint.y << std::endl;
                    targetGrid.set(static_cast<int>(currentPoint.x - bounds.origin.x),
                                   static_cast<int>(currentPoint.y - bounds.origin.y), foregroundValue);

                    // Find next ceil to move to
                    float xDist = 1 - (std::modf(currentPoint.x, &tmp));
                    float yDist = 1 - (std::modf(currentPoint.y, &tmp));

                    float xDistAlongRay = glm::length(glm::vec2(xDist, xDist * slope));
                    float yDistAlongRay = glm::length(glm::vec2(yDist * (1 / slope), yDist));

                    if (xDistAlongRay < yDistAlongRay) {
                        posAlongRay += xDistAlongRay + 0.001;
                    } else {
                        posAlongRay += yDistAlongRay + 0.001;
                    }
                }
            }

            // Fill in the grid edges with a flood fill.

            return targetGrid;
        }
    };
}

#endif //RIPOSTE_LINE_MESH_H
