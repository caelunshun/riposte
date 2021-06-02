//
// Created by Caelum van Ispelen on 6/2/21.
//

#include "ship.h"
#include "game.h"
#include "hud.h"
#include "stack.h"

namespace rip {
    CarryUnitsCapability::CarryUnitsCapability(const UnitId &unitId, int capacity) : Capability(unitId), capacity(capacity) {}

    void CarryUnitsCapability::onUnitMoved(Game &game) {
        // Move units with the ship.
        const auto newPos = game.getUnit(unitID).getPos();
        for (const auto unitID : carryingUnits) {
            auto &unit = game.getUnit(unitID);
            unit.teleportTo(newPos, game);
        }
    }

    class BoardUnitsWindow : public Window {
        bool close = false;
        StackId stackID;
        UnitId shipID;

    public:
        BoardUnitsWindow(const StackId &stack, const UnitId &ship) : stackID(stack), shipID(ship) {}

        void paint(Game &game, nk_context *nk, NVGcontext *vg) override {
            glm::vec2 size(200, 400);
            auto pos = glm::vec2(game.getCursor().getWindowSize().x - size.x - 20, 20);
            nk_begin(nk, "boardUnits", nk_rect(pos.x, pos.y, size.x, size.y), 0);

            auto &stack = game.getStack(stackID);
            auto &ship = game.getUnit(shipID);
            auto &cap = *ship.getCapability<CarryUnitsCapability>();

            nk_layout_row_dynamic(nk, 50, 1);
            nk_label(nk, ("Board " + ship.getKind().name).c_str(), NK_TEXT_ALIGN_LEFT);

            for (const auto unitID : stack.getUnits()) {
                auto &unitInStack = game.getUnit(unitID);
                if (unitInStack.getKind().ship) continue;

                int isCarried = cap.isCarryingUnit(unitID);
                nk_checkbox_label(nk, unitInStack.getKind().name.c_str(), &isCarried);

                if (isCarried) {
                    cap.addCarryingUnit(unitID);
                } else {
                    cap.removeCarryingUnit(unitID);
                }
            }

            if (nk_button_label(nk, "Done")) {
                close = true;
            }

            nk_end(nk);
        }

        bool shouldClose() override {
            return close;
        }
    };

    UnitUIStatus CarryUnitsCapability::paintMainUI(Game &game, Hud &hud, nk_context *nk) {
        const auto &unit = game.getUnit(unitID);
        nk_layout_row_push(nk, 100);
        if (nk_button_label(nk, "Board Units")) {
            auto stackID = unit.getStack(game);
            hud.openWindow(std::make_shared<BoardUnitsWindow>(stackID, unitID));
        }

        return UnitUIStatus::None;
    }

    void CarryUnitsCapability::addCarryingUnit(UnitId unit) {
        removeCarryingUnit(unit);
        carryingUnits.push_back(unit);
    }

    void CarryUnitsCapability::removeCarryingUnit(UnitId unit) {
        auto it = std::find(carryingUnits.begin(), carryingUnits.end(), unit);
        if (it != carryingUnits.end()) {
            carryingUnits.erase(it);
        }
    }

    bool CarryUnitsCapability::isCarryingUnit(UnitId unit) {
        return std::find(carryingUnits.begin(), carryingUnits.end(), unit) != carryingUnits.end();
    }
}
