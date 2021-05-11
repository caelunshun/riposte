//
// Created by Caelum van Ispelen on 5/11/21.
//

#ifndef RIPOSTE_TILE_H
#define RIPOSTE_TILE_H

namespace rip {
    /**
     * A type of terrain.
     */
    enum Terrain {
        Grassland,
        Desert,
    };

    /**
     * A tile on the map.
     */
    class Tile {
    private:
        Terrain terrain;

    public:
        Terrain getTerrain() const {
            return terrain;
        }

        void setTerrain(Terrain terrain) {
            this->terrain = terrain;
        }

        bool canSustainCity() const {
            return (terrain != Terrain::Desert);
        }
    };


}


#endif //RIPOSTE_TILE_H
