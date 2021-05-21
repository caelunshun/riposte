//
// Created by Caelum van Ispelen on 5/20/21.
//

#include "yield.h"

namespace rip {
    Yield::Yield(int hammers, int commerce, int food) : hammers(hammers), commerce(commerce), food(food) {}

    Yield Yield::operator+(const Yield &other) const {
        return Yield(hammers + other.hammers, commerce + other.commerce, food + other.food);
    }

    void Yield::operator+=(const Yield &other) {
        hammers += other.hammers;
        commerce += other.commerce;
        food += other.food;
    }
}
