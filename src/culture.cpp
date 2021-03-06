//
// Created by Caelum van Ispelen on 5/18/21.
//

#include "culture.h"
#include "game.h"
#include "city.h"
#include "traversal.h"
#include "server.h"

namespace rip {
    CultureValue::CultureValue(const PlayerId &owner, int amount) : owner(owner), amount(amount) {}

    CultureMap::CultureMap(int mapWidth, int mapHeight) : tiles(mapWidth * mapHeight), owners(mapWidth * mapHeight),
    owningCities(mapWidth * mapHeight), mapWidth(mapWidth), mapHeight(mapHeight) {}

    Culture &CultureMap::getTileCulture(glm::uvec2 pos) {
        return tiles[pos.x + pos.y * mapWidth];
    }

    const Culture &CultureMap::getTileCulture(glm::uvec2 pos) const {
        return tiles[pos.x + pos.y * mapWidth];
    }

    void Culture::sort() {
        std::sort(values.begin(), values.end(), [](const CultureValue &a, const CultureValue &b) {
            return a.amount > b.amount;
        });
    }

    CultureValue &Culture::getCultureValue(PlayerId player) {
       for (auto &value : values) {
           if (value.owner == player) return value;
       }
       values.emplace_back(player, 0);
       return values[values.size() - 1];
    }

    int Culture::getCultureAmount(PlayerId player) const {
        for (const auto &value : values) {
            if (value.owner == player) return value.amount;
        }
        return 0;
    }

    int Culture::getCultureForPlayer(PlayerId player) const {
        return getCultureAmount(player);
    }

    void Culture::addCultureForPlayer(PlayerId player, int amount) {
        getCultureValue(player).amount += amount;
        sort();
    }

    void Culture::clearCultureForPlayer(PlayerId player) {
        auto &culture = getCultureValue(player);
        culture.amount = 0;
        sort();
    }

    const absl::InlinedVector<CultureValue, 3> &Culture::getValues() const {
        return values;
    }

    void CultureMap::updateForCity(Game &game, CityId cityID) {
        const auto &city = game.getCity(cityID);
        const auto level = city.getCultureLevel().value;

        const auto culturePerTurn = city.getCulturePerTurn();

        bool isVisibilityDirty = false;

        breadthFirstSearch(game, city.getPos(), [&] (Tile &tile, glm::uvec2 pos) {
            auto &cultureTile = getTileCulture(pos);
            cultureTile.addCultureForPlayer(city.getOwner(), culturePerTurn);

            // Add 20 * (radius - distance) culture to the plot as well.
            auto extraCulture = 20 * (level - static_cast<int>(round(dist(pos, city.getPos()))));
            if (extraCulture > 0) {
                cultureTile.addCultureForPlayer(city.getOwner(), extraCulture);
            }

            auto &touchingCities = owningCities[pos.x + pos.y * mapWidth];
            if (std::find(touchingCities.begin(), touchingCities.end(), cityID) == touchingCities.end()) {
                touchingCities.push_back(cityID);
            }

            // Update the tile owner.
            auto currentOwner = getTileOwner(pos);
            for (const auto touchingCityID : touchingCities) {
                const auto &touchingCity = game.getCity(touchingCityID);
                if (!currentOwner.has_value() || cultureTile.getCultureForPlayer(touchingCity.getOwner())
                    > cultureTile.getCultureForPlayer(*currentOwner)) {
                    currentOwner = touchingCity.getOwner();
                    isVisibilityDirty = true;
                }
            }
            owners[pos.x + pos.y * mapWidth] = currentOwner;

            game.getServer().markTileDirty(pos);
        }, [&] (Tile &tile, glm::uvec2 pos) {
            auto d = dist(pos, city.getPos());
            return static_cast<int>(round(d)) <= level;
        });

        if (isVisibilityDirty) {
            game.getPlayer(city.getOwner()).recomputeVisibility(game);
        }
    }

    void CultureMap::onTurnEnd(Game &game) {
        for (const auto &city : game.getCities()) {
            updateForCity(game, city.getID());
        }
    }

    void CultureMap::onCityCreated(Game &game, CityId city) {
        updateForCity(game, city);

        // Add 1 initial culture per adjacent tile
        for (const auto tilePos : getNeighbors(game.getCity(city).getPos())) {
            getTileCulture(tilePos).addCultureForPlayer(game.getCity(city).getOwner(), 1);
        }
    }

    void CultureMap::onCityDestroyed(Game &game, CityId cityID) {
        const auto &city = game.getCity(cityID);
        const auto level = 6;

        breadthFirstSearch(game, city.getPos(), [&] (Tile &tile, glm::uvec2 tilePos) {
            auto &touchingCities = owningCities[tilePos.x + tilePos.y * mapWidth];
            auto it = std::find(touchingCities.begin(), touchingCities.end(), cityID);
            if (it != touchingCities.end()) {
                touchingCities.erase(it);
            }

            auto &owner = owners[tilePos.x + tilePos.y * mapWidth];
            if (owner == city.getOwner() && touchingCities.empty()) {
                owner = {};
                game.getServer().markTileDirty(tilePos);
            }
        }, [&] (Tile &tile, glm::uvec2 tilePos) {
            auto d = dist(tilePos, city.getPos());
            return static_cast<int>(round(d)) <= level;
        });
    }

    std::optional<PlayerId> CultureMap::getTileOwner(glm::uvec2 pos) const {
        return owners[pos.x + pos.y * mapWidth];
    }

    void CultureMap::setCulture(glm::uvec2 tilePos, Culture culture) {
        tiles[tilePos.x + tilePos.y * mapWidth] = std::move(culture);
    }
}
