//
// Created by Caelum van Ispelen on 6/20/21.
//

#ifndef RIPOSTE_SERVER_H
#define RIPOSTE_SERVER_H

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

        template<typename T>
        void send(T packet) {
            std::string data;
            packet.SerializeToString(&data);
            bridge->sendPacket(std::move(data));
        }

    public:
        Connection(std::unique_ptr<Bridge> bridge, PlayerId playerID) : bridge(std::move(bridge)), playerID(playerID) {}

        void sendGameData(Game &game);

        void update(Game &game);
    };

    // The Riposte game server.
    //
    // Wraps a Game and handles connections by sending/handling packets.
    class Server {
        Game game;

        std::vector<Connection> connections;

    public:
        Server(std::shared_ptr<Registry> registry, std::shared_ptr<TechTree> techTree);

        void addConnection(std::unique_ptr<Bridge> bridge);

        void run();
    };
}

#endif //RIPOSTE_SERVER_H
