//
// Created by Caelum van Ispelen on 5/12/21.
//

#include "mapgen.h"
#include "tech.h"
#include "unit.h"
#include "tile.h"
#include "game.h"
#include "mapgen/grid.h"
#include "mapgen/land.h"
#include "mapgen/starting_locations.h"
#include "mapgen/terrain.h"
#include "mapgen/resources.h"

namespace rip {
    std::pair<Game, std::map<uint32_t, PlayerId>> MapGenerator::generate(const std::vector<proto::LobbySlot> &playerSlots, mapgen::MapgenSettings settings,
                                std::shared_ptr<Registry> registry, const std::shared_ptr<TechTree> &techTree,
                                Server *server) {
        Game game(settings.mapwidth(), settings.mapheight(), registry, techTree);
        game.setServer(server);

        std::cerr << 22 << std::endl;

        std::unique_ptr<mapgen::LandGenerator> landGen;
        if (settings.has_continents()) {
            landGen = std::make_unique<mapgen::ContinentsGenerator>(settings.continents());
        } else {
            throw std::string("invalid setting");
        }

        std::cerr << 31 << std::endl;

        const auto landGrid = landGen->generateLandGrid(settings.mapwidth(), settings.mapheight(), rng);

        std::cerr << 35 << std::endl;

        mapgen::DefaultTerrainGenerator terrainGen;
        const auto tileGrid = terrainGen.generateTerrain(landGrid, rng);

        std::cerr << 40 << std::endl;

        int numPlayers = 0;
        for (const auto &slot : playerSlots) {
            if (slot.occupied()) ++numPlayers;
        }

        mapgen::StartingLocationsGenerator startingLocGen;
        const auto startingLocations = startingLocGen.generateStartingLocations(landGrid, tileGrid, rng, numPlayers);

        mapgen::BalancedResourceGenerator resourceGen;
        const auto resourceGrid = resourceGen.distributeResources(rng, *registry, tileGrid, startingLocations);

        std::cerr << 48 << std::endl;

        // Copy the tile grid into the Game.
        for (int y = 0; y < settings.mapheight(); y++) {
            for (int x = 0; x < settings.mapwidth(); x++) {
                game.setTile(glm::uvec2(x, y), tileGrid.get(x, y));

                const auto &resource = resourceGrid.get(x, y);
                if (resource.has_value()) {
                    game.getTile(glm::uvec2(x, y)).setResource(*resource);
                }
            }
        }

        std::map<uint32_t, PlayerId> playerIDMapping;

        // Create players and spawn their initial units.
        int playerIDCounter = 0;
        for (const auto &slot : playerSlots) {
            if (!slot.occupied()) continue;

            int playerID = playerIDCounter++;
            const auto startingLocation = startingLocations[playerID];

            const auto civ = registry->getCiv(slot.civid());

            std::optional<Leader> leader;
            for (const auto &l : civ->leaders) {
                if (l.name == slot.leadername()) {
                    leader = l;
                }
            }

            if (!leader.has_value()) {
                throw std::string("invalid leader name");
            }

            const auto player = game.addPlayer(Player("", civ, *leader, settings.mapwidth(), settings.mapheight(), techTree));
            playerIDMapping[slot.id()] = player;

            if (slot.isai()) {
                game.getPlayer(player).enableAI();
            }

            Unit settler(registry->getUnit("settler"), startingLocation, player);
            game.addUnit(std::move(settler));

            std::shared_ptr<UnitKind> unitKind;
            if (game.getPlayer(player).getTechs().isTechUnlocked("Hunting")) {
                unitKind = registry->getUnit("scout");
            } else {
                unitKind = registry->getUnit("warrior");
            }

            auto possibleUnitPositions = getNeighbors(startingLocation);
            rng.shuffle(possibleUnitPositions.begin(), possibleUnitPositions.end());

            glm::uvec2 unitPos;
            for (const auto pos : possibleUnitPositions) {
                if (game.getTile(pos).getTerrain() != Terrain::Ocean) {
                    unitPos = pos;
                }
            }

            game.addUnit(Unit(unitKind, unitPos, player));

            game.getPlayer(player).recomputeVisibility(game);
        }

        return std::pair<Game, std::map<uint32_t, PlayerId>>(std::move(game), std::move(playerIDMapping));
    }
}
