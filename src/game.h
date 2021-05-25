//
// Created by Caelum van Ispelen on 5/11/21.
//

#ifndef RIPOSTE_GAME_H
#define RIPOSTE_GAME_H

#include <memory>
#include <glm/vec2.hpp>
#include <rea.h>
#include <GLFW/glfw3.h>

// Avoid including headers that change frequently here,
// or changing those headers will cause a recompilation of almost all source
// files.
#include "era.h"
#include "ids.h"
#include "view.h"
#include "cursor.h"

namespace rip {
    class Registry;
    class Unit;
    class Player;
    class City;
    class Tile;
    class CultureMap;
    class TradeRoutes;
    class Combat;

    class Game {
        class _impl;
        std::unique_ptr<_impl> impl;

    public:
        Game(uint32_t mapWidth, uint32_t mapHeight, std::shared_ptr<Registry> registry);

        ~Game();

        Game(Game &&other);
        Game(const Game &other) = delete;

        // Advances to the next turn, updating all necessary game state.
        void advanceTurn();

        // Gets the next unit the player should be prompted to move on this turn.
        // If this returns an empty, then the turn should end.
        std::optional<UnitId> getNextUnitToMove();

        uint32_t getMapWidth() const;
        uint32_t getMapHeight() const;
        bool containsTile(glm::uvec2 pos) const;
        Tile &getTile(glm::uvec2 pos);
        const Tile &getTile(glm::uvec2 pos) const;

        Cursor &getCursor();
        const Cursor &getCursor() const;

        View &getView();
        const View &getView() const;

        float getDeltaTime() const;

        glm::vec2 getMapOrigin() const;
        glm::vec2 getScreenOffset(glm::uvec2 tile) const;
        glm::uvec2 getPosFromScreenOffset(glm::vec2 offset) const;

        void tick(GLFWwindow *window, bool hudHasFocus);

        const rea::versioned_slot_map<City> &getCities() const;
        rea::versioned_slot_map<City> &getCities();
        CityId addCity(City city);
        City *getCityAtLocation(glm::uvec2 location);
        const City *getCityAtLocation(glm::uvec2 location) const;
        City &getCity(CityId id);
        const City &getCity(CityId id) const;

        Player &getPlayer(PlayerId id);
        const Player &getPlayer(PlayerId id) const;
        Player &getThePlayer();
        const Player &getThePlayer() const;
        PlayerId getThePlayerID() const;
        size_t getNumPlayers() const;
        void setThePlayerID(PlayerId id);
        PlayerId addPlayer(Player player);
        rea::versioned_slot_map<Player> &getPlayers();

        const Registry &getRegistry() const;

        UnitId addUnit(Unit unit);
        const Unit &getUnit(UnitId id) const;
        Unit &getUnit(UnitId id);
        Unit *getUnitAtPosition(glm::uvec2 location);
        void killUnit(UnitId id);
        // Enqueues a unit to be killed as soon as possible.
        void deferKillUnit(UnitId id);
        rea::versioned_slot_map<Unit> &getUnits();
        const rea::versioned_slot_map<Unit> &getUnits() const;

        int getTurn() const;

        void toggleCheatMode();
        bool isCheatMode() const;

        bool isTileWorked(glm::uvec2 pos) const;
        void setTileWorked(glm::uvec2 pos, bool worked);

        CultureMap &getCultureMap();
        const CultureMap &getCultureMap() const;

        TradeRoutes &getTradeRoutes();
        const TradeRoutes &getTradeRoutes() const;

        void addCombat(Combat &combat);

        Era getEra() const;
    };
}

#endif //RIPOSTE_GAME_H
