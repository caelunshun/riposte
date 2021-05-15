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

    // A smooth cosine interpolation between two points in 2D space.
    class SmoothAnimation {
        glm::vec2 fromPos;
        glm::vec2 targetPos;
        float time;
        float maxVel;
        float accelerateTime;

    public:
        SmoothAnimation(glm::vec2 fromPos, glm::vec2 targetPos, float maxVel, float accelerateTime);

        glm::vec2 getCurrentPos() const;
        void advance(float dt);
        bool isComplete();

        float getPosInternal() const;
    };
}

#endif //RIPOSTE_RIPMATH_H
