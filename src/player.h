//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_PLAYER_H
#define RIPOSTE_PLAYER_H

#include <rea.h>
#include <string>
#include <vector>
#include <glm/vec2.hpp>
#include "ai.h"
#include "registry.h"
#include "ids.h"
#include "tech.h"

namespace rip {
    class Game;

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
         VisibilityMap(uint32_t mapWidth, uint32_t mapHeight) : map(mapWidth * mapHeight), mapWidth(mapWidth), mapHeight(mapHeight) {
             for (int i = 0; i < mapWidth * mapHeight; i++) {
                 map[i] = Visibility::Hidden;
             }
         }

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

     struct ResearchingTech {
         std::shared_ptr<Tech> tech;
         int beakersAccumulated = 0;

         ResearchingTech(std::shared_ptr<Tech> tech);

         bool isFinished() const;
         int estimateCompletionTurns(int beakersPerTurn) const;
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

         std::optional<AI> ai;

         PlayerTechs techs;
         std::optional<ResearchingTech> researchingTech;

         int baseRevenue = 0;
         int beakerRevenue = 0;
         int goldRevenue = 0;
         int expenses = 0;
         int gold = 0;

         // Determines how much of the base revenue is converted to beakers.
         int sciencePercent = 100;

         CityId capital;

         std::string getNextCityName(const Game &game);

         void recomputeRevenue(Game &game);
         void recomputeExpenses(Game &game);
         void updateResearch(Game &game);
         void doEconomyTurn(Game &game);

     public:
         Player(std::string username, std::shared_ptr<CivKind> civ, uint32_t mapWidth, uint32_t mapHeight, const std::shared_ptr<TechTree> &techTree);

         Player(Player &&other) = default;
         Player(const Player &other) = delete;

         void setID(PlayerId id);
         void setCapital(CityId capital);

         void enableAI();

         PlayerId getID() const;
         const std::string &getUsername() const;
         const std::vector<CityId> &getCities() const;
         const VisibilityMap &getVisibilityMap() const;
         const CivKind &getCiv() const;
         CityId getCapital() const;

         void registerCity(CityId id);
         void removeCity(CityId);

         // Creates a City.
         CityId createCity(glm::uvec2 pos, Game &game);

         void recomputeVisibility(const Game &game);

         bool isDead() const;

         void onTurnEnd(Game &game);

         const PlayerTechs &getTechs() const;

         int getBaseRevenue() const;
         int getGoldRevenue() const;
         int getBeakerRevenue() const;
         int getExpenses() const;
         int getNetGold() const;

         int getGold() const;

         const std::optional<ResearchingTech> &getResearchingTech() const;
         void setResearchingTech(const std::shared_ptr<Tech> &tech);

         int getSciencePercent() const;
         void setSciencePercent(int percent, Game &game);
     };
}

#endif //RIPOSTE_PLAYER_H
