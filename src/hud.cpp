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

            auto angleOffset = sin(glfwGetTime()) * 2 * pi();
            auto numDashes = 16;
            for (int i = 0; i < numDashes; i++) {
                auto arcLength = (2 * pi() / numDashes);
                auto arcStart = angleOffset + i * arcLength;
                auto arcEnd = angleOffset + (i + 1) * arcLength - 0.3;

                nvgArc(vg, center.x, center.y, radius, arcStart, arcEnd, NVG_CCW);
                nvgMoveTo(vg, center.x + radius * cos(arcEnd + 0.3), center.y + radius * sin(arcEnd + 0.3));
            }

            nvgStrokeColor(vg, nvgRGBA(255, 255, 255, 200));
            nvgStrokeWidth(vg, 6);
            nvgStroke(vg);
        }
    }

    void Hud::update(Game &game) {
        paintSelectedUnit(game);
    }

    void Hud::handleClick(Game &game, glm::vec2 offset) {
        auto tilePos = game.getPosFromScreenOffset(offset);
        auto unit = game.getUnitAtPosition(tilePos);
        if (unit == nullptr) {
            selectedUnit = std::optional<UnitId>();
        } else {
            selectedUnit = std::make_optional<UnitId>(unit->getID());
        }
    }
}
