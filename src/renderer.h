//
// Created by Caelum van Ispelen on 5/11/21.
//

#ifndef RIPOSTE_RENDERER_H
#define RIPOSTE_RENDERER_H

#include <GL/glew.h>
#define NANOVG_GL3_IMPLEMENTATION
#include <nanovg.h>
#include <nanovg_gl.h>

#include <GLFW/glfw3.h>
#include "game.h"

namespace rip {
    class Painter {
    public:
        virtual ~Painter();
        virtual void paint(NVGcontext *vg, Game &game);
    };

    class Renderer {
        NVGcontext *vg;
        std::vector<std::unique_ptr<Painter>> painters;

    public:
        explicit Renderer(GLFWwindow *window) {
            vg = nvgCreateGL3(NVG_ANTIALIAS | NVG_STENCIL_STROKES);
        }

        void paint(Game &game) {
            for (auto &painter : painters) {
                painter->paint(vg, game);
            }
        }

        ~Renderer() {
            nvgDeleteGL3(vg);
        }
    };
}

#endif //RIPOSTE_RENDERER_H
