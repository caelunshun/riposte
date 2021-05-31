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

    // A window opened in the HUD.
    class Window {
    public:
        virtual ~Window() = default;

        // Paints the window with Nuklear.
        //
        // This method _must_ call nk_begin and nk_end.
        virtual void paint(Game &game, nk_context *nk) = 0;

        virtual bool shouldClose() = 0;
    };

    struct HudMessage {
        std::string text;
        float disappearTime;
        std::array<uint8_t, 3> color;

        HudMessage(std::string text, float disappearTime, std::array<uint8_t, 3> color) : text(text), disappearTime(disappearTime), color(color) {}
    };

    // Renders the UI overlay during the game.
    // Also handles certain interactions.
    class Hud {
        NVGcontext *vg;
        nk_context *nk;
        GLFWwindow *window;
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

        std::vector<std::unique_ptr<Window>> windows;

        void paintSelectedUnit(Game &game);
        void paintMainHud(Game &game);
        void paintMessages(Game &game);
        void paintGenericUnitUI(Game &game);
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

        void paintWindows(Game &game);

        bool wasRectClicked(glm::vec2 pos, glm::vec2 size) const;

    public:
        Hud(std::shared_ptr<Assets> assets, NVGcontext *vg, nk_context *nk, GLFWwindow *window);

        // Renders the UI and handles input.
        void update(Game &game);

        void handleClick(Game &game, MouseEvent event);
        void handleKey(Game &game, int key);

        void updateSelectedUnit(Game &game);

        void pushMessage(std::string message, std::array<uint8_t, 3> color);

        bool hasFocus(const Game &game) const;

        void openWindow(std::unique_ptr<Window> window);
    };
}

#endif //RIPOSTE_HUD_H
