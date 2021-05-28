//
// Created by Caelum van Ispelen on 5/14/21.
//

#include <iostream>
#include "renderer.h"
#include "hud.h"
#include "game.h"
#include "unit.h"
#include "city.h"
#include "ripmath.h"
#include "stack.h"
#include <nuklear.h>
#include <sstream>
#include <iomanip>

namespace rip {
    Hud::Hud(std::shared_ptr<Assets> assets, NVGcontext *vg, nk_context *nk, GLFWwindow *window) : vg(vg), nk(nk), selectedUnitPath(std::vector<glm::uvec2>()), assets(assets), window(window) {
        goldIcon = std::dynamic_pointer_cast<Image>(assets->get("icon/gold"));
        beakerIcon = std::dynamic_pointer_cast<Image>(assets->get("icon/beaker"));
    }

    void Hud::paintPath(Game &game, const Stack &stack, glm::uvec2 start, const Path &path) {
        auto prev = start;
        nvgBeginPath(vg);
        for (const auto point : path.getPoints()) {
            auto prevOffset = game.getScreenOffset(prev) + 50.0f;
            auto currOffset = game.getScreenOffset(point) + 50.0f;
            nvgMoveTo(vg, prevOffset.x, prevOffset.y);
            nvgLineTo(vg, currOffset.x, currOffset.y);
            prev = point;
        }

        NVGcolor color;
        bool wouldAttack = false;
        for (const auto unitID : selectedUnits) {
            if (game.getUnit(unitID).wouldAttackPos(game, path.getDestination())) {
                wouldAttack = true;
                break;
            }
        }
        if (wouldAttack) {
            color = nvgRGBA(225, 82, 62, 180);
        } else {
            color = nvgRGBA(255, 255, 255, 180);
        }
        nvgStrokeColor(vg, color);
        nvgStrokeWidth(vg, 5);
        nvgLineCap(vg, NVG_ROUND);
        nvgStroke(vg);
    }

