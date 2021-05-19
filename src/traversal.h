//
// Created by caelum on 5/18/21.
//

#ifndef RIPOSTE_TRAVERSAL_H
#define RIPOSTE_TRAVERSAL_H

#include <glm/vec2.hpp>
#include <deque>
#include <absl/container/flat_hash_set.h>
#include "ripmath.h"
#include "game.h"

namespace rip {
    // Runs a breadth-first search on the tile grid.
    // Invokes `callback` for every visited tile.
    // Invokes `shouldVisit` to determine whether a tile (and subsequently its
    // descendents) should be visited.
    // Both functions take a Tile& and glm::uvec2 as arguments.
    template<class Callback, class ShouldVisit>
    void breadthFirstSearch(Game &game, glm::uvec2 startPos, Callback callback, ShouldVisit shouldVisit) {
        std::deque<glm::uvec2> queue;
        queue.push_back(startPos);

        absl::flat_hash_set<glm::uvec2, PosHash> visited;
        visited.insert(startPos);

        while (!queue.empty()) {
            auto current = queue[0];
            queue.pop_front();

            auto &tile = game.getTile(current);
            callback(tile, current);

            for (const auto neighbor : getNeighbors(current)) {
                if (!game.containsTile(neighbor)) {
                    continue;
                }
                if (visited.contains(neighbor)) {
                    continue;
                }
                visited.insert(neighbor);
                auto &neighborTile = game.getTile(neighbor);
                if (shouldVisit(neighborTile, neighbor)) {
                    queue.push_back(neighbor);
                }
            }
        }
    }
}

#endif //RIPOSTE_TRAVERSAL_H
