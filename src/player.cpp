//
// Created by Caelum van Ispelen on 5/12/21.
//

#include "player.h"
#include "game.h"

namespace rip {
    Player::Player(std::string username, uint32_t mapWidth, uint32_t mapHeight) : username(std::move(username)), visibilityMap(mapWidth, mapHeight) {

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

    std::string Player::getNextCityName() {

    }

    CityId Player::createCity(glm::uvec2 pos, Game &game) {
        auto name = getNextCityName();
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

        for (const auto cityID : cities) {
            const auto &city = game.getCity(cityID);

            for (int dx = -2; dx <= 2; dx++) {
                for (int dy = -2; dy <= 2; dy++) {
                    glm::uvec2 pos(glm::ivec2(city.getPos()) + glm::ivec2(dx, dy));
                    visibilityMap[pos] = Visibility::Visible;
                }
            }
        }
    }

    bool Player::isDead() const {
        return cities.empty();
    }
}
