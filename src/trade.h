//
// Created by Caelum van Ispelen on 5/21/21.
//

#ifndef RIPOSTE_TRADE_H
#define RIPOSTE_TRADE_H

#include <absl/container/flat_hash_set.h>
#include <absl/container/flat_hash_map.h>
#include <rea.h>
#include "ripmath.h"
#include "ids.h"

namespace rip {
    class Game;
    class City;

    // A trade route is defined as the set of all tiles visited
    // by a contiguous sequence of roads or cities.
    class TradeRoute {
        // Tiles on the trade route.
        absl::flat_hash_set<glm::uvec2, PosHash> tiles;
        // Cities on the trade route.
        absl::flat_hash_set<CityId> visitedCities;

        TradeRouteId id;

    public:
        void mount(TradeRouteId id);

        bool containsTile(glm::uvec2 pos) const;
        void addTile(glm::uvec2 pos, std::optional<CityId> nodeCity);
        void removeTile(glm::uvec2 pos);

        const absl::flat_hash_set<glm::uvec2, PosHash> &getTiles() const;

        const absl::flat_hash_set<CityId> &getVisitedCities() const;

        void merge(const TradeRoute &other);
    };

    // Manages trade route information for the whole map.
    class TradeRoutes {
        rea::versioned_slot_map<TradeRoute> routes;

        // An index to quickly access the trade route for a given tile.
        absl::flat_hash_map<glm::uvec2, TradeRouteId, PosHash> routesByPos;

        // Adds a node. The effect will be one of the following:
        // * if the node is adjacent to one trade route, add it to that route
        // * if the node is adjacent to two or more trade routes, merge the routes
        // and add this node to the product
        // * if the node is adjacent to zero trade routes, create a new one
        void addNode(const Game &game, glm::uvec2 pos);

        // Merges two trade routes together.
        // The result is stored in `one`. `two` is deleted.
        void mergeRoutes(TradeRouteId one, TradeRouteId two);

        TradeRouteId createRoute();
        void deleteRoute(TradeRouteId id);
        TradeRoute &getRoute(TradeRouteId id);

        void addNodeToRoute(const Game &game, TradeRouteId routeID, glm::uvec2 pos);
        void deleteNodeFromRoute(TradeRouteId routeID, glm::uvec2 pos);

        std::optional<TradeRouteId> getRouteForNode(glm::uvec2 pos) const;
    public:
        void onCityCreated(const Game &game, const City &city);
        void onRoadBuilt(const Game &game, glm::uvec2 pos);

        // Updates resources accessible to each city.
        void updateResources(Game &game);
    };
}

#endif //RIPOSTE_TRADE_H
