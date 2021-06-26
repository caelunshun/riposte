//
// Created by Caelum van Ispelen on 5/12/21.
//

#include <bitset>
#include <array>
#include <glm/glm.hpp>
#include <FastNoise/FastNoise.h>
#include <absl/container/flat_hash_set.h>
#include "mapgen.h"
#include "tech.h"
#include "unit.h"
#include "tile.h"

namespace rip {
    // CONTINENT GENERATION
    // Operates on a bitset. false=ocean, true=land.

    uint32_t getZoomSize(uint32_t oldSize) {
        return oldSize * 2 + 1;
    }

    class LandMap {
        std::vector<bool> map;
        uint32_t mapWidth;
        uint32_t mapHeight;

    public:
        LandMap(uint32_t mapWidth, uint32_t mapHeight) : map(mapWidth * mapHeight), mapWidth(mapWidth), mapHeight(mapHeight) {}

        bool isLand(glm::uvec2 pos) const {
            return map[pos.x + pos.y * mapWidth];
        }

        void setLand(glm::uvec2 pos, bool isLand) {
            map[pos.x + pos.y * mapWidth] = isLand;
        }

        /**
         * Grows the map randomly. The returned map has dimensions n*2+1.
         */
        LandMap grow(Rng &rng) const {
            // For each pair of adjacent values in the original grid,
            // output 3 new values where the value in between is randomly
            // selected between the two other values.
            //
            // For example, let's say the input is a 2x2 grid:
            // a b
            // c d
            // The output will be a 3x3 grid with some random values based on their neighbors:
            // a         (a or b)           b
            // (a or c)  (a or b or c or d) (b or d)
            // c         (c or b)           d
            //
            // This technique was pioneered by the Cuberite project
            // for generating biome grids for Minecraft. For more information,
            // see http://cuberite.xoft.cz/docs/Generator.html#biomegen; scroll down to
            // "Grown biomes."
            const auto newWidth = getZoomSize(mapWidth);
            const auto newHeight = getZoomSize(mapHeight);

            LandMap result(newWidth, newHeight);
            for (int x = 0; x < mapWidth; x++) {
                for (int y = 0; y < mapHeight; y++) {
                    const auto targetX = 2 * (x + 1) - 2;
                    const auto targetY = 2 * (y + 1) - 2;
                    glm::uvec2 target(targetX, targetY);
                    glm::uvec2 pos(x, y);

                    // this tile
                    const auto current = isLand(pos);
                    result.setLand(target, current);

                    auto onEdgeX = (x == mapWidth - 1);
                    auto onEdgeY = (y == mapHeight - 1);

                    // 1 to the right
                    if (!onEdgeX) {
                        auto nextX = isLand(pos + glm::uvec2(1, 0));
                        std::array<bool, 2> choices({current, nextX});
                        result.setLand(target + glm::uvec2(1, 0), rng.choose(choices));
                    }

                    // 1 down
                    if (!onEdgeY) {
                        auto nextY = isLand(pos + glm::uvec2(0, 1));
                        std::array<bool, 2> choices({current, nextY});
                        result.setLand(target + glm::uvec2(0, 1), rng.choose(choices));
                    }

                    // diagonally
                    if (!onEdgeX && !onEdgeY) {
                        auto nextX = isLand(pos + glm::uvec2(1, 0));
                        auto nextY = isLand(pos + glm::uvec2(0, 1));
                        auto diagonal = isLand(pos + glm::uvec2(1, 1));
                        std::array<bool, 4> choices({current, nextX, nextY, diagonal});
                        result.setLand(target + glm::uvec2(1, 1), rng.choose(choices));
                    }
                }
            }
            return result;
        }
    };

    // NB: most functions here return bools that indicate
    // whether generating was successful. If not, we need
    // to start over with a new seed.

    const auto numPlayers = 7;

    // CIVILIZATION GENERATOR
    void seedPlayers(Game &game, const std::shared_ptr<TechTree> &techTree, Rng &rng) {
        absl::flat_hash_set<std::string> usedCivIDs;
        while (game.getNumPlayers() < numPlayers) {
            const auto &civs = game.getRegistry().getCivs();
            auto index = rng.u32(0, civs.size());
            auto civ = civs[index];

            if (usedCivIDs.contains(civ->id)) {
                continue;
            }
            usedCivIDs.insert(civ->id);

            assert(!civ->leaders.empty());
            auto leader = civ->leaders[rng.u32(0, civ->leaders.size())];

            Player player(leader.name, civ, leader, game.getMapWidth(), game.getMapHeight(), techTree);
            auto playerID = game.addPlayer(std::move(player));

            auto &p = game.getPlayer(playerID);
            p.setID(playerID);

           if (game.getNumPlayers() == 1) {
                game.setThePlayerID(playerID);
            } else { // DEBUG - AI
                p.enableAI();
            }
        }
    }

