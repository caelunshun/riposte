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
    class Image;
    class Game;
    class Stack;

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
        std::shared_ptr<Assets> assets;

        std::optional<glm::vec2> clickPos;

        std::optional<StackId> selectedStack;
        std::vector<UnitId> selectedUnits;

        Path selectedUnitPath;
        std::optional<glm::uvec2> selectedUnitPathError;
        bool isSelectingPath = false;

        std::deque<HudMessage> messages;

        CityId lastCityBuildPrompt;

        std::shared_ptr<Image> goldIcon;
        std::shared_ptr<Image> beakerIcon;

        void paintSelectedUnit(Game &game);
        void paintMainHud(Game &game);
        void paintMessages(Game &game);
        void paintUnitUI(Game &game);
        void paintCityBuildPrompt(Game &game, CityId cityID);
        void paintPath(Game &game, const Stack &stack, glm::uvec2 start, const Path &path);

        void paintStackSelectionBar(Game &game);

        void paintResearchBar(Game &game);

        void paintScoreHud(Game &game);

        bool shouldShowTechPrompt(const Game &game) const;
        void paintTechPrompt(Game &game);

        void trySetSelectedPath(Game &game, glm::uvec2 from, glm::uvec2 to);

        std::optional<CityId> getCityBuildPrompt(const Game &game) const;

        void paintTopLeftHud(Game &game);

        bool wasRectClicked(glm::vec2 pos, glm::vec2 size) const;

    public:
        Hud(std::shared_ptr<Assets> assets, NVGcontext *vg, nk_context *nk);

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
