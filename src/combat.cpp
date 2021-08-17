//
// Created by Caelum van Ispelen on 5/24/21.
//

#include <cassert>
#include "combat.h"
#include "game.h"
#include "unit.h"
#include "event.h"
#include "stack.h"
#include "city.h"
#include <riposte.pb.h>
#include "server.h"
#include "tile.h"

namespace rip {
    const float roundTime = 0.4;

    Combat::Combat(UnitId attackerID, UnitId defenderID, Game &game) : attackerID(attackerID), defenderID(defenderID) {
        assert(attackerID != defenderID);
        auto &attacker = game.getUnit(attackerID);
        auto &defender = game.getUnit(defenderID);
        startingAttackerStrength = getUnitStrength(game, attacker, defender);
        startingDefenderStrength = getUnitStrength(game, defender, attacker);

        if (startingDefenderStrength == 0 || startingAttackerStrength == 0) {
            finished = true;

            if (startingAttackerStrength == 0) {
                attacker.setHealth(0);
            }
            if (startingDefenderStrength == 0)
                defender.setHealth(0);
        }

        const auto &defendingStack = game.getStack(*game.getStackByKey(defender.getOwner(), defender.getPos()));
        const auto numCollateralTargets = std::min((int) defendingStack.getUnits().size() - 1,
                                                   attacker.getKind().maxCollateralTargets);

        while (collateralDamageTargets.size() < numCollateralTargets) {
            const auto unitID = defendingStack.getUnits()[rng.u32(0, defendingStack.getUnits().size())];
            if (unitID != defenderID) {
                collateralDamageTargets.insert(unitID);
            }
        }
    }

    double Combat::getUnitStrength(const Game &game, const Unit &unit, const Unit &opponent) {
        double baseStrength = unit.getCombatStrength();

        bool defending = unit.getID() == defenderID;
        bool attacking = unit.getID() == attackerID;
        assert(defending || attacking);

        const auto *city = game.getCityAtLocation(unit.getPos());

        // Apply a percent bonus to the base strength,
        // based on the unit's combatBonuses.
        int percentBonus = 0;
        for (const CombatBonus &bonus : unit.getKind().combatBonuses) {
            if ((defending && bonus.onlyOnAttack) || (attacking && bonus.onlyOnDefense)) {
                continue;
            }
            if (city) {
                percentBonus += bonus.whenInCityBonus;
            }
            if (opponent.getKind().id == bonus.unit) {
                percentBonus += bonus.againstUnitBonus;
            }
            if (opponent.getKind().category == bonus.unitCategory) {
                percentBonus += bonus.againstUnitCategoryBonus;
            }
        }

        if (city && city->getOwner() == unit.getOwner() && defending) {
            percentBonus += city->getBuildingEffects().defenseBonusPercent;
            percentBonus += city->getCultureDefenseBonus();
        }

        percentBonus += game.getTile(unit.getPos()).getDefensiveBonus();

        return baseStrength + (percentBonus / 100.0) * baseStrength;
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

        CombatRound round;
        round.set_attackerhealth(attacker.getHealth());
        round.set_defenderhealth(defender.getHealth());
        rounds.push_back(std::move(round));

        ++nextRound;
    }

    bool Combat::isFinished() const {
        return finished;
    }

    void Combat::finish(Game &game) {
        while (!isFinished()) {
            doRound(game);
        }

        doCollateralDamage(game);

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
            attacker.moveTo(defender.getPos(), game, false);
            winner = attackerID;
        }

        if (attacker.getOwner() == game.getThePlayerID() || defender.getOwner() == game.getThePlayerID()) {
            UnitId enemy;
            UnitId ours;
            if (attacker.getOwner() == game.getThePlayerID()) { enemy = defenderID; ours = attackerID; }
            else { enemy = attackerID; ours = defenderID; }
            game.getServer().broadcastCombatEvent(
                    attacker.getID(),
                    defender.getID(),
                    winner,
                    getRounds(),
                    collateralDamageTargets.size()
                    );
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

    const std::vector<CombatRound> &Combat::getRounds() const {
        return rounds;
    }

    void Combat::doCollateralDamage(Game &game) {
        auto &attacker = game.getUnit(attackerID);
        for (int i = 0; i < collateralDamageTargets.size(); i++) {
            auto start = collateralDamageTargets.cbegin();
            std::advance(start, rng.u32(0, collateralDamageTargets.size()));
            const auto targetID = *start;

            auto &target = game.getUnit(targetID);

            // intentionally not using getCombatStrength()
            const auto a = attacker.getHealth() * attacker.getKind().strength;
            const auto d = target.getHealth() * target.getKind().strength;

            const auto damage = 0.1 * (3 * a + d) / (3 * d + a);
            target.setHealth(target.getHealth() - damage);

            game.getServer().markUnitDirty(targetID);
        }
    }
}
