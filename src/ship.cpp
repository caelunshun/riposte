//
// Created by Caelum van Ispelen on 6/2/21.
//

#include "ship.h"
#include "game.h"
#include "hud.h"
#include "stack.h"
#include "tile.h"

namespace rip {
    CarryUnitsCapability::CarryUnitsCapability(const UnitId &unitId, int capacity) : Capability(unitId), capacity(capacity) {}

    void CarryUnitsCapability::onUnitMoved(Game &game, glm::uvec2 oldPos) {
        // Move units with the ship.
        const auto newPos = game.getUnit(unitID).getPos();
        std::vector<UnitId> toRemove;
        for (const auto unitID : carryingUnits) {
            if (!game.getUnits().id_is_valid(unitID)) {
                toRemove.push_back(unitID);
                continue;
            }
            auto &unit = game.getUnit(unitID);
            if (unit.getPos() == oldPos) {
                unit.teleportTo(newPos, game);
                unit.fortify();
            } else {
                toRemove.push_back(unitID);
            }
        }
        for (const auto id : toRemove) removeCarryingUnit(id);
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

            nk_label(nk, (std::to_string(cap.getNumCarriedUnits()) + " / " + std::to_string(cap.getCapacity()) + " units").c_str(), NK_TEXT_ALIGN_LEFT);

            bool canDisembarkHere = game.getTile(ship.getPos()).getTerrain() != Terrain::Ocean;

            for (const auto unitID : stack.getUnits()) {
                auto &unitInStack = game.getUnit(unitID);
                if (unitInStack.getKind().ship) continue;

                int isCarried = cap.isCarryingUnit(unitID);
                nk_checkbox_label(nk, unitInStack.getKind().name.c_str(), &isCarried);

                if (isCarried) {
                    if (cap.getNumCarriedUnits() < cap.getCapacity()) {
                        cap.addCarryingUnit(unitID);
                        unitInStack.fortify();
                    }
                } else if (canDisembarkHere) {
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
            update(game);
            auto stackID = unit.getStack(game);
            hud.openWindow(std::make_shared<BoardUnitsWindow>(stackID, unitID));
        }

        return UnitUIStatus::None;
    }

    void CarryUnitsCapability::addCarryingUnit(UnitId unit) {
        if (carryingUnits.size() < capacity) {
            removeCarryingUnit(unit);
            carryingUnits.push_back(unit);
        }
    }

    void CarryUnitsCapability::removeCarryingUnit(UnitId unit) {
        auto it = std::find(carryingUnits.begin(), carryingUnits.end(), unit);
        if (it != carryingUnits.end()) {
            carryingUnits.erase(it);
        }
    }

    bool CarryUnitsCapability::isCarryingUnit(UnitId unit) const {
        return std::find(carryingUnits.begin(), carryingUnits.end(), unit) != carryingUnits.end();
    }

    int CarryUnitsCapability::getCapacity() const {
        return capacity;
    }

    int CarryUnitsCapability::getNumCarriedUnits() const {
        return carryingUnits.size();
    }

    void CarryUnitsCapability::update(Game &game) {
        for (int i = carryingUnits.size() - 1; i >= 0; i--) {
            if (!game.getUnits().id_is_valid(carryingUnits[i])) {
                removeCarryingUnit(carryingUnits[i]);
            }

            if (game.getUnit(carryingUnits[i]).getPos() != game.getUnit(unitID).getPos()) {
                removeCarryingUnit(carryingUnits[i]);
            }
        }
    }

    const std::vector<UnitId> &CarryUnitsCapability::getCarryingUnits() const {
        return carryingUnits;
    }
}