    void Hud::paintSelectedUnit(Game &game) {
        if (selectedStack.has_value()) {
            auto stackID = *selectedStack;

            const auto &stack = game.getStack(stackID);
            auto offset = game.getScreenOffset(stack.getPos());

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
            bool movementLeft = true;
            for (const auto unitID : selectedUnits) {
                if (game.getUnit(unitID).getMovementLeft() <= 0.1) {
                    movementLeft = false;
                    break;
                }
            }
            if (!movementLeft) {
                color = nvgRGBA(218, 41, 28, 200);
            } else {
                color = nvgRGBA(255, 255, 255, 200);
            }

            nvgStrokeColor(vg, color);
            nvgStrokeWidth(vg, 4);
            nvgStroke(vg);

            if (selectedUnitPath.getNumPoints() != 0) {
                paintPath(game, stack, stack.getPos(), selectedUnitPath);
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

    void Hud::paintGenericUnitUI(Game &game) {
        if (!selectedStack.has_value() || selectedUnits.empty()) return;

        nk_layout_row_push(nk, 100);
        if (nk_button_label(nk, "Fortify")) {
            for (const auto unitID : selectedUnits) {
                game.getUnit(unitID).fortify();
            }
            selectedStack = {};
            updateSelectedUnit(game);
        }
        bool needsHealing = false;
        for (const auto unitID : selectedUnits) {
            if (game.getUnit(unitID).getHealth() != 1) {
                needsHealing = true;
                break;
            }
        }
        if (needsHealing) {
            nk_layout_row_push(nk, 100);
            if (nk_button_label(nk, "Heal")) {
                for (const auto unitID : selectedUnits) {
                    game.getUnit(unitID).fortifyUntilHealed();
                }
                selectedStack = {};
                updateSelectedUnit(game);
            }
        }
        nk_layout_row_push(nk, 100);
        if (nk_button_label(nk, "Skip")) {
            for (const auto unitID : selectedUnits) {
                game.getUnit(unitID).skipTurn();
            }
            selectedStack = {};
            updateSelectedUnit(game);
        }
    }

    void Hud::paintUnitUI(Game &game) {
        paintGenericUnitUI(game);
        if (selectedStack.has_value() && selectedUnits.size() == 1) {
            auto selectedUnitID = selectedUnits[0];
            bool kill = false;
            auto &unit = game.getUnit(selectedUnitID);

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

                text = "Movement: " + std::to_string(static_cast<int>(unit.getMovementLeft()));
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
                    selectedStack = {};
                    return;
                }
            }

            if (kill) {
                game.killUnit(selectedUnitID);
                selectedStack = {};
            }
        } else if (selectedStack.has_value() && selectedUnits.size() != 1) {
            nk_layout_row_push(nk, 150);
            nk_label_wrap(nk, (std::to_string(selectedUnits.size()) + " units").c_str());
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
            /*if (game.getNextUnitToMove().has_value()) { // DEBUG - AI
                // Need to move all units first.
                pushMessage("Move all your units before ending the turn!", {255,255,255});
                updateSelectedUnit(game);
            } else*/ {
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

        const auto spacing = 20;

        // Background pane.
        // First we have to find the text bounds.
        glm::vec2 pos(posX, posY);
        glm::vec2 end = pos;
        auto posYTemp = posY;
        for (const auto &message : messages) {
            float bounds[4];
            nvgTextBounds(vg, posX, posYTemp, message.text.c_str(), nullptr, bounds);

            if (pos.x > bounds[0]) {
                pos.x = bounds[0];
            }
            if (pos.y > bounds[1]) {
                pos.y = bounds[1];
            }
            if (end.x < bounds[2]) {
                end.x = bounds[2];
            }
            if (end.y < bounds[3]) {
                end.y = bounds[3];
            }

            posYTemp += spacing;
        }

        const auto padding = 5;
        nvgBeginPath(vg);
        nvgRect(vg, pos.x - padding, pos.y - padding, end.x - pos.x + padding, end.y - pos.y + padding);
        nvgFillColor(vg, nvgRGBA(0, 0, 0, 150));
        nvgFill(vg);

        for (const auto &message : messages) {
            float alpha = 1;
            float timeLeft =  message.disappearTime - glfwGetTime();
            if (timeLeft < 1) {
                alpha = std::clamp(timeLeft, 0.0f, 1.0f);
            }
            auto color = message.color;
            nvgFillColor(vg, nvgRGBA(color[0], color[1], color[2], static_cast<uint8_t>(alpha * 255.0f)));

            nvgText(vg, posX, posY, message.text.c_str(), nullptr);

            posY += spacing;
        }

        while (!messages.empty() && messages[0].disappearTime <= glfwGetTime()) {
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
        if (selectedStack.has_value() &&
                !game.getStacks().id_is_valid(*selectedStack)) {
            selectedStack = {};
        }

        if (!selectedStack.has_value()) {
            selectedUnits.clear();
        }

        for (const auto unitID : selectedUnits) {
            if (!game.getUnits().id_is_valid(unitID)) {
                selectedUnits.clear();
            }
        }

        if (hasFocus(game)) {
            selectedStack = {};
        }

        if (isSelectingPath && selectedStack.has_value()) {
            auto currentPos = game.getPosFromScreenOffset(game.getCursor().getPos());
            if (selectedUnitPath.getNumPoints() == 0 || currentPos != selectedUnitPath.getDestination()) {
                trySetSelectedPath(game, game.getStack(*selectedStack).getPos(), currentPos);
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
        paintScoreHud(game);
        paintStackSelectionBar(game);

        clickPos = {};
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
        return; // DEBUG - AI
        auto unit = game.getNextUnitToMove();
        selectedUnits.clear();
        if (unit.has_value()) {
            selectedStack = game.getUnit(*unit).getStack(game);
            selectedUnits.push_back(*unit);
        } else {
            selectedStack = {};
        }
        if (selectedStack.has_value() && !hasFocus(game)) {
            SmoothAnimation animation(game.getView().getMapCenter(), glm::vec2(game.getStack(*selectedStack).getPos()) * 100.0f, 2000.0f, 2.0f);
            game.getView().setCenterAnimation(animation);
        }
    }

    void Hud::handleClick(Game &game, MouseEvent event) {
        if (event.button == MouseButton::Left && event.action == MouseAction::Press) {
            clickPos = game.getCursor().getPos();
        }

        if (game.getCursor().getPos().y > game.getCursor().getWindowSize().y - 150) {
            // Click in UI. Don't interfere with the HUD.
            return;
        }

        if (hasFocus(game)) {
            return;
        }

        auto tilePos = game.getPosFromScreenOffset(game.getCursor().getPos());
        if (event.button == MouseButton::Left && event.action == MouseAction::Press) {
            auto stackID = game.getStackByKey(game.getThePlayerID(), tilePos);
            selectedUnits.clear();
            selectedStack = stackID;
            if (selectedStack.has_value()) {
                selectedUnits.push_back(game.getStack(*selectedStack).getBestUnit(game));
            }
        } else if (selectedStack.has_value()
                   && event.button == MouseButton::Right && event.action == MouseAction::Press) {
            const auto &stack = game.getStack(*selectedStack);
            trySetSelectedPath(game, stack.getPos(), tilePos);
        } else if (selectedStack.has_value()
            && event.button == MouseButton::Right && event.action == MouseAction::Release) {

            bool shouldFinishPath = false;
            for (const auto unitID : selectedUnits) {
                auto &unit = game.getUnit(unitID);
                unit.setPath(selectedUnitPath);
                unit.moveAlongCurrentPath(game);
                selectedStack = unit.getStack(game);

                if (unit.getMovementLeft() == 0) {
                    shouldFinishPath = true;
                }
            }

            if (shouldFinishPath) {
                selectedStack = {};
                updateSelectedUnit(game);
                selectedUnitPathError = std::optional<glm::uvec2>();
                selectedUnitPath = Path(std::vector<glm::uvec2>());
            }

            selectedUnitPath = Path(std::vector<glm::uvec2>());
            selectedUnitPathError = std::optional<glm::uvec2>();
            isSelectingPath = false;
        }
    }

    void Hud::pushMessage(std::string message, std::array<uint8_t, 3> color) {
        messages.emplace_front(message, glfwGetTime() + 7, color);
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
                            nk_label(nk, ("* Can train " + std::string(article(unit->name)) + " " + unit->name).c_str(), NK_TEXT_ALIGN_LEFT);
                        }
                        for (const auto &improvement : tech->unlocksImprovements) {
                            nk_label(nk, ("* Can build " + std::string(article(improvement)) + " " + improvement).c_str(), NK_TEXT_ALIGN_LEFT);
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
        glm::vec2 size(300, 150);
        nk_begin(nk, "topLeft", nk_rect(0, 0, size.x, size.y), 0);

        nk_layout_row_begin(nk, NK_STATIC, 20, 3);

        // Gold
        nk_layout_row_push(nk, 20);
        auto image = nk_image_id(goldIcon->id);
        nk_image(nk, image);

        auto &player = game.getThePlayer();
        const auto text = ": " + std::to_string(player.getGold());
        nk_layout_row_push(nk, 40);
        nk_label(nk, text.c_str(), NK_TEXT_ALIGN_LEFT);

        const auto netGold = player.getNetGold();
        nk_color netTextColor;
        std::string netText;
        if (netGold < 0) {
            netText = "(-" + std::to_string(abs(netGold)) + " / turn)";
            netTextColor = nk_rgb(231, 60, 62);
        } else {
            netText = "(+" + std::to_string(netGold) + " / turn)";
            netTextColor = nk_rgb(68, 194, 113);
        }
        nk_layout_row_push(nk, 100);
        nk_label_colored(nk, netText.c_str(), NK_TEXT_ALIGN_LEFT, netTextColor);

        nk_layout_row_push(nk, 100);
        nk_label_colored(nk, ("Expenses: " + std::to_string(player.getExpenses())).c_str(), NK_TEXT_ALIGN_LEFT, nk_rgb(231, 60, 62));

        nk_layout_row_push(nk, 100);
        nk_label_colored(nk, ("Revenue: " + std::to_string(player.getBaseRevenue())).c_str(), NK_TEXT_ALIGN_LEFT, nk_rgb(68, 194, 113));

        // Research slider
        nk_layout_row_end(nk);

        nk_layout_row_begin(nk, NK_STATIC, 50, 1);
        nk_layout_row_push(nk, 280);
        if (nk_group_begin(nk, "researchSlider", 0)) {
            nk_layout_row_begin(nk, NK_STATIC, 20, 5);

            nk_layout_row_push(nk, 30);
            auto beaker = nk_image_id(beakerIcon->id);
            nk_image(nk, beaker);

            nk_layout_row_push(nk, 40);
            auto percentText = std::to_string(player.getSciencePercent()) + "%";
            nk_label(nk, percentText.c_str(), NK_TEXT_ALIGN_LEFT);

            nk_layout_row_push(nk, 30);
            if (nk_button_label(nk, "+")) {
                player.setSciencePercent(player.getSciencePercent() + 10, game);
            }
            nk_layout_row_push(nk, 30);
            if (nk_button_label(nk, "-")) {
                player.setSciencePercent(player.getSciencePercent() - 10, game);
            }

            nk_layout_row_push(nk, 80);
            auto beakersText = "(" + std::to_string(player.getBeakerRevenue()) + " / turn)";
            nk_label(nk, beakersText.c_str(), NK_TEXT_ALIGN_LEFT);

            nk_group_end(nk);
        }

        nk_end(nk);
    }

    void Hud::paintScoreHud(Game &game) {
        auto windowSize = game.getCursor().getWindowSize();
        glm::vec2 size(200, 200);
        auto pos = windowSize - size - glm::vec2(0, 100);

        nvgBeginPath(vg);
        nvgRect(vg, pos.x, pos.y, size.x, size.y);
        nvgFillColor(vg, nvgRGBA(0x2b, 0x2b, 0x2b, 255));
        nvgFill(vg);

        // Sort players by score.
        std::vector<const Player*> players(game.getPlayers().size());
        int i = 0;
        for (const auto &player : game.getPlayers()) {
            players[i++] = &player;
        }
        std::sort(players.begin(), players.end(), [] (const Player *a, const Player *b) {
            return a->getScore() > b->getScore();
        });

        // Draw players.
        nvgFontSize(vg, 15);
        nvgTextAlign(vg, NVG_ALIGN_LEFT | NVG_ALIGN_TOP);
        float textCursor = pos.y + 5;
        for (const auto *player : players) {
            const auto &leaderName = player->getCiv().leader;
            const auto score = player->getScore();
            const auto color = player->getCiv().color;

            nvgFillColor(vg, nvgRGB(255, 255, 255));
            nvgText(vg, pos.x, textCursor, std::to_string(score).c_str(), nullptr);
            float _bounds[4];
            float advance = nvgTextBounds(vg, pos.x, textCursor, std::to_string(score).c_str(), nullptr, _bounds);
            nvgFillColor(vg, nvgRGB(color[0], color[1], color[2]));
            nvgText(vg, pos.x + advance, textCursor, (" " + leaderName).c_str(), nullptr);

            textCursor += 30;
        }
        nvgReset(vg);
    }

    void Hud::paintStackSelectionBar(Game &game) {
        if (!selectedStack.has_value()) return;
        const auto &stack = game.getStack(*selectedStack);

        // Paint icon for each unit in the stack.
        glm::vec2 pos(200, game.getCursor().getWindowSize().y - 140);
        for (const auto unitID : stack.getUnits()) {
            const auto &unit = game.getUnit(unitID);
            const auto iconWidth = 35;
            const auto padding = 7;

            nvgBeginPath(vg);
            nvgRect(vg, pos.x, pos.y, iconWidth, iconWidth);

            nvgFillColor(vg, nvgRGB(80, 80, 80));
            nvgFill(vg);

            auto iconID = "icon/unit_head/" + unit.getKind().id;
            const auto icon = std::dynamic_pointer_cast<Image>(assets->get(iconID));
            const auto paint = nvgImagePattern(vg, pos.x, pos.y, iconWidth, iconWidth, 0, icon->id, 1);
            nvgFillPaint(vg, paint);
            nvgFill(vg);

            NVGcolor borderColor;
            if (std::find(selectedUnits.begin(), selectedUnits.end(), unitID) != selectedUnits.end()) {
                borderColor = nvgRGB(255, 205, 0);
            } else {
                borderColor = nvgRGB(0, 0, 0);
            }
            nvgStrokeColor(vg, borderColor);
            nvgStrokeWidth(vg, 1);
            nvgStroke(vg);

            const float circleRadius = 5;
            const glm::vec2 circlePos(pos.x + iconWidth - circleRadius / 2, pos.y + circleRadius / 2);
            nvgBeginPath(vg);
            nvgCircle(vg, circlePos.x, circlePos.y, circleRadius);

            NVGcolor circleColor;
            if (unit.isFortified()) {
                circleColor = nvgRGB(180, 180, 180);
            } else if (unit.getMovementLeft() <= 0.1) {
                circleColor = nvgRGB(231, 60, 62);
            } else if (unit.getMovementLeft() == unit.getKind().movement) {
                circleColor = nvgRGB(68, 194, 113);
            } else {
                circleColor = nvgRGB(254, 221, 0);
            }
            nvgFillColor(vg, circleColor);
            nvgFill(vg);
            nvgStrokeColor(vg, nvgRGB(0, 0, 0));
            nvgStrokeWidth(vg, 0.8);
            nvgStroke(vg);

            // Check for click - if so, toggle selection on this unit.
            if (wasRectClicked(pos, glm::vec2(iconWidth))) {
                auto it = std::find(selectedUnits.begin(), selectedUnits.end(), unitID);
                if (glfwGetKey(window, GLFW_KEY_LEFT_SHIFT) == GLFW_PRESS) {
                    if (it != selectedUnits.end()) {
                        selectedUnits.erase(it);
                    } else {
                        selectedUnits.push_back(unitID);
                    }
                } else {
                    selectedUnits.clear();
                    selectedUnits.push_back(unitID);
                }
            }

            pos.x += iconWidth + padding;
        }
    }

    bool Hud::wasRectClicked(glm::vec2 pos, glm::vec2 size) const {
        return clickPos.has_value()
            && (clickPos->x >= pos.x && clickPos->y >= pos.y)
            && (clickPos->x <= pos.x + size.x && clickPos->y <= pos.y + size.y);
    }
}
