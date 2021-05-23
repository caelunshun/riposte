//
// Created by Caelum van Ispelen on 5/14/21.
//

#include <iostream>
#include "renderer.h"
#include "hud.h"
#include "game.h"
#include "ripmath.h"
#include <nuklear.h>
#include <sstream>
#include <iomanip>

namespace rip {
    Hud::Hud(const Assets &assets, NVGcontext *vg, nk_context *nk) : vg(vg), nk(nk), selectedUnit(), selectedUnitPath(std::vector<glm::uvec2>()) {
        goldIcon = std::dynamic_pointer_cast<Image>(assets.get("icon/gold"));
    }

    void Hud::paintPath(Game &game, glm::uvec2 start, const Path &path) {
        auto prev = start;
        nvgBeginPath(vg);
        for (const auto point : path.getPoints()) {
            auto prevOffset = game.getScreenOffset(prev) + 50.0f;
            auto currOffset = game.getScreenOffset(point) + 50.0f;
            nvgMoveTo(vg, prevOffset.x, prevOffset.y);
            nvgLineTo(vg, currOffset.x, currOffset.y);
            prev = point;
        }

        nvgStrokeColor(vg, nvgRGBA(255, 255, 255, 180));
        nvgStrokeWidth(vg, 5);
        nvgLineCap(vg, NVG_ROUND);
        nvgStroke(vg);
    }

    void Hud::paintSelectedUnit(Game &game) {
        if (selectedUnit.has_value()) {
            auto unitID = *selectedUnit;
            if (!game.getUnits().id_is_valid(unitID)) {
                selectedUnit = std::optional<UnitId>();
                return;
            }

            auto &unit = game.getUnit(unitID);
            auto offset = game.getScreenOffset(unit.getPos());

            nvgBeginPath(vg);

            auto radius = 50.0f;
            auto center = offset + radius;

            auto angleOffset = glfwGetTime() * 2 * pi() / 10;
            auto numDashes = 16;
            for (int i = 0; i < numDashes; i++) {
                auto arcLength = (2 * pi() / numDashes);
                auto arcStart = angleOffset + i * arcLength;
                auto arcEnd = angleOffset + (i + 1) * arcLength - 0.1;

                nvgArc(vg, center.x, center.y, radius, arcStart, arcEnd, NVG_CW);
                nvgMoveTo(vg, center.x + radius * cos(arcEnd + 0.3), center.y + radius * sin(arcEnd + 0.3));
            }

            NVGcolor color;
            if (unit.getMovementLeft() == 0) {
                color = nvgRGBA(218, 41, 28, 200);
            } else {
                color = nvgRGBA(255, 255, 255, 200);
            }

            nvgStrokeColor(vg, color);
            nvgStrokeWidth(vg, 4);
            nvgStroke(vg);

            if (selectedUnitPath.getNumPoints() != 0) {
                paintPath(game, unit.getPos(), selectedUnitPath);
            }

            if (selectedUnitPathError.has_value()) {
                auto offset = game.getScreenOffset(*selectedUnitPathError) + 50.0f;
                nvgBeginPath(vg);
                nvgCircle(vg, offset.x, offset.y, 50);
                nvgStrokeColor(vg, nvgRGBA(218, 41, 28, 200));
                nvgStrokeWidth(vg, 5);
                nvgStroke(vg);
            }
        }
    }

    void Hud::paintUnitUI(Game &game) {
        if (selectedUnit.has_value()) {
            bool kill = false;
            auto &unit = game.getUnit(*selectedUnit);

            nk_layout_row_push(nk, 150);
            if (nk_group_begin(nk, "unit hud", 0)) {
                nk_layout_row_dynamic(nk, 15, 1);

                auto text = unit.getKind().name;
                nk_label(nk, text.c_str(), NK_TEXT_ALIGN_LEFT);

                if (unit.getKind().strength != 0) {
                    std::stringstream strength;
                    strength << "Strength: " << std::fixed << std::setprecision(1) << unit.getCombatStrength();
                    nk_label(nk, strength.str().c_str(), NK_TEXT_ALIGN_LEFT);
                }

                text = "Movement: " + std::to_string(unit.getMovementLeft());
                if (unit.getMovementLeft() != unit.getKind().movement) {
                    text += " / " + std::to_string(unit.getKind().movement);
                }
                nk_label(nk, text.c_str(), NK_TEXT_ALIGN_LEFT);

                nk_group_end(nk);
            }

            nk_layout_row_push(nk, 100);
            if (nk_button_label(nk, "Kill")) {
                kill = true;
            }

            for (const auto &capability : unit.getCapabilities()) {
                if (capability->paintMainUI(game, nk) == UnitUIStatus::Deselect) {
                    selectedUnit = {};
                    return;
                }
            }

            if (kill) {
                game.killUnit(*selectedUnit);
                selectedUnit = std::optional<UnitId>();
            }
        }
    }

