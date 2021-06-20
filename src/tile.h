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
#include "ids.h"

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
        virtual void paint(const Game &game, glm::uvec2 pos, NVGcontext *vg, const Assets &assets) = 0;

        virtual int getNumBuildTurns() const = 0;

        // Called each turn the improvement is worked by a city.
        virtual void onWorked(Game &game, City &workedBy) {}

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

        void paint(const Game &game, glm::uvec2 pos, NVGcontext *vg, const Assets &assets) override;
    };

    enum class CottageLevel {
        Cottage = 1,
        Hamlet = 2,
        Village = 3,
        Town = 4,
    };

    class Cottage : public Improvement {
        CottageLevel level = CottageLevel::Cottage;
        int turnsUntilGrowth = 10;

    public:
        explicit Cottage(glm::uvec2 pos) : Improvement(pos) {}

        bool isCompatible(const Tile &tile) const override;

        Yield getYieldContribution(const Game &game) const override;

        int getNumBuildTurns() const override;

        std::string getName() const override;

        void onWorked(Game &game, City &workedBy) override;

        CottageLevel getLevel() const;
        const char *getLevelName() const;

        int getTurnsUntilGrowth() const;

        void paint(const Game &game, glm::uvec2 pos, NVGcontext *vg, const Assets &assets) override;
    };

    class Farm : public Improvement {
    public:
        explicit Farm(glm::uvec2 pos) : Improvement(pos) {}

        bool isCompatible(const Tile &tile) const override;

        Yield getYieldContribution(const Game &game) const override;

        int getNumBuildTurns() const override;

        std::string getName() const override;

        void paint(const Game &game, glm::uvec2 pos, NVGcontext *vg, const Assets &assets) override;
    };

    class Pasture : public Improvement {
    public:
        explicit Pasture(glm::uvec2 pos) : Improvement(pos) {}

        bool isCompatible(const Tile &tile) const override;

        Yield getYieldContribution(const Game &game) const override;

        int getNumBuildTurns() const override;

        std::string getName() const override;

        void paint(const Game &game, glm::uvec2 pos, NVGcontext *vg, const Assets &assets) override;
    };

    class Road : public Improvement {
    public:
        explicit Road(glm::uvec2 pos) : Improvement(pos) {}

        bool isCompatible(const Tile &tile) const override;

        Yield getYieldContribution(const Game &game) const override;

        int getNumBuildTurns() const override;

        std::string getName() const override;

        void paint(const Game &game, glm::uvec2 pos, NVGcontext *vg, const Assets &assets) override;
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
        bool hilled = false;
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

        bool isHilled() const;

        void setHilled(bool hilled);

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

        float getMovementCost() const;

        Yield getYield(const Game &game, glm::uvec2 pos, PlayerId player) const;

        const std::vector<std::unique_ptr<Improvement>> &getImprovements() const;

        bool addImprovement(std::unique_ptr<Improvement> improvement);
        bool hasImprovement(const std::string &name) const;
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

        bool hasNonRoadImprovements() const {
            for (const auto &improvement : improvements) {
                if (improvement->getName() != "Road") {
                    return true;
                }
            }
            return false;
        }

        bool hasImproveableResource(const std::string &improvement) const {
            if (resource.has_value()) {
                return (*resource)->improvement == improvement;
            } else {
                return false;
            }
        }

        std::vector<std::unique_ptr<Improvement>> getPossibleImprovements(Game &game, glm::uvec2 pos) const;
    };


}


#endif //RIPOSTE_TILE_H
