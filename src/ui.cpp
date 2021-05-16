//
// Created by Caelum van Ispelen on 5/14/21.
//

#include <GL/glew.h>
#include "ui.h"

#define NK_INCLUDE_STANDARD_IO
#define NK_INCLUDE_STANDARD_VARARGS
#define NK_INCLUDE_DEFAULT_ALLOCATOR
#define NK_INCLUDE_VERTEX_BUFFER_OUTPUT
#define NK_INCLUDE_FONT_BAKING
#define NK_IMPLEMENTATION
#define NK_KEYSTATE_BASED_INPUT
#include <nuklear.h>

#define NK_GLFW_GL3_IMPLEMENTATION
#include "nuklear_glfw_gl3.h"

namespace rip {
    void ui_mouse_callback(GLFWwindow *window, int button, int action, int mods) {
        nk_glfw3_mouse_button_callback(window, button, action, mods);
    }

    struct Ui::impl {
        nk_context *nk;
        nk_font_atlas *fontAtlas;
        std::unique_ptr<nk_glfw> nkGlfw;

        impl(nk_context *nk, nk_font_atlas *fontAtlas, std::unique_ptr<nk_glfw> nkGlfw) : nk(nk), fontAtlas(fontAtlas), nkGlfw(std::move(nkGlfw)) {}
    };

    Ui::Ui(GLFWwindow *window) {
        auto nkGlfw = std::make_unique<nk_glfw>();

        nk_context *nk = nk_glfw3_init(&*nkGlfw, window, NK_GLFW3_DEFAULT);

        nk_font_atlas *atlas;
        nk_glfw3_font_stash_begin(&*nkGlfw, &atlas);
        nk_font *defaultFont = nk_font_atlas_add_from_file(atlas, "assets/font/Merriweather-Regular.ttf", 18, 0);
        nk_glfw3_font_stash_end(&*nkGlfw);
        nk_style_set_font(nk, &defaultFont->handle);
        nk_style_load_all_cursors(nk, atlas->cursors);
        nk_style_hide_cursor(nk);

        glfwSetScrollCallback(window, nk_gflw3_scroll_callback);
        glfwSetCharCallback(window, nk_glfw3_char_callback);

        _impl = std::make_unique<impl>(nk, atlas, std::move(nkGlfw));
    }

    void Ui::render() {
        nk_glfw3_render(&*_impl->nkGlfw, NK_ANTI_ALIASING_ON, 1024 * 1024, 1024 * 1024);
    }

    Ui::~Ui() = default;

    nk_context *Ui::getNk() {
        return _impl->nk;
    }

    void Ui::begin() {
        nk_glfw3_new_frame(&*_impl->nkGlfw);
    }
}