    void Hud::paintMainHud(Game &game) {
        auto height = 100;
        nk_begin(nk, "HUD",
                 nk_rect(0, game.getCursor().getWindowSize().y - height, game.getCursor().getWindowSize().x, height),
                 0);

        nk_layout_row_begin(nk, NK_STATIC, 80, 20);
        nk_layout_row_push(nk, 100);

        auto turnText = "Turn " + std::to_string(game.getTurn());
        nk_label(nk, turnText.c_str(), NK_TEXT_ALIGN_CENTERED);

        nk_layout_row_push(nk, 100);

        if (nk_button_label(nk, "Next Turn") && !hasFocus(game)) {
            if (game.getNextUnitToMove().has_value()) {
                // Need to move all units first.
                pushMessage("Move all your units before ending the turn!");
                updateSelectedUnit(game);
            } else {
                game.advanceTurn();
                updateSelectedUnit(game);
            }
        }

        paintUnitUI(game);

        nk_end(nk);
    }

    void Hud::paintMessages(Game &game) {
        auto posX = game.getCursor().getWindowSize().x / 2;
        auto posY = 50.0f;

        nvgFontSize(vg, 14);
        nvgFontFace(vg, "default");
        nvgTextAlign(vg, NVG_ALIGN_CENTER | NVG_ALIGN_BASELINE);

        for (const auto &message : messages) {
            float alpha = 1;
            float timeLeft =  message.disappearTime - glfwGetTime();
            if (timeLeft < 1) {
                alpha = std::clamp(timeLeft, 0.0f, 1.0f);
            }
            nvgFillColor(vg, nvgRGBA(255, 255, 255, static_cast<uint8_t>(alpha * 255.0f)));

            float bounds[4];
            nvgTextBounds(vg, 0, 0, message.text.c_str(), nullptr, bounds);
            nvgText(vg, posX, posY, message.text.c_str(), nullptr);

            posY += bounds[3] + 14;
        }

        if (!messages.empty() && messages[0].disappearTime <= glfwGetTime()) {
            messages.pop_front();
        }
    }

    void Hud::paintResearchBar(Game &game) {
        const auto &research = game.getThePlayer().getResearchingTech();
        float progress = 0;
        if (research.has_value()) {
            progress = static_cast<float>(research->beakersAccumulated) / research->tech->cost;
        }
        progress = std::clamp(progress, 0.0f, 1.0f);

        glm::vec2 size(400, 30);
        glm::vec2 progressSize(size.x * progress, size.y);
        glm::vec2 offset(game.getCursor().getWindowSize().x / 2 - size.x / 2, 1);
        auto end = offset + size;

        nvgBeginPath(vg);
        nvgRect(vg, offset.x, offset.y, size.x, size.y);
        nvgFillColor(vg, nvgRGBA(100, 100, 100, 150));
        nvgFill(vg);
        nvgStrokeColor(vg, nvgRGB(0, 0, 0));
        nvgStrokeWidth(vg, 1);
        nvgStroke(vg);

        nvgBeginPath(vg);
        nvgRect(vg, offset.x, offset.y, progressSize.x, progressSize.y);
        nvgFillColor(vg, nvgRGB(108, 198, 74));
        nvgFill(vg);

        std::string text = "Research: ";
        if (research.has_value()) {
            text += research->tech->name + " (" +
                    std::to_string(research->estimateCompletionTurns(game.getThePlayer().getBeakerRevenue()))
                    + ")";
        } else {
            text += "None";
        }
        nvgFontSize(vg, 15);
        nvgFillColor(vg, nvgRGB(255, 255, 255));
        nvgTextAlign(vg, NVG_ALIGN_MIDDLE | NVG_ALIGN_CENTER);
        nvgText(vg, offset.x + size.x / 2, offset.y + size.y / 2, text.c_str(), nullptr);
    }

    void Hud::update(Game &game) {
        if (selectedUnit.has_value() &&
                !game.getUnits().id_is_valid(*selectedUnit)) {
            selectedUnit = std::optional<UnitId>();
        }

        if (hasFocus(game)) {
            selectedUnit = std::optional<UnitId>();
        }

        if (isSelectingPath && selectedUnit.has_value()) {
            auto currentPos = game.getPosFromScreenOffset(game.getCursor().getPos());
            if (selectedUnitPath.getNumPoints() == 0 || currentPos != selectedUnitPath.getDestination()) {
                trySetSelectedPath(game, game.getUnit(*selectedUnit).getPos(), currentPos);
            }
        }

        paintSelectedUnit(game);
        paintMainHud(game);
        auto cityID = getCityBuildPrompt(game);
        if (cityID.has_value()) {
            if (*cityID != lastCityBuildPrompt) {
                lastCityBuildPrompt = *cityID;
                const auto &city = game.getCity(*cityID);
                game.getView().setCenterAnimation(SmoothAnimation(game.getView().getMapCenter(), glm::vec2(city.getPos()) * 100.0f, 2000.0f, 0.5f));
            }
            paintCityBuildPrompt(game, *cityID);
        }
        paintResearchBar(game);
        paintTechPrompt(game);
        paintMessages(game);
        paintTopLeftHud(game);
    }

