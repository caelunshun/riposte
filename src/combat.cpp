//
// Created by Caelum van Ispelen on 5/24/21.
//

#include <cassert>
#include "combat.h"
#include "game.h"
#include "unit.h"
#include "event.h"

namespace rip {
    const float roundTime = 0.4;

    Combat::Combat(UnitId attacker, UnitId defender, const Game &game) : attackerID(attacker), defenderID(defender) {
        assert(attacker != defender);
        startingAttackerStrength = game.getUnit(attacker).getCombatStrength();
        startingDefenderStrength = game.getUnit(defender).getCombatStrength();

        if (startingDefenderStrength == 0 || startingAttackerStrength == 0) {
            finished = true;
        }
    }

    float Combat::getNextRoundTime() const {
        return nextRound * roundTime;
    }

    void Combat::doRound(Game &game) {
        auto &attacker = game.getUnit(attackerID);
        auto &defender = game.getUnit(defenderID);

        double r = startingAttackerStrength / startingDefenderStrength;

        double attackerDamage = 20.0 * (3 * r + 1) / (3 + r) / 100.0;
        double defenderDamage = 20.0 * (3 + r) / (3 * r + 1) / 100.0;

        bool attackerWon = rng.chance(r / (1 + r));
        if (attackerWon) {
            auto newHealth = defender.getHealth() - attackerDamage;
            defender.setHealth(newHealth);
        } else {
            auto newHealth = attacker.getHealth() - defenderDamage;
            attacker.setHealth(newHealth);
        }

        if (attacker.shouldDie() || defender.shouldDie()) finished = true;

        ++nextRound;
    }

    void Combat::advance(Game &game, float dt) {
        time += dt;

        if (getNextRoundTime() > time) return;

        if (finished) return;

        doRound(game);
    }

    bool Combat::isFinished() const {
        return finished;
    }

    void Combat::finish(Game &game) {
        while (!isFinished()) {
            doRound(game);
        }

        auto &attacker = game.getUnit(attackerID);
        auto &defender = game.getUnit(defenderID);

        attacker.setInCombat(false);
        defender.setInCombat(false);

        UnitId winner;

        if (attacker.shouldDie() || attacker.getCombatStrength() == 0) {
            game.deferKillUnit(attackerID);
            winner = defenderID;
        } else if (defender.shouldDie() || defender.getCombatStrength() == 0) {
            game.deferKillUnit(defenderID);
            attacker.moveTo(defender.getPos(), game);
            winner = attackerID;
        }

        if (attackerID == game.getThePlayerID() || defenderID == game.getThePlayerID()) {
            UnitId enemy;
            UnitId ours;
            if (attackerID == game.getThePlayerID()) { enemy = defenderID; ours = attackerID; }
            else { enemy = attackerID; ours = defenderID; }
            game.addEvent(std::make_unique<CombatEvent>(
                        winner == game.getThePlayerID(),
                        game.getPlayer(game.getUnit(enemy).getOwner()).getCiv().adjective,
                        game.getUnit(ours).getKind().name,
                        game.getUnit(enemy).getKind().name
                    ));
        }
    }

    UnitId Combat::getAttacker() {
        return attackerID;
    }

    UnitId Combat::getDefender() {
        return defenderID;
    }
}
