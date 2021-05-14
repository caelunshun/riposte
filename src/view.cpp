//
// Created by Caelum van Ispelen on 5/12/21.
//

#include "view.h"
#include "ripmath.h"

#include <cmath>
#include <algorithm>

namespace rip {
    const auto DIR_RIGHT = 0x01;
    const auto DIR_LEFT = 0x02;
    const auto DIR_UP = 0x04;
    const auto DIR_DOWN = 0x10;

    float sampleVelocityCurve(float time) {
        const auto cutoff = 1;
        const auto max = 300;
        if (time >= cutoff) {
            return max;
        } else {
            return -(max / 2) * std::cos(time / (0.1 * pi())) + (max / 2);
        }
    }

    void View::tick(float dt, const Cursor &cursor) {
        const auto threshold = 2;
        const auto cPos = cursor.getPos();
        const auto wSize = cursor.getWindowSize();

        const auto oldMoveDir = moveDir;
        moveDir = 0;

        if (fabs(cPos.x - wSize.x) <= threshold) {
            moveDir |= DIR_RIGHT;
        } else if (fabs(cPos.x) <= threshold) {
            moveDir |= DIR_LEFT;
        }

        if (fabs(cPos.y - wSize.y) <= threshold) {
            moveDir |= DIR_DOWN;
        } else if (fabs(cPos.y) <= threshold) {
            moveDir |= DIR_UP;
        }

        if (moveDir != oldMoveDir) {
            moveTime = 0;
        }

        if (moveDir == 0) {
            centerVelocity *= powf(0.02, dt);
        }

        float speed = sampleVelocityCurve(moveTime);
        if (moveDir & DIR_RIGHT) {
            centerVelocity.x = speed;
        } else if (moveDir & DIR_LEFT) {
            centerVelocity.x = -speed;
        }

        if (moveDir & DIR_DOWN) {
            centerVelocity.y = speed;
        } else if (moveDir & DIR_UP) {
            centerVelocity.y = -speed;
        }

        moveTime += dt;
        mapCenter += (centerVelocity * (1 / zoomFactor)) * dt;
    }

    glm::vec2 View::getMapCenter() const {
        return mapCenter;
    }

    float View::getZoomFactor() const {
        return zoomFactor;
    }

    void View::setMapCenter(glm::vec2 pos) {
        mapCenter = pos;
    }
}
