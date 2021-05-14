//
// Created by Caelum van Ispelen on 5/11/21.
//

#ifndef RIPOSTE_RIPMATH_H
#define RIPOSTE_RIPMATH_H

#include <cmath>
#include <glm/vec2.hpp>

namespace rip {
    constexpr double pi() { return 3.14159265358979323846264338327950288; }

    double dist(glm::uvec2 a, glm::uvec2 b);
}

#endif //RIPOSTE_RIPMATH_H
