//
// Created by Caelum van Ispelen on 5/14/21.
//

#ifndef RIPOSTE_UI_H
#define RIPOSTE_UI_H

#include <memory>
#include <GLFW/glfw3.h>

struct nk_context;

namespace rip {
    void ui_mouse_callback(GLFWwindow *window, int button, int action, int mods);
    void ui_scroll_callback(GLFWwindow *window, double offsetX, double offsetY);

    enum MouseButton {
        Right,
        Middle,
        Left,
    };

    enum MouseAction {
        Press,
        Release,
    };

    struct MouseEvent {
        MouseButton button;
        MouseAction action;

        MouseEvent(MouseButton button, MouseAction action) : button(button), action(action) {}
    };

    class Ui {
        class impl;
        std::unique_ptr<impl> _impl;

    public:
        Ui(GLFWwindow *window);

        nk_context *getNk();

        void begin();
        void render();

        ~Ui();
        Ui(Ui &&other) = delete;
    };
}


#endif //RIPOSTE_UI_H
