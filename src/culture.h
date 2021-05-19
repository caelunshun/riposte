//
// Created by Caelum van Ispelen on 5/18/21.
//

#ifndef RIPOSTE_CULTURE_H
#define RIPOSTE_CULTURE_H

#include <absl/container/inlined_vector.h>
#include <glm/vec2.hpp>
#include <optional>
#include "ids.h"
#include "ripmath.h"

namespace rip {
    class Game;

    struct CultureValue {
        // The civ owning this culture.
        PlayerId owner;
        // The amount of accumulated culture.
        int amount;

        CultureValue(const PlayerId &owner, int amount);
    };

    // Stores the culture for a single plot or city.
    class Culture {
        absl::InlinedVector<CultureValue, 3> values;

        CultureValue &getCultureValue(PlayerId player);
        int getCultureAmount(PlayerId player) const;
    public:
        // Gets the culture value for the given civ at this tile.
        int getCultureForPlayer(PlayerId player) const;

        // Grants the given amount of culture to the given civ.
        void addCultureForPlayer(PlayerId player, int amount);

        // Removes all the culture for a given player.
        void clearCultureForPlayer(PlayerId player);

        const absl::InlinedVector<CultureValue, 3> &getValues() const;
    };


    // Manages culture for each plot.
    class CultureMap {
        int mapWidth, mapHeight;
        std::vector<Culture> tiles;
        std::vector<std::optional<PlayerId>> owners;

        std::vector<absl::InlinedVector<CityId, 2>> owningCities;

        Culture &getTileCulture(glm::uvec2 pos);

        void updateForCity(Game &game, CityId city);

    public:
        CultureMap(int mapWidth, int mapHeight);

        const Culture &getTileCulture(glm::uvec2 pos) const;

        // Updates culture values on turn end.
        void onTurnEnd(Game &game);

        void onCityCreated(Game &game, CityId city);
        void onCityDestroyed(Game &game, CityId city);

        // Gets the owner of the given tile.
        std::optional<PlayerId> getTileOwner(glm::uvec2 pos) const;
    };
}

#endif //RIPOSTE_CULTURE_H
