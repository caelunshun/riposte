//
// Created by Caelum van Ispelen on 6/20/21.
//

#include "server.h"
#include "mapgen.h"

namespace rip {
    void Connection::joinGame(Game &game) {

    }

    void Connection::update(Game &game) {

    }

    Server::Server(std::shared_ptr<Registry> registry, std::shared_ptr<TechTree> techTree)
        : game(MapGenerator().generate(64, 64, registry, techTree)) {
    }

    void Server::addConnection(std::unique_ptr<Bridge> bridge) {
        connections.emplace_back(std::move(bridge));
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
