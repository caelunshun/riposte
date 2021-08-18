//
// Created by Caelum van Ispelen on 8/17/21.
//

#ifndef RIPOSTE_SAVELOAD_H
#define RIPOSTE_SAVELOAD_H

#include <string>
#include <vector>
#include <absl/container/flat_hash_map.h>

#include "game.h"

namespace rip {
    struct SaveFile {
        std::string category;
        std::string name;
        std::string path;
        uint32_t turn;
    };

    std::vector<SaveFile> getAllSaves(const std::string &category);

    std::string getSavePath(const std::string &category, const std::string &name, uint32_t turn);

    struct SaveData {
        Game game;
    };

    // Handles conversion of serialized IDs back to internal slotmap IDs.
    class IdConverter {
        absl::flat_hash_map<uint32_t, ID> mapping;
        slot_map<uint16_t> idAllocator;

    public:
        ID get(uint32_t serialized) const {
            return mapping.at(serialized);
        }

        ID insert(uint32_t serialized) {
            const auto id = idAllocator.insert(0);
            mapping[serialized] = id;
            return id;
        }
    };

    std::string serializeGameToSave(Game &game, std::string name);
    SaveData loadGameFromSave(Server *server, const SaveFile &file, std::shared_ptr<Registry> registry, std::shared_ptr<TechTree> techTree);
}

#endif //RIPOSTE_SAVELOAD_H
