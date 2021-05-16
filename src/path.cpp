//
// Created by Caelum van Ispelen on 5/16/21.
//

#include "path.h"
#include "game.h"
#include <queue>
#include <absl/container/flat_hash_map.h>
#include <absl/container/flat_hash_set.h>

namespace rip {
    Path::Path(std::vector<glm::uvec2> points) : points(std::move(points)) {}

    const std::vector<glm::uvec2> &Path::getPoints() const {
        return points;
    }

    size_t Path::getNumPoints() const {
        return points.size();
    }

    std::optional<glm::uvec2> Path::popNextPoint() {
        if (points.empty()) {
            return std::optional<glm::uvec2>();
        } else {
            auto p = points[0];
            points.erase(points.begin());
            return std::make_optional(p);
        }
    }

    glm::uvec2 Path::getDestination() const {
        return points.at(points.size() - 1);
    }

    std::optional<Path> computeShortestPath(const Game &game, glm::uvec2 source, glm::uvec2 target, std::optional<VisibilityMap> visibilityMask) {
        // A* algorithm.
        using Pos = std::pair<uint32_t, uint32_t>;
        using OpenEntry = std::pair<double, Pos>;

        std::priority_queue<OpenEntry, std::vector<OpenEntry>, std::greater<>> openSet;
        absl::flat_hash_set<Pos> inOpenSet;
        openSet.emplace(dist(source, target), Pos(source.x, source.y));
        inOpenSet.emplace(Pos(source.x, source.y));

        absl::flat_hash_map<Pos, glm::uvec2> cameFrom;

        absl::flat_hash_map<Pos, double> gScore;
        gScore[Pos(source.x, source.y)] = 0;

        absl::flat_hash_map<Pos, double> fScore;
        fScore[Pos(source.x, source.y)] = dist(source, target);

        while (!openSet.empty()) {
            auto entry = openSet.top();
            openSet.pop();
            auto current = glm::uvec2(entry.second.first, entry.second.second);
            auto currentPos = entry.second;
            inOpenSet.erase(currentPos);
            if (currentPos == Pos(target.x, target.y)) {
                // Found a path.
                std::vector<glm::uvec2> points;
                points.push_back(target);
                auto curr = target;
                while (cameFrom.contains(Pos(curr.x, curr.y))) {
                    curr = cameFrom[Pos(curr.x, curr.y)];
                    points.insert(points.begin(), curr);
                }
                return std::make_optional<Path>(std::move(points));
            }

            auto neighbors = getNeighbors(current);
            for (const auto neighbor : neighbors) {
                if (!game.containsTile(neighbor)) {
                    continue;
                }

                if (visibilityMask.has_value() && (*visibilityMask)[neighbor] == Visibility::Hidden) {
                    continue;
                }

                const auto &tile = game.getTile(neighbor);
                if (tile.getTerrain() == Terrain::Ocean) {
                    continue;
                }

                auto tentativeGScore = gScore[currentPos] + tile.getMovementCost();
                const Pos neighborPos(neighbor.x, neighbor.y);
                if (!gScore.contains(neighborPos) || tentativeGScore < gScore[neighborPos]) {
                    cameFrom[neighborPos] = current;
                    gScore[neighborPos] = tentativeGScore;
                    auto fScoreVal = tentativeGScore + dist(neighbor, target);
                    fScore[neighborPos] = fScoreVal;

                    if (!inOpenSet.contains(neighborPos)) {
                        inOpenSet.emplace(neighborPos);
                        openSet.emplace(fScoreVal, neighborPos);
                    }
                }
            }
        }

        // No path found.
        return std::optional<Path>();
    }
}
