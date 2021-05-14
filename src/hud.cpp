//
// Created by Caelum van Ispelen on 5/14/21.
//

#include <iostream>
#include "hud.h"
#include "game.h"
#include "ripmath.h"

namespace rip {
    Hud::Hud(NVGcontext *vg, nk_context *nk) : vg(vg), nk(nk), selectedUnit() {}

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

            nvgStrokeColor(vg, nvgRGBA(255, 255, 255, 200));
            nvgStrokeWidth(vg, 4);
            nvgStroke(vg);
        }
    }

    void Hud::update(Game &game) {
        paintSelectedUnit(game);
    }

    void Hud::handleClick(Game &game, MouseEvent event) {
        auto tilePos = game.getPosFromScreenOffset(game.getCursor().getPos());
        if (event.button == MouseButton::Left && event.action == MouseAction::Press) {
            auto unit = game.getUnitAtPosition(tilePos);
            if (unit == nullptr) {
                selectedUnit = std::optional<UnitId>();
            } else if (unit->getOwner() == game.getThePlayerID()) {
                selectedUnit = std::make_optional<UnitId>(unit->getID());
            }
        } else if (selectedUnit.has_value()
                && event.button == MouseButton::Right && event.action == MouseAction::Release) {
            auto &unit = game.getUnit(*selectedUnit);
            unit.moveTo(tilePos, game);
        }
    }
}
