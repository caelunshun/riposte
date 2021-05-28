//
// Created by Caelum van Ispelen on 5/24/21.
//

#ifndef RIPOSTE_COMBAT_H
#define RIPOSTE_COMBAT_H

#include "rng.h"
#include "ids.h"

namespace rip {
    class Game;
    class Unit;

    // An ongoing combat event.
    class Combat {
        bool finished = false;
        UnitId attackerID;
        UnitId defenderID;
        float time = 0;
        int nextRound = 0;
        Rng rng;

        double startingAttackerStrength;
        double startingDefenderStrength;

        float getNextRoundTime() const;

        void doRound(Game &game);

        // Gets a unit's strength with all bonuses applied.
        double getUnitStrength(const Game &game, const Unit &unit, const Unit &opponent);

    public:
        Combat(UnitId attacker, UnitId defender, Game &game);

        // Advances combat by the given time.
        void advance(Game &game, float dt);

        // Determines whether combat has finished.
        bool isFinished() const;

        // Finishes combat by killing the loser (if needed).
        void finish(Game &game);

        UnitId getAttacker();
        UnitId getDefender();
    };
}

#endif //RIPOSTE_COMBAT_H
