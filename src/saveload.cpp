//
// Created by Caelum van Ispelen on 8/17/21.
//

#include "saveload.h"
#include "protocol.h"

#include "city.h"
#include "tile.h"
#include "unit.h"
#include "player.h"
#include "trade.h"

#include <ghc/filesystem.hpp>
#include <sstream>

#include <zstd.h>

namespace fs = ghc::filesystem;

namespace rip {
    const size_t headerSize = 128;

    std::string serializeGameToSave(Game &game, const std::vector<proto::LobbySlot> &lobbySlots, const absl::flat_hash_map<uint32_t, PlayerId> slotIDToPlayerID, std::string name) {
        GameSave packet;

        for (auto &player : game.getPlayers()) {
            packet.add_players()->CopyFrom(getUpdatePlayerPacket(game, player));
        }
        for (auto &city : game.getCities()) {
            packet.add_cities()->CopyFrom(getUpdateCityPacket(game, city));
        }
        for (auto &unit : game.getUnits()) {
            packet.add_units()->CopyFrom(getUpdateUnitPacket(game, unit));
        }

        packet.set_mapwidth(game.getMapWidth());
        packet.set_mapheight(game.getMapHeight());
        packet.set_turn(game.getTurn());

        for (int y = 0; y < game.getMapHeight(); y++) {
            for (int x = 0; x < game.getMapWidth(); x++) {
                setTile(game, PlayerId(0), glm::uvec2(x, y), game.getTile(glm::uvec2(x, y)), *packet.add_tiles());
            }
        }

        for (const auto &slot : lobbySlots) {
            packet.add_lobbyslots()->CopyFrom(slot);
        }

        for (const auto & [slotID, playerID] : slotIDToPlayerID) {
            (*packet.mutable_slotidtoplayerid())[slotID] = playerID.encode();
        }

        std::string data;

        SaveFileHeader header;
        header.set_name(name);
        header.set_turn(game.getTurn());
        header.AppendToString(&data);

        data.insert(data.begin(), data.size());
        while (data.size() < headerSize) {
            data.push_back(0);
        }

        std::string packetData;
        packet.AppendToString(&packetData);

        // Compress only the packet - not the header
        std::string compressedPacketData(ZSTD_compressBound(packetData.size()), '\0');
        const auto compressedSize = ZSTD_compress(compressedPacketData.data(), compressedPacketData.size(),
                      packetData.data(), packetData.size(), 6);
        compressedPacketData.erase(compressedSize, std::string::npos);

        data.append(compressedPacketData);

        return data;
    }

    proto::GameSave loadGameSaveFromBytes(std::string data) {
        // skip header
        data.erase(data.begin(), data.begin() + headerSize);

        // decompress
        const auto decompressedSize = ZSTD_getFrameContentSize(data.data(), data.size());
        std::string decompressedData(decompressedSize, '\0');
        ZSTD_decompress(decompressedData.data(), decompressedData.size(),
                        data.data(), data.size());

        GameSave packet;
        packet.ParseFromString(decompressedData);

        return packet;
    }

    SaveData loadGameFromSave(Server *server, proto::GameSave &packet, std::shared_ptr<Registry> registry,
                              std::shared_ptr<TechTree> techTree) {
        // First pass: compute ID mappings
        IdConverter playerIDs;
        IdConverter cityIDs;
        IdConverter unitIDs;
        for (auto &player : *packet.mutable_players()) {
            player.set_id(playerIDs.insert(player.id()).encode());
        }
        for (auto &city : *packet.mutable_cities()) {
            city.set_id(cityIDs.insert(city.id()).encode());
        }
        for (auto &unit : *packet.mutable_units()) {
            unit.set_id(unitIDs.insert(unit.id()).encode());
        }

        Game game(packet.mapwidth(), packet.mapheight(), registry, techTree);
        game.setServer(server);
        game.setTurn(packet.turn());

        // Second pass: create objects
        for (const auto &player : packet.players()) {
            game.addPlayer(Player(player, *registry, techTree, cityIDs, playerIDs, game.getMapWidth(), game.getMapHeight()));
        }
        for (auto &city : *packet.mutable_cities()) {
            game.loadCity(City(city, *registry, playerIDs));
        }
        for (const auto &unit : packet.units()) {
            game.addUnit(Unit(unit, playerIDs, unitIDs, *registry, UnitId(unit.id())));
        }

        for (int x = 0; x < game.getMapWidth(); x++) {
            for (int y = 0; y < game.getMapHeight(); y++) {
                glm::uvec2 pos(x, y);

                const auto &tile = packet.tiles(x + y * game.getMapWidth());
                game.setTile(pos, Tile(tile, playerIDs, *registry, pos));

                game.getCultureMap().setCulture(pos, getCultureFromProto(tile.culturevalues(), playerIDs));

                if (game.getTile(pos).hasImprovement<Road>()) {
                    game.getTradeRoutes().onRoadBuilt(game, pos);
                }
            }
        }

        game.getTradeRoutes().updateResources(game);

        for (auto &city : game.getCities()) {
            city.updateHealth(game);
            city.updateHappiness(game);
        }
        for (auto &player : game.getPlayers()) {
            player.onLoaded(game);
        }

        absl::flat_hash_map<uint32_t, PlayerId> slotIDToPlayerID;
        for (auto &[slotID, playerID] : packet.slotidtoplayerid()) {
            slotIDToPlayerID[slotID] = playerIDs.get(playerID);
        }

        return SaveData {
            .game = std::move(game),
            .slotIDToPlayerID = std::move(slotIDToPlayerID),
        };
    }
}
