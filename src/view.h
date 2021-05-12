//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_VIEW_H
#define RIPOSTE_VIEW_H

#include <glm/vec2.hpp>
#include "cursor.h"

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

        float moveTime = 0;
        uint32_t moveDir = 0;

        float zoomVelocity;
        glm::vec2 centerVelocity;

    public:
        View() : mapCenter(500, 500), zoomFactor(1), zoomVelocity(0), centerVelocity(0, 0) {}

        void tick(float dt, const Cursor &cursor);

        glm::vec2 getMapCenter() const;

        float getZoomFactor() const;
    };
}

#endif //RIPOSTE_VIEW_H
