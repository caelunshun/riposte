//
// Created by Caelum van Ispelen on 6/20/21.
//

#include "server.h"
#include "mapgen.h"
#include "player.h"
#include "tile.h"
#include "culture.h"
#include <riposte.pb.h>

#define SEND(packet, anyservername)  { AnyServer _anyServer; _anyServer.mutable_##anyservername()->CopyFrom(packet); send(std::move(_anyServer)); }

namespace rip {
    void setPlayerInfo(const Player &player, PlayerInfo &playerInfo) {
        playerInfo.set_username(player.getUsername());
        playerInfo.set_civid(player.getCiv().id);
        playerInfo.set_leadername(player.getLeader().name);
        playerInfo.set_score(player.getScore());
        playerInfo.set_id(player.getID().second);
        playerInfo.set_isadmin(false); // TODO: permissions
    }

    UpdateGlobalData getUpdateGlobalDataPacket(Game &game) {
        UpdateGlobalData packet;
        packet.set_turn(game.getTurn());
        packet.set_era(static_cast<::Era>(static_cast<int>(game.getEra())));

        for (const auto &player : game.getPlayers()) {
            auto *protoPlayer = packet.add_players();
            setPlayerInfo(player, *protoPlayer);
        }

        return packet;
    }

    void setTile(const Game &game, PlayerId player, glm::uvec2 pos, const Tile &tile, ::Tile &protoTile) {
        protoTile.set_terrain(static_cast<::Terrain>(static_cast<int>(tile.getTerrain())));
        protoTile.set_forested(tile.isForested());
        protoTile.set_hilled(tile.isHilled());

        const auto yield = tile.getYield(game, pos, player);
        auto *protoYield = protoTile.mutable_yield();
        protoYield->set_commerce(yield.commerce);
        protoYield->set_food(yield.food);
        protoYield->set_hammers(yield.hammers);

        const auto owner = game.getCultureMap().getTileOwner(pos);
        if (owner.has_value()) {
            protoTile.set_ownerid(owner->second);
        }
        protoTile.set_hasowner(owner.has_value());

        for (const auto &improvement : tile.getImprovements()) {
            auto *protoImprovement = protoTile.add_improvements();
            protoImprovement->set_id(improvement->getName());

            auto *cottage = dynamic_cast<Cottage*>(&*improvement);
            if (cottage) {
                protoImprovement->set_cottagelevel(cottage->getLevelName());
            }
        }
    }

    UpdateMap getUpdateMapPacket(Game &game, PlayerId player) {
        UpdateMap packet;
        packet.set_width(game.getMapWidth());
        packet.set_height(game.getMapHeight());

        for (int x = 0; x < game.getMapWidth(); x++) {
            for (int y = 0; y < game.getMapHeight(); y++) {
                glm::uvec2 pos(x, y);
                const auto &tile = game.getTile(pos);
                auto protoTile = packet.add_tiles();
                setTile(game, player, pos, tile, *protoTile);
            }
        }

        return packet;
    }

    void Connection::joinGame(Game &game) {
        SEND(getUpdateGlobalDataPacket(game), updateglobaldata);
        SEND(getUpdateMapPacket(game, playerID), updatemap);
    }

    void Connection::update(Game &game) {

    }

    Server::Server(std::shared_ptr<Registry> registry, std::shared_ptr<TechTree> techTree)
        : game(MapGenerator().generate(64, 64, registry, techTree)) {
    }

    void Server::addConnection(std::unique_ptr<Bridge> bridge) {
        connections.emplace_back(std::move(bridge), game.getThePlayerID()); // TODO: multiplayer
        connections[connections.size() - 1].joinGame(game);
    }

    void Server::run() {
        while (!connections.empty()) {
            for (auto &connection : connections) {
                connection.update(game);
            }
        }
    }
}
