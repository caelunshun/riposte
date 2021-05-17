//
// Created by Caelum van Ispelen on 5/11/21.
//

#ifndef RIPOSTE_TILE_H
#define RIPOSTE_TILE_H

#include <string>
#include <glm/vec2.hpp>

namespace rip {
    struct Yield;

    class Game;

    /**
     * A type of terrain.
     */
    enum Terrain {
        Grassland,
        Desert,
        Ocean,
        Plains,
    };

    /**
     * A tile on the map.
     */
    class Tile {
    private:
        Terrain terrain;
        bool forested = false;

    public:
        Tile(Terrain terrain) : terrain(terrain) {}
        Tile() : terrain(Terrain::Grassland) {}

        Terrain getTerrain() const {
            return terrain;
        }

        void setTerrain(Terrain terrain) {
            this->terrain = terrain;
        }

        bool isForested() const;

        void setForested(bool forested);

        bool canSustainCity() const {
            return (terrain != Terrain::Desert);
        }

        const char *getTerrainID() const {
            switch (terrain) {
                case Grassland:
                    return "grassland";
                case Desert:
                    return "desert";
                case Plains:
                    return "plains";
                case Ocean:
                    return "ocean";
            }
        }

        int getMovementCost() const;

        Yield getYield(const Game &game, glm::uvec2 pos) const;
    };


}


#endif //RIPOSTE_TILE_H
