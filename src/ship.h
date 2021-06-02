//
// Created by Caelum van Ispelen on 6/2/21.
//

#ifndef RIPOSTE_SHIP_H
#define RIPOSTE_SHIP_H

#include "unit.h"

namespace rip {
    class Hud;

    class CarryUnitsCapability : public Capability {
        std::vector<UnitId> carryingUnits;
        int capacity;

    public:
        CarryUnitsCapability(const UnitId &unitId, int capacity);

        void onUnitMoved(Game &game, glm::uvec2 oldPos) override;

        void update(Game &game);

        UnitUIStatus paintMainUI(Game &game, Hud &hud, nk_context *nk) override;

        void addCarryingUnit(UnitId unit);
        void removeCarryingUnit(UnitId unit);
        bool isCarryingUnit(UnitId unit) const;

        int getCapacity() const;
        int getNumCarriedUnits() const;
    };
}

#endif //RIPOSTE_SHIP_H
