//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_CURSOR_H
#define RIPOSTE_CURSOR_H

#include <glm/vec2.hpp>
#include <GLFW/glfw3.h>

namespace rip {
    /**
     * Custom mouse cursor.
     */
    class Cursor {
        glm::vec2 pos;
        glm::vec2 windowSize;

    public:
        void tick(GLFWwindow *window);

        glm::vec2 getPos() const;
        glm::vec2 getWindowSize() const;
    };
}

#endif //RIPOSTE_CURSOR_H
