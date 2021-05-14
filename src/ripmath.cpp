//
// Created by Caelum van Ispelen on 5/13/21.
//

#include "ripmath.h"

namespace rip {
    double dist(glm::uvec2 a, glm::uvec2 b) {
        return sqrt(pow(static_cast<double>(b.x - a.x), 2) + pow(static_cast<double>(b.y - a.y), 2));
    }
}

