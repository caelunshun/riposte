//
// Created by Caelum van Ispelen on 5/14/21.
//

#ifndef RIPOSTE_HUD_H
#define RIPOSTE_HUD_H

#include <nuklear.h>
#include <nanovg.h>
#include <optional>
#include <glm/vec2.hpp>
#include "ids.h"

namespace rip {
    class Game;

    // Renders the UI overlay during the game.
    // Also handles certain interactions.
    class Hud {
        NVGcontext *vg;
        nk_context *nk;

        std::optional<UnitId> selectedUnit;

        void paintSelectedUnit(Game &game);

    public:
        Hud(NVGcontext *vg, nk_context *nk);

        // Renders the UI and handles input.
        void update(Game &game);

        void handleClick(Game &game, glm::vec2 pos);
    };
}

#endif //RIPOSTE_HUD_H
