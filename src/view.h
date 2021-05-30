//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_VIEW_H
#define RIPOSTE_VIEW_H

#include <glm/vec2.hpp>
#include <optional>
#include "cursor.h"
#include "ripmath.h"

namespace rip {
    /**
     * Stores the client's view information on the game, like zoom
     * and position.
     */
    class View {
        /**
         * The center of the map in world space (1 tile = 100 units).
         * The zoom factor does not affect this coordinate space.
         */
        glm::vec2 mapCenter;
        /**
         * Scale factor to apply.
         */
        float zoomFactor;

        glm::vec2 moveTime = glm::vec2(0);
        uint32_t moveDir = 0;

        glm::vec2 centerVelocity;

        // Used to animate the view position when it is moved programatically.
        std::optional<SmoothAnimation> centerAnimation;

    public:
        View() : mapCenter(500, 500), zoomFactor(1), centerVelocity(0, 0) {}

        void tick(float dt, const Cursor &cursor, bool hudHasFocus);

        void handleScroll(double offsetY);

        glm::vec2 getMapCenter() const;

        float getZoomFactor() const;

        void setMapCenter(glm::vec2 pos);

        void setCenterAnimation(SmoothAnimation animation);
    };
}

#endif //RIPOSTE_VIEW_H
