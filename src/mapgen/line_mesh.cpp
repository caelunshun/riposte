//
// Created by Caelum van Ispelen on 8/30/21.
//

#include "line_mesh.h"

namespace rip::mapgen {
    void LineMesh::addPoint(glm::vec2 point) noexcept {
        points.push_back(point);
    }

    const std::vector<glm::vec2> &LineMesh::getPoints() const noexcept {
        return points;
    }

    std::vector<glm::vec2> &LineMesh::getPoints() noexcept {
        return points;
    }

    MeshBounds LineMesh::getBounds() const noexcept {
        MeshBounds bounds {
            .origin = glm::vec2(std::numeric_limits<float>::infinity(), std::numeric_limits<float>::infinity()),
            .size = glm::vec2(-std::numeric_limits<float>::infinity(), -std::numeric_limits<float>::infinity()),
        };

        for (const auto point : getPoints()) {
            if (point.x < bounds.origin.x) {
                bounds.origin.x = point.x;
            }
            if (point.y < bounds.origin.y) {
                bounds.origin.y = point.y;
            }
            if (point.x > bounds.size.x) {
                bounds.size.x = point.x;
            }
            if (point.y > bounds.size.y) {
                bounds.size.y = point.y;
            }
        }

        bounds.size -= bounds.origin;

        return bounds;
    }

    void LineMesh::scale(float factor) noexcept {
        for (auto &point : getPoints()) {
            point *= factor;
        }
    }
}