    // CITY GENERATOR
    bool placeCities(Game &game, Rng &rng) {
        std::vector<glm::uvec2> positions;
        for (auto &player : game.getPlayers()) {
            int attempts = 0;
            while(true) {
                if (++attempts > 1000) {
                    return false;
                }

                auto x = rng.u32(0, game.getMapWidth());
                auto y = rng.u32(0, game.getMapHeight());
                glm::uvec2 pos(x, y);

                const auto minDistToOtherCities = 12;
                bool foundClose = false;
                for (auto otherPos : positions) {
                    if (dist(pos, otherPos) < minDistToOtherCities) {
                        foundClose = true;
                        break;
                    }
                }
                if (foundClose) {
                    continue;
                }

                if (game.getTile(pos).getTerrain() == Terrain::Ocean) {
                   continue;
                }

                if (game.getCityAtLocation(pos) == nullptr) {
                    // Unit settler(game.getRegistry().getUnits().at(0), pos, player.getID());
                    // game.addUnit(std::move(settler));

                    player.createCity(pos, game);

                    glm::uvec2 warriorPos;
                    auto neighbors = getNeighbors(pos);
                    int att = 0;
                    while (true) {
                        warriorPos = rng.choose(neighbors);
                        if (game.getTile(warriorPos).getTerrain() != Terrain::Ocean) {
                           break;
                        }
                        if (++att > 100) {
                            return false;
                        }
                    }
                    Unit warrior(game.getRegistry().getUnit("warrior"), warriorPos, player.getID());
                    game.addUnit(std::move(warrior));

                    positions.push_back(pos);

                    break;
                }
            }
        }
        return true;
    }

    bool placeResources(Game &game, Rng &rng) {
        const auto numTiles = game.getMapWidth() * game.getMapHeight();
        for (const auto &entry : game.getRegistry().getResources()) {
            const auto &resource = entry.second;
            const auto minPlacements = resource->scarcity * (static_cast<float>(numTiles) / 1000);

            int placed = 0;
            int attempts = 0;
            while (placed < minPlacements) {
                if (++attempts > 1000) {
                    return false;
                }

                glm::uvec2 pos(rng.u32(0, game.getMapWidth()), rng.u32(0, game.getMapHeight()));
                auto &tile = game.getTile(pos);

                if (tile.hasResource() || tile.getTerrain() == Terrain::Ocean || tile.getTerrain() == Terrain::Desert) {
                    continue;
                }

                tile.setResource(resource);
                ++placed;
            }
        }
        return true;
    }

    // MAIN GENERATOR

