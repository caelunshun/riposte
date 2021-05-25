#include "assets.h"
#include "renderer.h"
#include "mapgen.h"
#include "registry.h"
#include "hud.h"
#include "ui.h"
#include "audio.h"
#include "unit.h"
#include "player.h"

#include <iostream>
#include <deque>

static std::deque<rip::MouseEvent> mouseEvents;
static std::deque<int> keyEvents;

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

static void key_callback(GLFWwindow *window, int key, int scancode, int action, int mods) {
    keyEvents.push_back(key);
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

    // glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
    glfwSetMouseButtonCallback(window, mouse_callback);
    glfwSetKeyCallback(window, key_callback);

    glewExperimental = true;

    if (glewInit() != GLEW_OK) {
        return -1;
    }

    glGetError();

    rip::Renderer renderer(window);
    auto registry = std::make_shared<rip::Registry>();

    auto audio = std::make_shared<rip::AudioManager>();

    auto assets = std::make_shared<rip::Assets>();
    assets->addLoader("image", std::make_unique<rip::ImageLoader>(renderer));
    assets->addLoader("font", std::make_unique<rip::FontLoader>(renderer));
    assets->addLoader("civ", std::make_unique<rip::CivLoader>(registry));
    assets->addLoader("unit", std::make_unique<rip::UnitLoader>(registry));
    assets->addLoader("resource", std::make_unique<rip::ResourceLoader>(registry));
    assets->addLoader("tech", std::make_unique<rip::TechLoader>());
    assets->addLoader("sound", std::make_unique<rip::AudioLoader>(audio));
    assets->loadAssetsDir("assets");

    audio->addSounds(*assets);

    auto techTree = std::make_shared<rip::TechTree>(*assets, *registry);

    renderer.init(assets);

    rip::Game game(64, 64, registry);
    rip::MapGenerator mapgen;
    mapgen.generate(game, techTree);

    for (auto &player : game.getPlayers()) {
        player.recomputeVisibility(game);
    }

    auto startPos = game.getUnits().begin()->getPos();
    game.getView().setMapCenter(glm::vec2(startPos) * glm::vec2(100, 100));

    rip::Ui ui(window);
    rip::Hud hud(*assets, renderer.getNvg(), ui.getNk());

    auto vendor = glGetString(GL_VENDOR);
    auto model = glGetString(GL_RENDERER);
    std::cout << "Selected GPU: " << vendor << " " << model << std::endl;

    glfwSwapInterval(0);
    while (!glfwWindowShouldClose(window)) {
        game.tick(window, hud.hasFocus(game));
        audio->update(game);

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
        while (!keyEvents.empty()) {
            auto event = keyEvents[0];
            hud.handleKey(game, event);
            keyEvents.pop_front();
        }
    }

    return 0;
}
