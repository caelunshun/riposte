//
// Created by Caelum van Ispelen on 5/11/21.
//

#ifndef RIPOSTE_RENDERER_H
#define RIPOSTE_RENDERER_H

#include <GL/glew.h>
#include <nanovg.h>
#include <memory>

#include <GLFW/glfw3.h>
#include "game.h"
#include "assets.h"

namespace rip {
    // Applies scaling based on the zoom factor.
    void scale(NVGcontext *vg, const Game &game);

    class Painter {
    public:
        virtual void paint(NVGcontext *vg, Game &game) = 0;
        virtual ~Painter() {}
    };

    class Renderer {
        NVGcontext *vg;
        GLFWwindow *window;
        std::vector<std::unique_ptr<Painter>> gamePainters;
        std::vector<std::unique_ptr<Painter>> overlayPainters;

    public:
        explicit Renderer(GLFWwindow *window);

        void init(const std::shared_ptr<Assets>& assets);

        void begin(bool clear) {
            int width, height, fbWidth, fbHeight;
            glfwGetWindowSize(window, &width, &height);
            glfwGetFramebufferSize(window, &fbWidth, &fbHeight);
            auto scaleFactor = static_cast<float>(fbWidth) / width;

            glViewport(0, 0, fbWidth, fbHeight);
            if (clear) {
                glClearColor(0, 0, 0, 0);
                glClear(GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT | GL_COLOR_BUFFER_BIT);
            }

            nvgBeginFrame(vg, width, height, scaleFactor);
        }

        void end() {
            nvgEndFrame(vg);
        }

        void paintGame(Game &game) {
            scale(vg, game);
            for (auto &painter : gamePainters) {
                painter->paint(vg, game);
            }
            nvgResetTransform(vg);
        }

        void paintOverlays(Game &game) {
            for (auto &painter : overlayPainters) {
                painter->paint(vg, game);
            }
        }

        NVGcontext *getNvg() const {
            return vg;
        }

        ~Renderer();
    };

    class Image : public Asset {
    public:
        int id;

        explicit Image(int id) : id(id) {}
    };

    class Font : public Asset {
    public:
        int id;

        explicit Font(int id) : id(id) {}
    };
}

#endif //RIPOSTE_RENDERER_H
