//
// Created by Caelum van Ispelen on 5/24/21.
//

#ifndef RIPOSTE_COMBAT_H
#define RIPOSTE_COMBAT_H

#include "rng.h"
#include "ids.h"

struct NVGcontext;

namespace rip {
    class Game;

    // An ongoing combat event.
    class Combat {
        bool finished = false;
        UnitId attackerID;
        UnitId defenderID;
        float time = 0;
        int nextRound = 0;
        Rng rng;

        float getNextRoundTime() const;

    public:
        Combat(UnitId attacker, UnitId defender);

        // Advances combat by the given time.
        void advance(Game &game, float dt);

        // Determines whether combat has finished.
        bool isFinished() const;

        // Finishes combat by killing the loser (if needed).
        void finish(Game &game);

        // Paints both units in the combat event.
        void paintUnits(NVGcontext *vg, const Game &game) const;
    };
}

#endif //RIPOSTE_COMBAT_H
