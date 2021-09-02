//
// Created by Caelum van Ispelen on 9/2/21.
//

#include "starting_locations.h"

#include <queue>

namespace rip::mapgen {
    // Step 1: distribute players to specific continents.
    //
    // We prefer larger continents over smaller ones,
    // and we try to ensure each continent gets either zero or at least two players.
    std::vector<uint32_t>
    StartingLocationsGenerator::assignContinents(const std::vector<std::vector<glm::uvec2>> &continents, Rng &rng,
                                                 int numPlayers) {
        std::vector<uint32_t> playerContinentAssignments(numPlayers);

        for (int player = 0; player < numPlayers; player++) {
            std::vector<float> continentScores(continents.size());
            for (int continent = continents.size() - 1; continent >= 0; continent--) {
                // Count players on this continent.
                int numPlayersOnContinent = 0;
                for (int i = 0; i < player; i++) {
                    if (playerContinentAssignments[i] == continent) {
                        ++numPlayersOnContinent;
                    }
                }

                // Heuristic: each player consumes 100 tiles for a continent.
                // We assign a score based on the remaining tiles for this continent.
                continentScores[continent] = (continents[continent].size() - 100 * numPlayersOnContinent);

                // If the continent is very small, we should never assign it.
                if (continents[continent].size() < 30) {
                    continentScores[continent] = -1000000;
                }
            }

            // Select continent with the highest score.
            std::optional<int> bestContinent;
            for (int c = 0; c < continentScores.size(); c++) {
                if (!bestContinent.has_value()
                    || continentScores[c] > continentScores[*bestContinent]) {
                    bestContinent = c;
                }
            }

            playerContinentAssignments[player] = *bestContinent;
        }

        // Shuffle player continent assignments so player 0 doesn't
        // always get the biggest one.
        rng.shuffle(playerContinentAssignments.begin(), playerContinentAssignments.end());

        return playerContinentAssignments;
    }

    float StartingLocationsGenerator::scoreTile(const Grid<Tile> &tileGrid, glm::uvec2 pos,
                                                const std::vector<glm::uvec2> &otherStartingLocations) {
        const auto &tile = tileGrid.get(pos.x, pos.y);

        float score = 0;

        const auto terrain = tile.getTerrain();
        if (terrain == Terrain::Grassland) {
            score += 10;
        } else if (terrain == Terrain::Desert) {
            score -= 50;
        }

        if (tile.isHilled()) {
            score += 5;
        }

        for (const auto otherPos : otherStartingLocations) {
            float d = dist(pos, otherPos);

            score -= 100 / d;
        }

        // Big fat cross scores
        for (const auto bfcPos : getBigFatCross(pos)) {
            if (bfcPos.x >= tileGrid.getWidth() || bfcPos.y >= tileGrid.getHeight()) {
                // Out of bounds
                score -= 3;
                continue;
            }

            const auto &bfcTile = tileGrid.get(bfcPos.x, bfcPos.y);

            const auto bfcTerrain = bfcTile.getTerrain();
            if (bfcTerrain == Terrain::Grassland || bfcTerrain == Terrain::Ocean) {
                score += 2;
            } else if (bfcTerrain == Terrain::Desert) {
                score -= 2;
            } else if (bfcTerrain == Terrain::Plains) {
                score += 1;
            }
        }

        return score;
    }

    std::vector<glm::uvec2>
    StartingLocationsGenerator::generateStartingLocations(const Grid<LandCell> &landGrid, const Grid<Tile> &tileGrid,
                                                          Rng &rng, int numPlayers) {
        auto continents = landGrid.withAssignedIDs().groupToPositions;
        // Sort continents by size
        std::sort(continents.begin(), continents.end(), [](const std::vector<glm::uvec2> &a, const std::vector<glm::uvec2> &b) {
            return a.size() < b.size();
        });
        // Only respect land continents, not ocean ones
        for (int c = continents.size() - 1; c >= 0; c--) {
            auto pos = continents[c].at(0);
            if (landGrid.get(pos.x, pos.y) == LandCell::Ocean) {
                continents.erase(continents.begin() + c);
            }
        }

        // Get each player's continent.
        const auto continentAssignments = assignContinents(continents, rng, numPlayers);

        // Step 2: Assign each player a starting location on the continent
        // they were assigned.
        //
        // For each player, we give each tile in their continent a score
        // depending on its food yield, the number of adjacent deserts,
        // the distance to other players' starting locations,
        // and its coastal status.
        std::vector<glm::uvec2> startingLocations;

        struct TileWithScore {
            glm::uvec2 pos;
            float score;

            bool operator<(const TileWithScore &other) const {
                return score < other.score;
            }
        };

        for (int player = 0; player < numPlayers; player++) {
            std::priority_queue<TileWithScore> tileScores;
            for (const auto pos : continents[continentAssignments[player]]) {
                float tileScore = scoreTile(tileGrid, pos, startingLocations);

                tileScores.emplace(TileWithScore {
                    .pos = pos,
                    .score = tileScore,
                });
            }

            const auto posWithBestScore = tileScores.top().pos;
            startingLocations.push_back(posWithBestScore);
        }

        return startingLocations;
    }
}
