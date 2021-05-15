//
// Created by Caelum van Ispelen on 5/12/21.
//

#include <bitset>
#include <array>
#include <FastNoise/FastNoise.h>
#include "mapgen.h"

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

    const auto numPlayers = 7;

    // CIVILIZATION GENERATOR
    void seedPlayers(Game &game, Rng &rng) {
        while (game.getNumPlayers() < numPlayers) {
            const auto &civs = game.getRegistry().getCivs();
            auto index = rng.u32(0, civs.size());
            auto civ = civs[index];

            Player player(civ->leader, civ, game.getMapWidth(), game.getMapHeight());
            auto playerID = game.addPlayer(std::move(player));
            game.getPlayer(playerID).setID(playerID);

            if (game.getNumPlayers() == 1) {
                game.setThePlayerID(playerID);
            }
        }
    }

    // CITY GENERATOR
    void placeCities(Game &game, Rng &rng) {
        for (auto &player : game.getPlayers()) {
            while(true) {
                auto x = rng.u32(0, game.getMapWidth());
                auto y = rng.u32(0, game.getMapHeight());
                glm::uvec2 pos(x, y);

                if (game.getTile(pos).getTerrain() == Terrain::Ocean) {
                    continue;
                }

                if (game.getCityAtLocation(pos) == nullptr) {
                    Unit settler(game.getRegistry().getUnits().at(0), pos, player.getID());
                    game.addUnit(std::move(settler));

                    Unit warrior(game.getRegistry().getUnits().at(1), pos + glm::uvec2(1, 0), player.getID());
                    game.addUnit(std::move(warrior));

                    break;
                }
            }
        }
    }

    // MAIN GENERATOR

    void buildTerrain(rip::Game &game, Rng &rng) {
        LandMap landMap(4, 4);
        const auto numContinents = 10;
        for (int continent = 0; continent < numContinents; continent++) {
            auto x = rng.u32(0, 4);
            auto y = rng.u32(0, 4);
            landMap.setLand(glm::uvec2(x, y), true);
        }

        const auto zooms = 4;
        for (int i = 0; i < zooms; i++) {
            landMap = landMap.grow(rng);
        }

        auto simplex = FastNoise::New<FastNoise::Simplex>();
        auto noise = FastNoise::New<FastNoise::FractalFBm>();
        noise->SetSource(simplex);
        std::vector<float> noiseOutput(game.getMapWidth() * game.getMapHeight());
        noise->GenUniformGrid2D(noiseOutput.data(), 0, 0, game.getMapWidth(), game.getMapHeight(), 10.0f, rng.u32(0, 0xFFFFFFFF));

        for (int x = 0; x < game.getMapWidth(); x++) {
            for (int y = 0; y < game.getMapHeight(); y++) {
                Terrain t;
                glm::uvec2 pos(x, y);
                if (landMap.isLand(pos)) {
                    auto noiseIndex = x + y * game.getMapWidth();
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
            }
        }
    }

    void MapGenerator::generate(rip::Game &game) {
        buildTerrain(game, rng);
        seedPlayers(game, rng);
        placeCities(game, rng);
    }
}
