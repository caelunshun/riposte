#include "assets.h"
#include "renderer.h"
#include "mapgen.h"
#include "registry.h"

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

    auto &capital = game.getCity(game.getThePlayer().getCities().at(0));
    game.getView().setMapCenter(glm::vec2(capital.getPos()) * glm::vec2(100, 100));

    glfwSwapInterval(0);
    while (!glfwWindowShouldClose(window)) {
        game.tick(window);
        renderer.paint(game);
    }

    return 0;
}
