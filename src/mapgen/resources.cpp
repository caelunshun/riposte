//
// Created by Caelum van Ispelen on 9/10/2021.
//

#include "resources.h"
#include "../ripmath.h"

#include <cmath>

namespace rip::mapgen {
    Grid<ResourceTile> BalancedResourceGenerator::distributeResources(Rng &rng, const Registry &registry, 
        const Grid<Tile> &tileGrid, 
        const std::vector<glm::uvec2> &startingLocations) {
        Grid<ResourceTile> resultGrid(tileGrid.getWidth(), tileGrid.getHeight(), {});

        int numBfcFoodResources = 0;

        // Spread instances of each resource according to a Poisson
        // distribution.
        for (const auto &[name, resource] : registry.getResources()) {
            std::vector<glm::uvec2> positions;
            std::deque<glm::uvec2> processingList;

            glm::uvec2 startingPoint(rng.u32(0, resultGrid.getWidth()), rng.u32(0, resultGrid.getHeight()));
            processingList.push_back(startingPoint);

            float minDist = 50.0 / resource->abundance;

            // For starting locations, we want guaranteed food resources.
            // Add one random position in the BFC to the grid.
            if (resource->improvedBonus.food > 0) {
                while (numBfcFoodResources < 2) {
                    for (const auto startingPosition : startingLocations) {
                        const auto bfc = getBigFatCross(startingPosition);
                        while (true) {
                            const auto pos = bfc[rng.u32(0, bfc.size())];
                            const auto &tile = tileGrid.get(pos.x, pos.y);
                            if (tile.getTerrain() != Terrain::Ocean && !(tile.getTerrain() == Terrain::Desert && !resource->allowDeserts)) {
                                resultGrid.set(pos.x, pos.y, resource);
                                positions.push_back(pos);
                                break;
                            }
                        }
                        ++numBfcFoodResources;
                    }
                }
            }

            while (!processingList.empty()) {
                const auto index = rng.u32(0, processingList.size());
                const auto currentPoint = processingList[index];
                processingList.erase(processingList.begin() + index);

                for (int i = 0; i < 15; i++) {
                    // Generate neighbors in a ring around the current point
                    float r1 = rng.f32();
                    float r2 = rng.f32();
                    float radius = minDist * (r1 + 1);
                    float angle = 2 * pi() * r2;
                    glm::uvec2 neighborPoint(static_cast<int>(currentPoint.x + std::cos(angle) * radius), 
                        static_cast<int>(currentPoint.y + std::sin(angle) * radius));

                    if (neighborPoint.x < 0 || neighborPoint.y < 0 
                        || neighborPoint.x >= resultGrid.getWidth() || neighborPoint.y >= resultGrid.getHeight()) {
                            continue;
                    }

                    // Check for points too close to neighborPoint
                    // PERF: terrible quadratic time complexity, maybe fix with
                    // a spatial grid or a tree acceleration structure?
                    bool isValid = true;
                    for (const auto &existingPoint : positions) {
                        if (dist(existingPoint, neighborPoint) < minDist) {
                            isValid = false;
                            break;
                        }
                    }

                    const auto &tile = tileGrid.get(neighborPoint.x, neighborPoint.y);
                    if (tile.getTerrain() == Terrain::Ocean || (tile.getTerrain() == Terrain::Desert && !resource->allowDeserts)) {
                        isValid = false;
                    }

                    if (isValid) {
                        processingList.push_back(neighborPoint);
                        positions.push_back(neighborPoint);
                    }
                }
            }

            std::cerr << "Generated " << positions.size() << " instances of resource " << resource->name << std::endl;
            for (const auto pos : positions) {
                resultGrid.set(pos.x, pos.y, resource);
            }
        }

        return resultGrid;
    }
}
