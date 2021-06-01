//
// Created by Caelum van Ispelen on 5/16/21.
//

#ifndef RIPOSTE_PATH_H
#define RIPOSTE_PATH_H

#include <vector>
#include <optional>
#include <glm/vec2.hpp>
#include "player.h"

namespace rip {
    class Game;

    // A path between two points on the map.
    class Path {
        std::vector<glm::uvec2> points;

    public:
        Path(std::vector<glm::uvec2> points);

        const std::vector<glm::uvec2> &getPoints() const;

        size_t getNumPoints() const;

        // Pops the next point from the path, returning it.
        std::optional<glm::uvec2> popNextPoint();

        glm::uvec2 getDestination() const;
    };

    // Computes a shortest path between two points on the map.
    std::optional<Path> computeShortestPath(const Game &game, glm::uvec2 source, glm::uvec2 target, std::optional<VisibilityMap> visibilityMask, const UnitKind &unit);
}

#endif //RIPOSTE_PATH_H
