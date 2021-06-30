//
// Created by Caelum van Ispelen on 6/20/21.
//

#ifndef RIPOSTE_SERVER_H
#define RIPOSTE_SERVER_H

#include <riposte.pb.h>

#include "bridge.h"
#include "game.h"
#include "tech.h"

namespace rip {
    // Represents a connection to the server from a client.
    class Connection {
        // The bridge used to transfer packets.
        std::unique_ptr<Bridge> bridge;
        // ID of the player using this connection.
        PlayerId playerID;

        uint32_t currentRequestID = 0;

    public:
        // Whether the player has ended their current turn.
        bool endedTurn = false;

        Connection(std::unique_ptr<Bridge> bridge, PlayerId playerID) : bridge(std::move(bridge)), playerID(playerID) {}

        template<typename T>
        void send(T &packet) {
            std::string data;
            packet.SerializeToString(&data);
            bridge->sendPacket(std::move(data));
        }

        void sendGameData(Game &game);

        void handleClientInfo(Game &game, const ClientInfo &packet);
        void handleComputePath(Game &game, const ComputePath &packet);
        void handleMoveUnits(Game &game, const MoveUnits &packet);
        void handleGetBuildTasks(Game &game, const GetBuildTasks &packet);
        void handleSetBuildTask(Game &game, const SetCityBuildTask &packet);
        void handleSetResearch(Game &game, const SetResearch &packet);
        void handleGetPossibleTechs(Game &game, const GetPossibleTechs &packet);
        void handleSetEconomySettings(Game &game, const SetEconomySettings &packet);
        void handleDoUnitAction(Game &game, const DoUnitAction &packet);
        void handlePacket(Game &game, AnyClient &packet);

        void update(Game &game);
    };

    // The Riposte game server.
    //
    // Wraps a Game and handles connections by sending/handling packets.
    class Server {
        std::vector<Connection> connections;

    public:
        Game game;

        Server(std::shared_ptr<Registry> registry, std::shared_ptr<TechTree> techTree);

        void addConnection(std::unique_ptr<Bridge> bridge);

        void broadcastUnitUpdate(Unit &unit);
        void broadcastCityUpdate(City &city);
        void broadcastUnitDeath(UnitId unitID);

        void run();
    };
}

#endif //RIPOSTE_SERVER_H
