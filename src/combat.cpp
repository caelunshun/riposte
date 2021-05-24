//
// Created by Caelum van Ispelen on 5/24/21.
//

#include <cassert>
#include "combat.h"
#include "game.h"
#include "unit.h"

namespace rip {
    const float roundTime = 0.5;

    Combat::Combat(UnitId attacker, UnitId defender) : attackerID(attacker), defenderID(defender) {
        assert(attacker != defender);
    }

    float Combat::getNextRoundTime() const {
        return nextRound * roundTime;
    }

    void Combat::advance(Game &game, float dt) {
        time += dt;

        if (getNextRoundTime() > time) return;

        if (finished) return;

        auto &attacker = game.getUnit(attackerID);
        auto &defender = game.getUnit(defenderID);

        double a = attacker.getCombatStrength();
        double d = defender.getCombatStrength();

        double r = a / d;

        double attackerDamage = 20.0 * (3 * r + 1) / (3 + r) / 100.0;
        double defenderDamage = 20.0 * (3 + r) / (3 * r + 1) / 100.0;

        attacker.setHealth(attacker.getHealth() - defenderDamage);
        defender.setHealth(defender.getHealth() - attackerDamage);

        if (attacker.shouldDie() || defender.shouldDie()) finished = true;

        ++nextRound;
    }

    bool Combat::isFinished() const {
        return finished;
    }

    void Combat::finish(Game &game) {
        assert(isFinished());

        auto &attacker = game.getUnit(attackerID);
        auto &defender = game.getUnit(defenderID);

        if (attacker.shouldDie()) {
            game.deferKillUnit(attackerID);
        } else if (defender.shouldDie()) {
            game.deferKillUnit(defenderID);
        }
    }

    void Combat::paintUnits(NVGcontext *vg, const Game &game) const {

    }
}
