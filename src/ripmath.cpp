//
// Created by Caelum van Ispelen on 5/13/21.
//

#include "ripmath.h"

namespace rip {
    double dist(glm::uvec2 a, glm::uvec2 b) {
        glm::vec2 af(a);
        glm::vec2 bf(b);
        return sqrt(pow(af.x - bf.x, 2) + pow(af.y - bf.y, 2));
    }
}

