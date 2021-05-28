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

    Combat::Combat(UnitId attackerID, UnitId defenderID, const Game &game) : attackerID(attackerID), defenderID(defenderID) {
        assert(attackerID != defenderID);
        const auto &attacker = game.getUnit(attackerID);
        const auto &defender = game.getUnit(defenderID);
        startingAttackerStrength = getUnitStrength(game, attacker, defender);
        startingDefenderStrength = getUnitStrength(game, defender, attacker);

        if (startingDefenderStrength == 0 || startingAttackerStrength == 0) {
            finished = true;
        }
    }

    double Combat::getUnitStrength(const Game &game, const Unit &unit, const Unit &opponent) {
        double baseStrength = unit.getCombatStrength();

        bool defending = unit.getID() == defenderID;
        bool attacking = unit.getID() == attackerID;
        assert(defending || attacking);

        // Apply a percent bonus to the base strength,
        // based on the unit's combatBonuses.
        int percentBonus = 0;
        for (const CombatBonus &bonus : unit.getKind().combatBonuses) {
            if ((defending && bonus.onlyOnAttack) || (attacking && bonus.onlyOnDefense)) {
                continue;
            }
            if (game.getCityAtLocation(unit.getPos())) {
                percentBonus += bonus.whenInCityBonus;
            }
            if (opponent.getKind().id == bonus.unit) {
                percentBonus += bonus.againstUnitBonus;
            }
            if (opponent.getKind().category == bonus.unitCategory) {
                percentBonus += bonus.againstUnitCategoryBonus;
            }
        }

        return baseStrength + (percentBonus / 100.0) * baseStrength;
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

        if (attacker.getOwner() == game.getThePlayerID() || defender.getOwner() == game.getThePlayerID()) {
            UnitId enemy;
            UnitId ours;
            if (attacker.getOwner() == game.getThePlayerID()) { enemy = defenderID; ours = attackerID; }
            else { enemy = attackerID; ours = defenderID; }
            game.addEvent(std::make_unique<CombatEvent>(
                        game.getUnit(winner).getOwner() == game.getThePlayerID(),
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
