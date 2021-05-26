//
// Created by Caelum van Ispelen on 5/25/21.
//

#include <optional>
#include "stack.h"
#include "unit.h"
#include "game.h"

namespace rip {
    Stack::Stack(PlayerId owner, glm::uvec2 pos) : owner(owner), pos(pos) {

    }

    void Stack::addUnit(UnitId unit) {
        units.push_back(unit);
    }

    void Stack::removeUnit(UnitId unit) {
        auto pos = std::find(units.begin(), units.end(), unit);
        if (pos != units.end()) {
            units.erase(pos);
        }
    }

    bool Stack::containsUnit(UnitId unit) const {
        return std::find(units.begin(), units.end(), unit) != units.end();
    }

    const absl::InlinedVector<UnitId, 2> &Stack::getUnits() const {
        return units;
    }

    UnitId Stack::getBestUnit(const Game &game) const {
        std::optional<UnitId> best;
        double bestStrength;
        for (const auto unitID : getUnits()) {
            const auto &unit = game.getUnit(unitID);
            if (!best.has_value() && unit.getCombatStrength() > bestStrength) {
                best = unitID;
                bestStrength = unit.getCombatStrength();
            }
        }
        assert(best.has_value());
        return *best;
    }

    void Stack::update(const Game &game) {

    }
}
