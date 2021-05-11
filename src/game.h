//
// Created by Caelum van Ispelen on 5/11/21.
//

#ifndef RIPOSTE_GAME_H
#define RIPOSTE_GAME_H

#include <vector>
#include <glm/ext/vector_uint2.hpp>
#include "tile.h"

namespace rip {
    class Game {
        std::vector<Tile> theMap;
        uint32_t mapWidth;
        uint32_t mapHeight;

    private:
        size_t getMapIndex(glm::uvec2 pos) const {
            return static_cast<size_t>(pos.x) + static_cast<size_t>(pos.y) * static_cast<size_t>(mapWidth);
        }

    public:
        Game(uint32_t mapWidth, uint32_t mapHeight)
        : theMap(static_cast<size_t>(mapWidth) * mapHeight),
        mapWidth(mapWidth),
        mapHeight(mapHeight) {}

        uint32_t getMapWidth() const {
            return mapWidth;
        }

        uint32_t getMapHeight() const {
            return mapHeight;
        }

        bool containsTile(glm::uvec2 pos) const {
            return (pos.x < mapWidth && pos.y < mapHeight);
        }

        Tile &getTile(glm::uvec2 pos) {
            assert(pos.x < mapWidth);
            assert(pos.y < mapHeight);
            auto index = getMapIndex(pos);
            return theMap.at(index);
        }

        const Tile &getTile(glm::uvec2 pos) const {
            auto index = getMapIndex(pos);
            return theMap.at(index);
        }
    };
}

#endif //RIPOSTE_GAME_H
