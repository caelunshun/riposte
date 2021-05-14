//
// Created by Caelum van Ispelen on 5/12/21.
//

#include <unordered_set>
#include "player.h"
#include "game.h"

namespace rip {
    Player::Player(std::string username, std::shared_ptr<CivKind> civ, uint32_t mapWidth, uint32_t mapHeight) : username(std::move(username)), visibilityMap(mapWidth, mapHeight), civ(civ) {

    }

    void Player::setID(PlayerId id) {
        this->id = id;
    }

    PlayerId Player::getID() const {
        return id;
    }

    const std::string &Player::getUsername() const {
        return username;
    }

    const std::vector<CityId> &Player::getCities() const {
        return cities;
    }

    const VisibilityMap &Player::getVisibilityMap() const {
        return visibilityMap;
    }

    void Player::registerCity(CityId id) {
        cities.push_back(id);
    }

    void Player::removeCity(CityId id) {
        cities.erase(std::remove(cities.begin(), cities.end(), id), cities.end());
    }

    std::string Player::getNextCityName(const Game &game) {
        std::unordered_set<std::string> usedNames;
        for (const auto cityID : cities) {
            const auto &city = game.getCity(cityID);
            usedNames.emplace(city.getName());
        }

        int numNews = 0;
        while (true) {
            for (const auto &name : civ->cities) {
                std::string prefix;
                for (int i = 0; i < numNews; i++) {
                    prefix += "New ";
                }
                auto prefixedName = prefix + name;
                if (usedNames.find(prefixedName) == usedNames.end()) {
                    return prefixedName;
                }

                ++numNews;
            }
        }
    }

    CityId Player::createCity(glm::uvec2 pos, Game &game) {
        auto name = getNextCityName(game);
        City city(pos, std::move(name), id);
        auto cityID = game.addCity(std::move(city));
        registerCity(cityID);
        return cityID;
    }

    void Player::recomputeVisibility(const Game &game) {
        // Change Visible => Fogged
        for (int x = 0; x < game.getMapWidth(); x++) {
            for (int y = 0; y < game.getMapWidth(); y++) {
                glm::uvec2 pos(x, y);
                if (visibilityMap[pos] == Visibility::Visible) {
                    visibilityMap[pos] = Visibility::Fogged;
                }
            }
        }

        std::vector<glm::uvec2> sightPositions;

        for (const auto cityID : cities) {
            const auto &city = game.getCity(cityID);
            sightPositions.push_back(city.getPos());
        }

        for (const auto &unit : game.getUnits()) {
            sightPositions.push_back(unit.getPos());
        }

        for (const auto sightPos : sightPositions) {
            for (int dx = -2; dx <= 2; dx++) {
                for (int dy = -2; dy <= 2; dy++) {
                    auto p = glm::ivec2(sightPos) + glm::ivec2(dx, dy);
                    if (p.x < 0 || p.y < 0 || p.x >= game.getMapWidth() || p.y >= game.getMapHeight()) {
                        continue;
                    }
                    glm::uvec2 pos(p);
                    visibilityMap[pos] = Visibility::Visible;
                }
            }
        }
    }

    bool Player::isDead() const {
        return cities.empty();
    }
}
