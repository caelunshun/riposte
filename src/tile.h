//
// Created by Caelum van Ispelen on 5/11/21.
//

#ifndef RIPOSTE_TILE_H
#define RIPOSTE_TILE_H

#include <string>
#include <vector>
#include <memory>
#include <glm/vec2.hpp>
#include "assets.h"
#include "registry.h"

struct NVGcontext;

namespace rip {
    struct Yield;

    class Game;

    class Tile;

    // A tile improvement (usually created by a worker)
    class Improvement {
    protected:
        glm::uvec2 pos;
        explicit Improvement(glm::uvec2 pos) : pos(pos) {}
    public:
        // Determines whether the improvement is compatible with the given
        // tile;
        virtual bool isCompatible(const Tile &tile) const = 0;

        // Gets the yield this improvement contributes to a tile.
        virtual Yield getYieldContribution(const Game &game) const = 0;

        // Draws the improvement.
        //
        // `offset` is the upper-left-hand corner of the improvement's tile.
        virtual void paint(NVGcontext *vg, const Assets &assets, glm::vec2 offset) = 0;

        virtual int getNumBuildTurns() const = 0;

        virtual std::string getName() const = 0;

        virtual ~Improvement() = default;
    };

    class Mine : public Improvement {
    public:
        explicit Mine(glm::uvec2 pos) : Improvement(pos) {}

        bool isCompatible(const Tile &tile) const override;

        Yield getYieldContribution(const Game &game) const override;

        int getNumBuildTurns() const override;

        std::string getName() const override;

        void paint(NVGcontext *vg, const Assets &assets, glm::vec2 offset) override;
    };

    class Cottage : public Improvement {
    public:
        explicit Cottage(glm::uvec2 pos) : Improvement(pos) {}

        bool isCompatible(const Tile &tile) const override;

        Yield getYieldContribution(const Game &game) const override;

        int getNumBuildTurns() const override;

        std::string getName() const override;

        void paint(NVGcontext *vg, const Assets &assets, glm::vec2 offset) override;
    };

    class Farm : public Improvement {
    public:
        explicit Farm(glm::uvec2 pos) : Improvement(pos) {}

        bool isCompatible(const Tile &tile) const override;

        Yield getYieldContribution(const Game &game) const override;

        int getNumBuildTurns() const override;

        std::string getName() const override;

        void paint(NVGcontext *vg, const Assets &assets, glm::vec2 offset) override;
    };

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
        std::vector<std::unique_ptr<Improvement>> improvements;
        std::optional<std::shared_ptr<Resource>> resource;

    public:
        Tile(Terrain terrain) : terrain(terrain) {}
        Tile() : terrain(Terrain::Grassland) {}

        Tile(const Tile &other) = delete;

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

        const std::vector<std::unique_ptr<Improvement>> &getImprovements() const;

        bool addImprovement(std::unique_ptr<Improvement> improvement);

        void clearImprovements();

        template<class T>
        bool hasImprovement() const {
            for (const auto &improvement : getImprovements()) {
                if (dynamic_cast<const T*>(&*improvement)) {
                    return true;
                }
            }
            return false;
        }

        const std::optional<std::shared_ptr<Resource>> &getResource() const {
            return resource;
        }

        bool hasResource() const {
            return resource.has_value();
        }

        void setResource(std::shared_ptr<Resource> resource) {
            this->resource = std::move(resource);
        }
    };


}


#endif //RIPOSTE_TILE_H
