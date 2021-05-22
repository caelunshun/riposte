//
// Created by Caelum van Ispelen on 5/21/21.
//

#include <iostream>
#include "trade.h"
#include "game.h"
#include "city.h"

namespace rip {
    void TradeRoute::mount(TradeRouteId id) {
        this->id = id;
    }

    bool TradeRoute::containsTile(glm::uvec2 pos) const {
        return tiles.contains(pos);
    }

    void TradeRoute::addTile(glm::uvec2 pos, std::optional<CityId> nodeCity) {
        tiles.insert(pos);
        if (nodeCity.has_value()) {
            visitedCities.insert(*nodeCity);
        }
    }

    void TradeRoute::removeTile(glm::uvec2 pos) {
        tiles.erase(pos);
    }

    void TradeRoute::merge(const TradeRoute &other) {
        for (const auto pos : other.getTiles()) {
            addTile(pos, {});
        }
        for (const auto city : other.getVisitedCities()) {
            visitedCities.insert(city);
        }
    }

    const absl::flat_hash_set<glm::uvec2, PosHash> &TradeRoute::getTiles() const {
        return tiles;
    }

    const absl::flat_hash_set<CityId> &TradeRoute::getVisitedCities() const {
        return visitedCities;
    }

    void TradeRoutes::addNode(const Game &game, glm::uvec2 pos) {
        absl::flat_hash_set<TradeRouteId> adjacentRoutes;
        for (const auto neighbor : getNeighbors(pos)) {
            auto neighborRoute = getRouteForNode(neighbor);
            if (neighborRoute.has_value()) {
                adjacentRoutes.insert(*neighborRoute);
            }
        }

        if (adjacentRoutes.empty()) {
            auto newRoute = createRoute();
            addNodeToRoute(game, newRoute, pos);
        } else if (adjacentRoutes.size() == 1) {
            addNodeToRoute(game, *adjacentRoutes.begin(), pos);
        } else {
            // Merge routes into routes[0].
            std::vector<TradeRouteId> routes;
            for (const auto route : adjacentRoutes) routes.push_back(route);
            while (routes.size() > 1) {
                auto first = routes[routes.size() - 1];
                auto second = routes[routes.size() - 2];
                mergeRoutes(second, first);
                routes.erase(routes.begin() + routes.size() - 1);
            }
            addNodeToRoute(game, routes[0], pos);
        }
    }

    void TradeRoutes::mergeRoutes(TradeRouteId one, TradeRouteId two) {
        assert(one != two);
        getRoute(one).merge(getRoute(two));
        deleteRoute(two);
    }

    TradeRouteId TradeRoutes::createRoute() {
        return routes.insert(TradeRoute()).second;
    }

    void TradeRoutes::deleteRoute(TradeRouteId id) {
        routes.erase(id);
    }

    void TradeRoutes::addNodeToRoute(const Game &game, TradeRouteId routeID, glm::uvec2 pos) {
        const auto *city = game.getCityAtLocation(pos);
        std::optional<CityId> cityID;
        if (city) {
            cityID = city->getID();
        }
        getRoute(routeID).addTile(pos, cityID);
        routesByPos[pos] = routeID;
    }

    void TradeRoutes::deleteNodeFromRoute(TradeRouteId routeID, glm::uvec2 pos) {
        getRoute(routeID).removeTile(pos);
        routesByPos.erase(pos);
    }

    std::optional<TradeRouteId> TradeRoutes::getRouteForNode(glm::uvec2 pos) const {
        if (routesByPos.contains(pos)) {
            return routesByPos.at(pos);
        } else {
            return {};
        }
    }

    void TradeRoutes::onCityCreated(const Game &game, const City &city) {
        addNode(game, city.getPos());
    }

    void TradeRoutes::onRoadBuilt(const Game &game, glm::uvec2 pos) {
        addNode(game, pos);
    }

    void TradeRoutes::updateResources(Game &game) {
        for (const auto &route : routes) {
            absl::flat_hash_set<std::shared_ptr<Resource>, ResourceHash> accessibleResources;
            for (const auto pos : route.getTiles()) {
                const auto &tile = game.getTile(pos);
                if (tile.hasResource()) {
                    const auto &resource = *tile.getResource();
                    if (tile.hasImprovement(resource->improvement)) {
                        accessibleResources.insert(resource);
                    }
                }
            }
            for (const auto cityID : route.getVisitedCities()) {
                auto &city = game.getCity(cityID);
                city.clearResources();
                for (const auto &resource : accessibleResources) {
                    city.addResource(resource);
                }
            }
        }
    }

    TradeRoute &TradeRoutes::getRoute(TradeRouteId id) {
        return routes.id_value(id);
    }
}
