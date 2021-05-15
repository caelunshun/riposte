//
// Created by Caelum van Ispelen on 5/14/21.
//

#ifndef RIPOSTE_HUD_H
#define RIPOSTE_HUD_H

#include <nuklear.h>
#include <nanovg.h>
#include <optional>
#include <deque>
#include <glm/vec2.hpp>
#include "ids.h"
#include "ui.h"

namespace rip {
    class Game;

    struct HudMessage {
        std::string text;
        float disappearTime;

        HudMessage(std::string text, float disappearTime) : text(std::move(text)), disappearTime(disappearTime) {}
    };

    // Renders the UI overlay during the game.
    // Also handles certain interactions.
    class Hud {
        NVGcontext *vg;
        nk_context *nk;

        std::optional<UnitId> selectedUnit;

        std::deque<HudMessage> messages;

        void paintSelectedUnit(Game &game);
        void paintMainHud(Game &game);
        void paintMessages(Game &game);

    public:
        Hud(NVGcontext *vg, nk_context *nk);

        // Renders the UI and handles input.
        void update(Game &game);

        void handleClick(Game &game, MouseEvent event);

        void updateSelectedUnit(Game &game);

        void pushMessage(std::string message);
    };
}

#endif //RIPOSTE_HUD_H
