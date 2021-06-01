//
// Created by Caelum van Ispelen on 6/1/21.
//

#include <sol/sol.hpp>
#include <nuklear.h>
#include <glm/vec2.hpp>
#include <nanovg.h>

#include "../renderer.h"
#include "../script.h"

namespace rip {
    static NVGcolor luaColor(sol::table color) {
        if (color.size() == 3) {
            return nvgRGB(color.get<int>(1), color.get<int>(2), color.get<int>(3));
        } else {
            return nvgRGBA(color.get<int>(1), color.get<int>(2), color.get<int>(3), color.get<int>(4));
        }
    }

    void bindOther(sol::state &lua, std::shared_ptr<Game*> game) {
        auto nk_type = lua.new_usertype<nk_context>("NuklearContext");
        nk_type["beginWindow"] = [=](nk_context *nk, std::string title, float posX, float posY, float sizeX, float sizeY) {
            nk_begin(nk, title.c_str(), nk_rect(posX, posY, sizeX, sizeY), 0);
        };
        nk_type["endWindow"] = [=](nk_context *nk) {
            nk_end(nk);
        };
        nk_type["layoutDynamic"] = [=](nk_context *nk, float height, int cols) {
            nk_layout_row_dynamic(nk, height, cols);
        };
        nk_type["spacing"] = [=](nk_context *nk, int cols) {
            nk_spacing(nk, cols);
        };
        nk_type["label"] = [=](nk_context *nk, const std::string &text) {
            nk_label(nk, text.c_str(), NK_TEXT_ALIGN_LEFT);
        };
        nk_type["labelWrap"] = [=](nk_context *nk, const std::string &text) {
            nk_label_wrap(nk, text.c_str());
        };
        nk_type["buttonLabel"] = [=](nk_context *nk, const std::string &text) {
            return nk_button_label(nk, text.c_str()) != 0;
        };

        auto vec2_type = lua.new_usertype<glm::vec2>("Vec2", sol::constructors<glm::vec2(float, float)>());
        vec2_type["x"] = &glm::vec2::x;
        vec2_type["y"] = &glm::vec2::y;

        auto uvec2_type = lua.new_usertype<glm::uvec2>("UVec2", sol::constructors<glm::uvec2(uint32_t, uint32_t)>());
        uvec2_type["x"] = &glm::uvec2::x;
        uvec2_type["y"] = &glm::uvec2::y;



        auto cv_type = lua.new_usertype<Canvas>("Canvas");
        cv_type["beginPath"] = [] (Canvas &cv) {
            nvgBeginPath(cv.vg);
        };
        cv_type["rect"] = [] (Canvas &cv, float posX, float posY, float sizeX, float sizeY) {
            nvgRect(cv.vg, posX, posY, sizeX, sizeY);
        };
        cv_type["circle"] = [] (Canvas &cv, float cx, float cy, float radius) {
            nvgCircle(cv.vg, cx, cy, radius);
        };
        cv_type["lineTo"] = [] (Canvas &cv, float x, float y) {
            nvgLineTo(cv.vg, x, y);
        };
        cv_type["fillColor"] = [] (Canvas &cv, sol::table color) {
            nvgFillColor(cv.vg, luaColor(color));
        };
        cv_type["strokeColor"] = [] (Canvas &cv, sol::table color) {
            nvgStrokeColor(cv.vg, luaColor(color));
        };
        cv_type["strokeWidth"] = [] (Canvas &cv, float width) {
            nvgStrokeWidth(cv.vg, width);
        };
        cv_type["fill"] = [] (Canvas &cv) {
            nvgFill(cv.vg);
        };
        cv_type["stroke"] = [] (Canvas &cv) {
            nvgStroke(cv.vg);
        };
        cv_type["textFormat"] = [] (Canvas &cv, int baseline, int align) {
            nvgTextAlign(cv.vg, baseline | align);
        };
        cv_type["fontSize"] = [] (Canvas &cv, float size) {
            nvgFontSize(cv.vg, size);
        };
        cv_type["text"] = [] (Canvas &cv, float x, float y, const std::string &text) {
            nvgText(cv.vg, x, y, text.c_str(), nullptr);
        };
        cv_type["applyZoom"] = [=] (Canvas &cv) {
            scale(cv.vg, **game);
        };
        cv_type["removeZoom"] = [=] (Canvas &cv) {
            nvgResetTransform(cv.vg);
        };

        lua["TextBaseline"] = lua.create_table_with(
                "Alphabetic", NVG_ALIGN_BASELINE,
                "Top", NVG_ALIGN_TOP,
                "Bottom", NVG_ALIGN_BOTTOM,
                "Middle", NVG_ALIGN_MIDDLE
        );
        lua["TextAlign"] = lua.create_table_with(
                "Left", NVG_ALIGN_LEFT,
                "Center", NVG_ALIGN_CENTER,
                "Right", NVG_ALIGN_RIGHT
        );

        lua["Key"] = lua.create_table_with(
                "Escape", GLFW_KEY_ESCAPE,
                "Enter", GLFW_KEY_ENTER,
                "LeftShift", GLFW_KEY_LEFT_SHIFT,
                "RightShift", GLFW_KEY_RIGHT_SHIFT,
                "Control", GLFW_KEY_LEFT_CONTROL,
                "Alt", GLFW_KEY_LEFT_ALT,
                "Tab", GLFW_KEY_TAB
        );
    }
}
