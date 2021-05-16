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
#include "path.h"

namespace rip {
    class Game;

    struct HudMessage {
        std::string text;
        float disappearTime;

        HudMessage(std::string text, float disappearTime) : text(text), disappearTime(disappearTime) {}
    };

    // Renders the UI overlay during the game.
    // Also handles certain interactions.
    class Hud {
        NVGcontext *vg;
        nk_context *nk;

        std::optional<UnitId> selectedUnit;
        Path selectedUnitPath;
        std::optional<glm::uvec2> selectedUnitPathError;
        bool isSelectingPath = false;

        std::deque<HudMessage> messages;

        CityId lastCityBuildPrompt;

        void paintSelectedUnit(Game &game);
        void paintMainHud(Game &game);
        void paintMessages(Game &game);
        void paintUnitUI(Game &game);
        void paintCityBuildPrompt(Game &game, CityId cityID);
        void paintPath(Game &game, glm::uvec2 start, const Path &path);

        void trySetSelectedPath(Game &game, glm::uvec2 from, glm::uvec2 to);

        std::optional<CityId> getCityBuildPrompt(const Game &game) const;

    public:
        Hud(NVGcontext *vg, nk_context *nk);

        // Renders the UI and handles input.
        void update(Game &game);

        void handleClick(Game &game, MouseEvent event);
        void handleKey(Game &game, int key);

        void updateSelectedUnit(Game &game);

        void pushMessage(std::string message);

        bool hasFocus(const Game &game) const;
    };
}

#endif //RIPOSTE_HUD_H
