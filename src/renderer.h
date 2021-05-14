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
    class Painter {
    public:
        virtual void paint(NVGcontext *vg, Game &game) = 0;
        virtual ~Painter() {}
    };

    class Renderer {
        NVGcontext *vg;
        GLFWwindow *window;
        std::vector<std::unique_ptr<Painter>> painters;

    public:
        explicit Renderer(GLFWwindow *window);

        void init(const std::shared_ptr<Assets>& assets);

        void paint(Game &game) {
            int width, height, fbWidth, fbHeight;
            glfwGetWindowSize(window, &width, &height);
            glfwGetFramebufferSize(window, &fbWidth, &fbHeight);
            auto scaleFactor = static_cast<float>(fbWidth) / width;

            glViewport(0, 0, fbWidth, fbHeight);
            glClearColor(0, 0, 0, 0);
            glClear(GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT | GL_COLOR_BUFFER_BIT);

            nvgBeginFrame(vg, width, height, scaleFactor);
            for (auto &painter : painters) {
                painter->paint(vg, game);
            }
            nvgEndFrame(vg);

            glfwSwapBuffers(window);
            glfwPollEvents();
        }

        NVGcontext *getNvg() const {
            return vg;
        }

        ~Renderer();
    };

    class ImageLoader : public AssetLoader {
        NVGcontext *vg;
    public:
        explicit ImageLoader(const Renderer &renderer) : vg(renderer.getNvg()) {}
        std::shared_ptr<Asset> loadAsset(const std::string &data) override;
    };

    class Image : public Asset {
    public:
        int id;

        explicit Image(int id) : id(id) {}
    };

    class FontLoader : public AssetLoader {
        NVGcontext *vg;
    public:
        explicit FontLoader(const Renderer &renderer) : vg(renderer.getNvg()) {}
        std::shared_ptr<Asset> loadAsset(const std::string &data) override;
    };

    class Font : public Asset {
    public:
        int id;

        explicit Font(int id) : id(id) {}
    };
}

#endif //RIPOSTE_RENDERER_H
