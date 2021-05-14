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

        nk_color table[NK_COLOR_COUNT];
        table[NK_COLOR_TEXT] = nk_rgba(20, 20, 20, 255);
        table[NK_COLOR_WINDOW] = nk_rgba(202, 212, 214, 230);
        table[NK_COLOR_HEADER] = nk_rgba(137, 182, 224, 220);
        table[NK_COLOR_BORDER] = nk_rgba(140, 159, 173, 255);
        table[NK_COLOR_BUTTON] = nk_rgba(137, 182, 224, 255);
        table[NK_COLOR_BUTTON_HOVER] = nk_rgba(142, 187, 229, 255);
        table[NK_COLOR_BUTTON_ACTIVE] = nk_rgba(147, 192, 234, 255);
        table[NK_COLOR_TOGGLE] = nk_rgba(177, 210, 210, 255);
        table[NK_COLOR_TOGGLE_HOVER] = nk_rgba(182, 215, 215, 255);
        table[NK_COLOR_TOGGLE_CURSOR] = nk_rgba(137, 182, 224, 255);
        table[NK_COLOR_SELECT] = nk_rgba(177, 210, 210, 255);
        table[NK_COLOR_SELECT_ACTIVE] = nk_rgba(137, 182, 224, 255);
        table[NK_COLOR_SLIDER] = nk_rgba(177, 210, 210, 255);
        table[NK_COLOR_SLIDER_CURSOR] = nk_rgba(137, 182, 224, 245);
        table[NK_COLOR_SLIDER_CURSOR_HOVER] = nk_rgba(142, 188, 229, 255);
        table[NK_COLOR_SLIDER_CURSOR_ACTIVE] = nk_rgba(147, 193, 234, 255);
        table[NK_COLOR_PROPERTY] = nk_rgba(210, 210, 210, 255);
        table[NK_COLOR_EDIT] = nk_rgba(210, 210, 210, 225);
        table[NK_COLOR_EDIT_CURSOR] = nk_rgba(20, 20, 20, 255);
        table[NK_COLOR_COMBO] = nk_rgba(210, 210, 210, 255);
        table[NK_COLOR_CHART] = nk_rgba(210, 210, 210, 255);
        table[NK_COLOR_CHART_COLOR] = nk_rgba(137, 182, 224, 255);
        table[NK_COLOR_CHART_COLOR_HIGHLIGHT] = nk_rgba( 255, 0, 0, 255);
        table[NK_COLOR_SCROLLBAR] = nk_rgba(190, 200, 200, 255);
        table[NK_COLOR_SCROLLBAR_CURSOR] = nk_rgba(64, 84, 95, 255);
        table[NK_COLOR_SCROLLBAR_CURSOR_HOVER] = nk_rgba(70, 90, 100, 255);
        table[NK_COLOR_SCROLLBAR_CURSOR_ACTIVE] = nk_rgba(75, 95, 105, 255);
        table[NK_COLOR_TAB_HEADER] = nk_rgba(156, 193, 220, 255);
        nk_style_from_table(nk, table);

        nk_font_atlas *atlas;
        nk_glfw3_font_stash_begin(&*nkGlfw, &atlas);
        nk_font *defaultFont = nk_font_atlas_add_from_file(atlas, "assets/font/Merriweather-Regular.ttf", 14, 0);
        nk_glfw3_font_stash_end(&*nkGlfw);
        nk_style_set_font(nk, &defaultFont->handle);
        nk_style_load_all_cursors(nk, atlas->cursors);

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
