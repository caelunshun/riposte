//
// Created by Caelum van Ispelen on 5/13/21.
//

#include "ripmath.h"
#include <glm/glm.hpp>
#include <algorithm>

namespace rip {
    double dist(glm::uvec2 a, glm::uvec2 b) {
        glm::vec2 af(a);
        glm::vec2 bf(b);
        return sqrt(pow(af.x - bf.x, 2) + pow(af.y - bf.y, 2));
    }

    SmoothAnimation::SmoothAnimation(glm::vec2 fromPos, glm::vec2 targetPos, float maxVel, float accelerateTime)
    : fromPos(fromPos), targetPos(targetPos), maxVel(maxVel), time(0), accelerateTime(accelerateTime) {}

    std::array<glm::uvec2, 8> getNeighbors(glm::uvec2 pos) {
        glm::ivec2 offsets[8] = {
                {1,0},
                {1,1},
                {0,1},
                {-1,1},
                {-1,0},
                {-1,-1},
                {0,-1},
                {1,-1},
        };

        std::array<glm::uvec2, 8> result;
        for (int i = 0; i < result.size(); i++) {
            result[i] = glm::uvec2(glm::ivec2(pos) + offsets[i]);
        }
        return result;
    }

    static float evaluateAnimationIntegral(float maxVel, float t, float accelerateTime) {
        return maxVel / pi() * -cos((1.0f / accelerateTime) * pi() * t) + (maxVel / pi());
    }

    float SmoothAnimation::getPosInternal() const {
        // The velocity function v(t) is ksin(a*pi*t), where k is the maximum velocity
        // and a is accelerationTime.
        // After t>accelerationTime, the velocity is set to k.
        //
        // This function computes the definite integral of v(t) between 0 and the current time
        // to determine the position.

        float pos;
        if (time <= accelerateTime) {
            pos = evaluateAnimationIntegral(maxVel, time, accelerateTime);
        } else {
            pos = evaluateAnimationIntegral(maxVel, accelerateTime, accelerateTime) + maxVel * (time - accelerateTime);
        }
        return pos;
    }

    glm::vec2 SmoothAnimation::getCurrentPos() const {
        if (glm::distance(targetPos, fromPos) <= 0.1) {
            return targetPos;
        }

        auto pos = getPosInternal();
        pos = std::clamp(pos, 0.0f, glm::distance(fromPos, targetPos));

        auto ray = glm::normalize(targetPos - fromPos);
        return fromPos + ray * pos;
    }

    void SmoothAnimation::advance(float dt) {
        time += dt;
    }

    bool SmoothAnimation::isComplete() {
        return getPosInternal() >= glm::distance(fromPos, targetPos);
    }
}

