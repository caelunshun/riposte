//
// Created by Caelum van Ispelen on 5/11/21.
//

#ifndef RIPOSTE_GAME_H
#define RIPOSTE_GAME_H

#include <vector>
#include <glm/ext/vector_uint2.hpp>
#include <rea.h>
#include <optional>

#include "tile.h"
#include "cursor.h"
#include "view.h"
#include "city.h"
#include "player.h"
#include "unit.h"
#include "registry.h"
#include "ids.h"

namespace rip {
    class Game {
        std::vector<Tile> theMap;
        uint32_t mapWidth;
        uint32_t mapHeight;

        rea::versioned_slot_map<City> cities;
        rea::versioned_slot_map<Player> players;
        rea::versioned_slot_map<Unit> units;

        // The human player.
        PlayerId thePlayer;

        Cursor cursor;
        View view;

        std::shared_ptr<Registry> registry;

        float dt = 0;
        float lastFrameTime = 0;

        int turn = 0;

        size_t getMapIndex(glm::uvec2 pos) const {
            return static_cast<size_t>(pos.x) + static_cast<size_t>(pos.y) * static_cast<size_t>(mapWidth);
        }

    public:
        Game(uint32_t mapWidth, uint32_t mapHeight, std::shared_ptr<Registry> registry)
        : theMap(static_cast<size_t>(mapWidth) * mapHeight),
        mapWidth(mapWidth),
        mapHeight(mapHeight),
        registry(registry),
        cursor() {

        }

        // Advances to the next turn, updating all necessary game state.
        void advanceTurn();

        // Gets the next unit the player should be prompted to move on this turn.
        // If this returns an empty, then the turn should end.
        std::optional<UnitId> getNextUnitToMove() const;

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

        glm::vec2 getScreenOffset(glm::uvec2 tile) const {
            return glm::vec2(tile) * 100.0f - getMapOrigin();
        }

        glm::uvec2 getPosFromScreenOffset(glm::vec2 offset) const {
            auto translated = offset + getMapOrigin();
            auto scaled = translated / 100.0f;
            return glm::uvec2(static_cast<uint32_t>(floor(scaled.x)), static_cast<uint32_t>(floor(scaled.y)));
        }

        void tick(GLFWwindow *window) {
            dt = glfwGetTime() - lastFrameTime;
            lastFrameTime = glfwGetTime();

            cursor.tick(window);
            view.tick(dt, cursor);
        }

        const rea::versioned_slot_map<City> &getCities() const {
            return cities;
        }

        rea::versioned_slot_map<City> &getCities() {
            return cities;
        }

        CityId addCity(City city) {
            return cities.insert(std::move(city)).second;
        }

        City *getCityAtLocation(glm::uvec2 location) {
            for (auto &city : cities) {
                if (city.getPos() == location) {
                    return &city;
                }
            }
            return nullptr;
        }

        City &getCity(CityId id) {
            return cities.id_value(id);
        }

        const City &getCity(CityId id) const {
            return cities.id_value(id);
        }

        Player &getPlayer(PlayerId id) {
            return players.id_value(id);
        }

        PlayerId getThePlayerID() const {
            return thePlayer;
        }

        Player &getThePlayer() {
            return getPlayer(thePlayer);
        }

        size_t getNumPlayers() const {
            return players.size();
        }

        void setThePlayerID(PlayerId id) {
            thePlayer = id;
        }

        PlayerId addPlayer(Player player) {
            return players.insert(std::move(player)).second;
        }

        rea::versioned_slot_map<Player> &getPlayers() {
            return players;
        }

        const Registry &getRegistry() const {
            return *registry;
        }

        UnitId addUnit(Unit unit) {
            auto id = units.insert(std::move(unit)).second;
            auto &u = getUnit(id);
            u.setID(id);
            return id;
        }

        const Unit &getUnit(UnitId id) const {
            return units.id_value(id);
        }

        Unit &getUnit(UnitId id) {
            return units.id_value(id);
        }

        Unit *getUnitAtPosition(glm::uvec2 location) {
            for (auto &unit : units) {
                if (unit.getPos() == location) {
                    return &unit;
                }
            }
            return nullptr;
        }

        rea::versioned_slot_map<Unit> &getUnits() {
            return units;
        }

        const rea::versioned_slot_map<Unit> &getUnits() const {
            return units;
        }

        int getTurn() const {
            return turn;
        }
    };
}

#endif //RIPOSTE_GAME_H
