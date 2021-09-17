//
// Created by Caelum van Ispelen on 9/10/2021.
//

#include "grid.h"
#include "terrain.h"
#include "../registry.h"

#include <optional>
#include <memory>

namespace rip::mapgen {
    typedef std::optional<std::shared_ptr<Resource>> ResourceTile;

    // Responsible for distributing resources across the map.
    class ResourceGenerator {
    public:
        virtual ~ResourceGenerator() = default;

        virtual Grid<ResourceTile> distributeResources(Rng &rng, const Registry &registry, const Grid<Tile> &tileGrid, const std::vector<glm::uvec2> &startingLocations) = 0;
    };

    // The default resource generator, which uniformly distributes
    // resources using a Poisson disc sampling algorithm.
    class BalancedResourceGenerator : public ResourceGenerator {
    public:
        virtual Grid<ResourceTile> distributeResources(Rng &rng, const Registry &registry, const Grid<Tile> &tileGrid, const std::vector<glm::uvec2> &startingLocations) override;
    };
}