    bool buildTerrain(Game &game, Rng &rng) {
        // Generate land/ocean map based on continents.
        const auto dim = 16;
        LandMap landMap(dim, dim);
        const auto numContinents = 3;
        const auto minSpacing = 7;
        const auto minDistFromEdge = 1;
        std::vector<glm::uvec2> continentCenters;

        int attempts = 0;
        while (continentCenters.size() < numContinents) {
            if (++attempts > 1000) {
                return false;
            }

            glm::uvec2 candidate(rng.u32(0, dim), rng.u32(0, dim));

            // Ensure we're a minimum distance away from all other continents
            // as well as the edges.
            bool valid = true;
            for (const auto otherPos : continentCenters) {
                if (dist(candidate, otherPos) < minSpacing) {
                    valid = false;
                    break;
                }
            }

            if (candidate.x < minDistFromEdge || candidate.y < minDistFromEdge
                || dim - candidate.x < minDistFromEdge || dim - candidate.y < minDistFromEdge) {
                valid = false;
            }

            if (valid) {
                continentCenters.push_back(candidate);
            }
        }

        // Write continents to the land map.

        auto continentNoiseGen = FastNoise::New<FastNoise::CellularValue>();
        std::vector<float> continentNoise(dim * dim);
        continentNoiseGen->GenUniformGrid2D(continentNoise.data(), 0, 0, dim, dim, 0.2, rng.u32(0, 0xFFFFFFFF));

        std::vector<glm::uvec2> landPositions;

        for (const auto center : continentCenters) {
            std::vector<glm::uvec2> continentLand;
            float noiseValue = continentNoise[center.x + dim * center.y];
            for (int x = 1; x < dim - 1; x++) {
                for (int y = 1; y < dim - 1; y++) {
                    float thisNoiseValue = continentNoise[x + dim * y];

                    // Prevent continents from merging.
                    bool isTooClose = false;
                    for (const auto otherPos : landPositions) {
                        if (isAdjacent(otherPos, glm::uvec2(x, y))) {
                            isTooClose = true;
                            break;
                        }
                    }

                    if (thisNoiseValue == noiseValue && !isTooClose) {
                        landMap.setLand(glm::uvec2(x, y), true);
                        continentLand.emplace_back(x, y);
                    }
                }
            }

            for (const auto p : continentLand) landPositions.push_back(p);
        }

        // Zoom the map to add detail.
        const auto zooms = 2;
        for (int i = 0; i < zooms; i++) {
            landMap = landMap.grow(rng);
        }

        // Set terrain types with a noise.
        auto cellular = FastNoise::New<FastNoise::CellularValue>();
        auto noise = FastNoise::New<FastNoise::FractalFBm>();
        noise->SetSource(cellular);
        auto treeNoise = FastNoise::New<FastNoise::FractalFBm>();
        treeNoise->SetSource(cellular);

        std::vector<float> noiseOutput(game.getMapWidth() * game.getMapHeight());
        noise->GenUniformGrid2D(noiseOutput.data(), 0, 0, game.getMapWidth(), game.getMapHeight(), 0.5, rng.u32(0, 0xFFFFFFFF));

        std::vector<float> treeNoiseOutput(game.getMapWidth() * game.getMapHeight());
        treeNoise->GenUniformGrid2D(treeNoiseOutput.data(), 0, 0, game.getMapWidth(), game.getMapHeight(), 5.0f, rng.u32(0, 0xFFFFFFFF));

        auto simplex = FastNoise::New<FastNoise::Simplex>();
        auto hillNoise = FastNoise::New<FastNoise::FractalFBm>();
        hillNoise->SetSource(simplex);

        std::vector<float> hillNoiseOutput(game.getMapWidth() * game.getMapHeight());
        hillNoise->GenUniformGrid2D(hillNoiseOutput.data(), 0, 0, game.getMapWidth(), game.getMapHeight(), 5.0f, rng.u32(0, 0xFFFFFFFF));

        for (int x = 0; x < game.getMapWidth(); x++) {
            for (int y = 0; y < game.getMapHeight(); y++) {
                auto noiseIndex = x + y * game.getMapWidth();
                Terrain t;
                glm::uvec2 pos(x, y);
                if (landMap.isLand(pos)) {
                    auto choice = noiseOutput[noiseIndex];
                    if (choice < -0.1) {
                        t = Terrain::Grassland;
                    } else if (choice < 0.4) {
                        t = Terrain::Plains;
                    } else {
                        t = Terrain::Desert;
                    }
                } else {
                    t = Terrain::Ocean;
                }

                game.getTile(pos).setTerrain(t);

                if (t != Terrain::Ocean) {
                    // Hills
                    if (hillNoiseOutput[noiseIndex] > 0.2) {
                        game.getTile(pos).setHilled(true);
                    }

                    // Forest
                    if (t != Terrain::Desert) {
                        if (treeNoiseOutput[noiseIndex] < 0.3) {
                            game.getTile(pos).setForested(true);
                        }
                    }
                }
            }
        }

        return true;
    }

    bool tryGenerate(Game &game, Rng &rng, const std::shared_ptr<TechTree> &techTree) {
        if (!buildTerrain(game, rng)) return false;
        seedPlayers(game, techTree, rng);
        if (!placeCities(game, rng)) return false;
        if (!placeResources(game, rng)) return false;
        return true;
    }

    Game MapGenerator::generate(uint32_t mapWidth, uint32_t mapHeight, std::shared_ptr<Registry> registry,
                                const std::shared_ptr<TechTree> &techTree) {
        while (true) {
            Game game(mapWidth, mapHeight, registry);
            if (tryGenerate(game, rng, techTree)) {
                return game;
            }
        }
    }
}
