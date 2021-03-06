//
// Created by Caelum van Ispelen on 5/13/21.
//

#include "ripmath.h"
#include <glm/glm.hpp>
#include <algorithm>
#include <string>

namespace rip {
    double dist(glm::uvec2 a, glm::uvec2 b) {
        glm::vec2 af(a);
        glm::vec2 bf(b);
        return sqrt(pow(af.x - bf.x, 2) + pow(af.y - bf.y, 2));
    }

    std::array<glm::uvec2, 20> getBigFatCross(glm::uvec2 center) {
        std::array<glm::uvec2, 20> result;

        int i = 0;
        for (int dx = -2; dx <= 2; dx++) {
            for (int dy = -2; dy <= 2; dy++) {
                if (abs(dx) == 2 && abs(dy) == 2) {
                    continue;
                }
                if (dx == 0 && dy == 0) {
                    continue;
                }
                auto pos = glm::uvec2(glm::ivec2(dx, dy) + glm::ivec2(center));
                assert(i < 20);
                result[i++] = pos;
            }
        }

        return result;
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

    std::array<glm::uvec2, 4> getSideNeighbors(glm::uvec2 pos) {
        return {
            pos + glm::uvec2(1, 0),
            pos - glm::uvec2(1, 0),
            pos + glm::uvec2(0, 1),
            pos - glm::uvec2(0, 1),
        };
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

    const char *article(const std::string &noun) {
        if (noun.empty()) {
            return "a";
        }

        char c = tolower(noun[0]);
        if (c == 'a' || c == 'o' || c == 'u' || c == 'e' || c == 'i') {
            return "an";
        } else {
            return "a";
        }
    }

    bool isAdjacent(glm::uvec2 a, glm::uvec2 b) {
        return dist(a, b) < 1.9;
    }

    int percentOf(int amount, int percent) {
        return (amount * percent) / 100;
    }

    double cosineInterpolate(double y1, double y2, double time) {
        double mu2 = (1 - cos(time * pi())) / 2;
        return(y1 * (1 - mu2) + y2 * mu2);
    }
}

