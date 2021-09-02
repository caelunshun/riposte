//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_MAPGEN_H
#define RIPOSTE_MAPGEN_H

#include <riposte.pb.h>
#include <mapgen.pb.h>
#include <map>
#include "game.h"
#include "rng.h"

namespace rip {
    class TechTree;
    class Server;

    class MapGenerator {
        Rng rng;
    public:
        std::pair<Game, std::map<uint32_t, PlayerId>> generate(const std::vector<proto::LobbySlot> &playerSlots, mapgen::MapgenSettings settings, std::shared_ptr<Registry> registry, const std::shared_ptr<TechTree> &techTree, Server *server);
    };
}

#endif //RIPOSTE_MAPGEN_H
