#include "assets.h"
#include "renderer.h"
#include "mapgen.h"
#include "registry.h"
#include "hud.h"
#include "ui.h"

#include <deque>

static std::deque<rip::MouseEvent> mouseEvents;

static void mouse_callback(GLFWwindow *window, int button, int action, int mods) {
    rip::ui_mouse_callback(window, button, action, mods);

    rip::MouseButton b;
    rip::MouseAction a;

    switch (button) {
        case GLFW_MOUSE_BUTTON_LEFT:
            b = rip::MouseButton::Left;
            break;
        case GLFW_MOUSE_BUTTON_MIDDLE:
            b = rip::MouseButton::Middle;
            break;
        case GLFW_MOUSE_BUTTON_RIGHT:
            b = rip::MouseButton::Right;
            break;
        default:
            return;
    }

    switch (action) {
        case GLFW_PRESS:
            a = rip::MouseAction::Press;
            break;
        case GLFW_RELEASE:
            a = rip::MouseAction::Release;
            break;
        default:
            return;
    }

    rip::MouseEvent event(b, a);
    mouseEvents.push_back(event);
}

static void error_callback(int error, const char* description)
{
    fprintf(stderr, "Error: %s\n", description);
}

int main() {
    glfwSetErrorCallback(error_callback);
    glfwInit();
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 2);
    glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
    auto window = glfwCreateWindow(1920 / 2, 1080 / 2, "Riposte", nullptr, nullptr);
    if (!window) {
        glfwTerminate();
        return 1;
    }
    glfwMakeContextCurrent(window);
    glfwSetTime(0);

    glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
    glfwSetMouseButtonCallback(window, mouse_callback);

    glewExperimental = true;

    if (glewInit() != GLEW_OK) {
        return -1;
    }

    glGetError();

    rip::Renderer renderer(window);
    auto registry = std::make_shared<rip::Registry>();

    auto assets = std::make_shared<rip::Assets>();
    assets->addLoader("image", std::make_unique<rip::ImageLoader>(renderer));
    assets->addLoader("font", std::make_unique<rip::FontLoader>(renderer));
    assets->addLoader("civ", std::make_unique<rip::CivLoader>(registry));
    assets->addLoader("unit", std::make_unique<rip::UnitLoader>(registry));
    assets->loadAssetsDir("assets");

    renderer.init(assets);

    rip::Game game(64, 64, registry);
    rip::MapGenerator mapgen;
    mapgen.generate(game);

    for (auto &player : game.getPlayers()) {
        player.recomputeVisibility(game);
    }

    auto startPos = game.getUnits().begin()->getPos();
    game.getView().setMapCenter(glm::vec2(startPos) * glm::vec2(100, 100));

    rip::Ui ui(window);
    rip::Hud hud(renderer.getNvg(), ui.getNk());

    glfwSwapInterval(0);
    while (!glfwWindowShouldClose(window)) {
        game.tick(window);

        // Paint order: game, UI, overlays
        renderer.begin(true);
        renderer.paintGame(game);

        ui.begin();
        hud.update(game);
        renderer.end();
        ui.render();

        renderer.begin(false);
        renderer.paintOverlays(game);
        renderer.end();

        glfwSwapBuffers(window);
        glfwPollEvents();

        while (!mouseEvents.empty()) {
            auto event = mouseEvents[0];
            hud.handleClick(game, event);
            mouseEvents.pop_front();
        }
    }

    return 0;
}