    void Hud::trySetSelectedPath(Game &game, glm::uvec2 from, glm::uvec2 to) {
        std::optional<VisibilityMap> visMap;
        if (!game.isCheatMode()) {
            visMap = game.getThePlayer().getVisibilityMap();
        }
        auto path = computeShortestPath(game, from, to, std::move(visMap));
        if (path.has_value()) {
            selectedUnitPath = std::move(*path);
            selectedUnitPathError = std::optional<glm::uvec2>();
        } else {
            selectedUnitPathError = std::make_optional(to);
            selectedUnitPath = Path(std::vector<glm::uvec2>());
        }

        isSelectingPath = true;
    }

    void Hud::updateSelectedUnit(Game &game) {
        selectedUnit = game.getNextUnitToMove();
        if (selectedUnit.has_value() && !hasFocus(game)) {
            SmoothAnimation animation(game.getView().getMapCenter(), glm::vec2(game.getUnit(*selectedUnit).getPos()) * 100.0f, 2000.0f, 2.0f);
            game.getView().setCenterAnimation(animation);
        }
    }

    void Hud::handleClick(Game &game, MouseEvent event) {
        if (game.getCursor().getPos().y > game.getCursor().getWindowSize().y - 100) {
            // Click in UI. Don't interfere with the HUD.
            return;
        }

        if (hasFocus(game)) {
            return;
        }

        auto tilePos = game.getPosFromScreenOffset(game.getCursor().getPos());
        if (event.button == MouseButton::Left && event.action == MouseAction::Press) {
            auto unit = game.getUnitAtPosition(tilePos);
            if (unit == nullptr) {
                selectedUnit = std::optional<UnitId>();
            } else if (unit->getOwner() == game.getThePlayerID()) {
                selectedUnit = std::make_optional<UnitId>(unit->getID());

                if (unit->hasPath()) {
                    selectedUnitPath = unit->getPath();
                }
            }
        } else if (selectedUnit.has_value()
                   && event.button == MouseButton::Right && event.action == MouseAction::Press) {
            const auto &unit = game.getUnit(*selectedUnit);
            trySetSelectedPath(game, unit.getPos(), tilePos);
        } else if (selectedUnit.has_value()
            && event.button == MouseButton::Right && event.action == MouseAction::Release) {
            auto &unit = game.getUnit(*selectedUnit);
            unit.setPath(std::move(selectedUnitPath));
            unit.moveAlongCurrentPath(game);
            selectedUnitPath = Path(std::vector<glm::uvec2>());
            selectedUnitPathError = std::optional<glm::uvec2>();
            isSelectingPath = false;

            if (unit.getMovementLeft() == 0) {
                updateSelectedUnit(game);
                selectedUnitPathError = std::optional<glm::uvec2>();
                selectedUnitPath = Path(std::vector<glm::uvec2>());
            }
        }
    }

    void Hud::pushMessage(std::string message) {
        messages.emplace_front(message, glfwGetTime() + 7);
    }

    void Hud::paintCityBuildPrompt(Game &game, CityId cityID) {
        auto &city = game.getCity(cityID);

        auto size = game.getCursor().getWindowSize();
        auto width = 300;
        auto height = 400;
        auto margin = 20;

        nk_begin(nk, "city build prompt", nk_rect(size.x - width - margin, margin, width, height), 0);
        nk_layout_row_dynamic(nk, 50, 1);

        const auto &previousTask = city.getPreviousBuildTask();
        std::string text;
        if (previousTask.empty()) {
            text = "What would you like to build in " + city.getName() + "?";
        } else {
            text = "You have built a " + city.getPreviousBuildTask() + " in " + city.getName() + ". What would you like to work on next?";
        }
        nk_label_colored_wrap(nk, text.c_str(), nk_rgb(255, 255, 255));

        for (auto &task : city.getPossibleBuildTasks(game)) {
            auto label = task->getName() + " ("
                    + std::to_string(city.estimateTurnsForCompletion(*task, game))
                    + ")";
            if (nk_button_label(nk, label.c_str())) {
                city.setBuildTask(std::move(task));
                updateSelectedUnit(game);
            }
        }

        nk_end(nk);
    }

