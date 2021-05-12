//
// Created by Caelum van Ispelen on 5/12/21.
//

#include <algorithm>
#include "cursor.h"

namespace rip {
    void Cursor::tick(GLFWwindow *window) {
        double xPos, yPos;
        glfwGetCursorPos(window, &xPos, &yPos);

        int width, height;
        glfwGetWindowSize(window, &width, &height);
        xPos = std::clamp(xPos, 0.0, static_cast<double>(width));
        yPos = std::clamp(yPos, 0.0, static_cast<double>(height));

        glfwSetCursorPos(window, xPos, yPos);

        pos = glm::vec2(xPos, yPos);
        windowSize = glm::vec2(width, height);
    }

    glm::vec2 Cursor::getPos() const {
        return pos;
    }

    glm::vec2 Cursor::getWindowSize() const {
        return windowSize;
    }
}
