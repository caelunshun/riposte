//
// Created by Caelum van Ispelen on 5/11/21.
//

#ifndef RIPOSTE_GAME_H
#define RIPOSTE_GAME_H

#include <vector>
#include <glm/ext/vector_uint2.hpp>
#include "tile.h"
#include "cursor.h"
#include "view.h"
#include "city.h"

namespace rip {
    class Game {
        std::vector<Tile> theMap;
        uint32_t mapWidth;
        uint32_t mapHeight;

        std::vector<City> cities;

        Cursor cursor;
        View view;

        float dt = 0;
        float lastFrameTime = 0;

        size_t getMapIndex(glm::uvec2 pos) const {
            return static_cast<size_t>(pos.x) + static_cast<size_t>(pos.y) * static_cast<size_t>(mapWidth);
        }

    public:
        Game(uint32_t mapWidth, uint32_t mapHeight)
        : theMap(static_cast<size_t>(mapWidth) * mapHeight),
        mapWidth(mapWidth),
        mapHeight(mapHeight) {}

        uint32_t getMapWidth() const {
            return mapWidth;
        }

        uint32_t getMapHeight() const {
            return mapHeight;
        }

        bool containsTile(glm::uvec2 pos) const {
            return (pos.x < mapWidth && pos.y < mapHeight);
        }

        Tile &getTile(glm::uvec2 pos) {
            assert(pos.x < mapWidth);
            assert(pos.y < mapHeight);
            auto index = getMapIndex(pos);
            return theMap.at(index);
        }

        const Tile &getTile(glm::uvec2 pos) const {
            auto index = getMapIndex(pos);
            return theMap.at(index);
        }

        Cursor &getCursor() {
            return cursor;
        }

        const Cursor &getCursor() const {
            return cursor;
        }

        View &getView() {
            return view;
        }

        const View &getView() const {
            return view;
        }

        float getDeltaTime() const {
            return dt;
        }

        glm::vec2 getMapOrigin() const {
            return getView().getMapCenter() - (getCursor().getWindowSize() / 2.0f);
        }

        void tick(GLFWwindow *window) {
            dt = glfwGetTime() - lastFrameTime;
            lastFrameTime = glfwGetTime();

            cursor.tick(window);
            view.tick(dt, cursor);
        }

        const std::vector<City> &getCities() const {
            return cities;
        }

        std::vector<City> &getCities() {
            return cities;
        }

        void addCity(City city) {
            cities.push_back(std::move(city));
        }

        City *getCityAtLocation(glm::uvec2 location) {
            for (auto &city : cities) {
                if (city.getPos() == location) {
                    return &city;
                }
            }
            return nullptr;
        }
    };
}

#endif //RIPOSTE_GAME_H
