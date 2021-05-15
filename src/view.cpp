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

        if (!(moveDir & DIR_RIGHT || moveDir & DIR_LEFT)) {
            centerVelocity.x *= powf(0.02, dt);
            moveTime.x = 0;
        }
        if (!(moveDir & DIR_DOWN || moveDir & DIR_UP)) {
            centerVelocity.y *= powf(0.02, dt);
            moveTime.y = 0;
        }

        float speedX = sampleVelocityCurve(moveTime.x);
        float speedY = sampleVelocityCurve(moveTime.y);

        if (moveDir & DIR_RIGHT) {
            centerVelocity.x = speedX;
        } else if (moveDir & DIR_LEFT) {
            centerVelocity.x = -speedX;
        }

        if (moveDir & DIR_DOWN) {
            centerVelocity.y = speedY;
        } else if (moveDir & DIR_UP) {
            centerVelocity.y = -speedY;
        }

        moveTime += dt;
        mapCenter += (centerVelocity * (1 / zoomFactor)) * dt;

        if (centerAnimation.has_value()) {
            mapCenter = centerAnimation->getCurrentPos();
            centerAnimation->advance(dt);
            if (centerAnimation->isComplete()) {
                centerAnimation = std::optional<SmoothAnimation>();
            }
        }
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

    void View::setCenterAnimation(SmoothAnimation animation) {
        centerAnimation = std::make_optional<SmoothAnimation>(animation);
    }
}
