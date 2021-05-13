//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_PLAYER_H
#define RIPOSTE_PLAYER_H

#include <rea.h>
#include <string>
#include <vector>
#include "registry.h"

namespace rip {
    class Game;

    class Player;

    typedef rea::versioned_slot_map<Player>::id_type PlayerId;

    enum Visibility : uint8_t {
        // The tile is completely hidden (black).
        Hidden,
        // The tile is visible but under fog of war (dimmed, units not visible)
        Fogged,
        // The tile is fully visible.
        Visible,
    };

    /**
     * Stores an enum for each tile on the map.
     * The bitflag determines whether the tiles are visible.
     */
     class VisibilityMap {
        std::vector<Visibility> map;
        uint32_t mapWidth, mapHeight;

     public:
         VisibilityMap(uint32_t mapWidth, uint32_t mapHeight) : mapWidth(mapWidth), mapHeight(mapHeight) {}

         Visibility operator[](glm::uvec2 pos) const {
             return map[pos.x + mapWidth * pos.y];
         }

         Visibility &operator[](glm::uvec2 pos) {
             return map[pos.x + mapWidth * pos.y];
         }

         void clear() {
             for (auto &vis : map) {
                 vis = Visibility::Hidden;
             }
         }
     };

    /**
     * A player is an instantiation of a civilization within a game.
     */
     class Player {
         // The player's ID in the Game::players slotmap.
         PlayerId id;
         // The player's name.
         std::string username;
         // Cities belonging to the player.
         std::vector<CityId> cities;
         // What tiles the player can see.
         VisibilityMap visibilityMap;
         // The player's civilization.
         std::shared_ptr<CivKind> civ;

         std::string getNextCityName();

     public:
         Player(std::string username, uint32_t mapWidth, uint32_t mapHeight);

         void setID(PlayerId id);

         PlayerId getID() const;
         const std::string &getUsername() const;
         const std::vector<CityId> &getCities() const;
         const VisibilityMap &getVisibilityMap() const;

         void registerCity(CityId id);
         void removeCity(CityId);

         // Creates a City.
         CityId createCity(glm::uvec2 pos, Game &game);

         void recomputeVisibility(const Game &game);

         bool isDead() const;
     };
}

#endif //RIPOSTE_PLAYER_H