    std::optional<CityId> Hud::getCityBuildPrompt(const Game &game) const {
       for (const auto &city : game.getCities()) {
           if (!city.hasBuildTask() && city.getOwner() == game.getThePlayerID()) {
               return std::make_optional(city.getID());
           }
       }
       return std::optional<CityId>();
    }

    bool Hud::hasFocus(const Game &game) const {
        return getCityBuildPrompt(game).has_value() || shouldShowTechPrompt(game);
    }

    void Hud::handleKey(Game &game, int key) {
        if (key == GLFW_KEY_L) {
            game.toggleCheatMode();
        }
    }

    bool Hud::shouldShowTechPrompt(const Game &game) const {
        return game.getTurn() != 0 && !game.getThePlayer().getResearchingTech().has_value();
    }

    void Hud::paintTechPrompt(Game &game) {
        if (!shouldShowTechPrompt(game)) return;

        const auto windowSize = game.getCursor().getWindowSize();
        glm::vec2 size(600, 350);
        auto bounds = nk_rect(
                windowSize.x / 2 - size.x / 2,
                50,
                size.x,
                size.y);
        nk_begin(nk, "research prompt", bounds, 0);
        nk_layout_row_dynamic(nk, 60, 1);

        nk_label_colored_wrap(nk, "What would you like to research next?", nk_rgb(255, 255, 255));

        auto beakers = game.getThePlayer().getBeakerRevenue();
        for (const auto &tech : game.getThePlayer().getTechs().getPossibleResearches()) {
            if (nk_widget_is_hovered(nk)) {
                // Show tooltip.
                if (nk_tooltip_begin(nk, 300)) {
                    nk_layout_row_begin(nk, NK_STATIC, 100, 2);
                    nk_layout_row_push(nk, 30);
                    nk_spacing(nk, 1);

                    nk_layout_row_push(nk, 270);
                    if (nk_group_begin(nk, "tech info", 0)) {
                        nk_layout_row_dynamic(nk, 20, 1);
                        nk_label(nk, ("Cost: " + std::to_string(tech->cost)).c_str(), NK_TEXT_ALIGN_LEFT);

                        for (const auto &unit : tech->unlocksUnits) {
                            nk_label(nk, ("* Can train a " + unit->name).c_str(), NK_TEXT_ALIGN_LEFT);
                        }
                        for (const auto &improvement : tech->unlocksImprovements) {
                            nk_label(nk, ("* Can build a " + improvement).c_str(), NK_TEXT_ALIGN_LEFT);
                        }
                        for (const auto &leadsTo : tech->leadsTo) {
                            nk_label(nk, ("* Leads to " + leadsTo->name).c_str(), NK_TEXT_ALIGN_LEFT);
                        }
                        for (const auto &entry : game.getRegistry().getResources()) {
                            const auto &resource = entry.second;
                            if (resource->revealedBy == tech->name) {
                                nk_label(nk, ("* Reveals " + resource->name).c_str(), NK_TEXT_ALIGN_LEFT);
                            }
                        }
                        nk_group_end(nk);
                    }

                    nk_tooltip_end(nk);
                }
            }

            int turnEstimate = tech->cost + 1;
            if (beakers != 0) {
                turnEstimate = (tech->cost + beakers - 1) / beakers;
            }
            auto text = tech->name + " (" + std::to_string(turnEstimate) + ")";
            if (nk_button_label(nk, text.c_str())) {
                game.getThePlayer().setResearchingTech(tech);
            }
        }

        nk_end(nk);
    }

    void Hud::paintTopLeftHud(Game &game) {
        glm::vec2 size(300, 100);
        nk_begin(nk, "topLeft", nk_rect(0, 0, size.x, size.y), 0);

        nk_layout_row_begin(nk, NK_STATIC, 20, 2);

        // Gold
        nk_layout_row_push(nk, 20);
        auto image = nk_image_id(goldIcon->id);
        nk_image(nk, image);

        const auto &player = game.getThePlayer();
        const auto text = std::to_string(player.getGold());
        nk_layout_row_push(nk, 50);
        nk_label(nk, text.c_str(), NK_TEXT_ALIGN_LEFT);

        nk_layout_row_push(nk, 100);
        nk_label_colored(nk, ("Expenses: " + std::to_string(player.getExpenses())).c_str(), NK_TEXT_ALIGN_LEFT, nk_rgb(231, 60, 62));

        nk_layout_row_push(nk, 100);
        nk_label_colored(nk, ("Revenue: " + std::to_string(player.getBaseRevenue())).c_str(), NK_TEXT_ALIGN_LEFT, nk_rgb(68, 194, 113));

        nk_end(nk);
    }
}
