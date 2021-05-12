//
// Created by Caelum van Ispelen on 5/11/21.
//

#include <GL/glew.h>

#define NANOVG_GL3_IMPLEMENTATION
#include <nanovg.h>
#include <nanovg_gl.h>

#include "renderer.h"

#include <glm/vec2.hpp>

namespace rip {
    std::shared_ptr<Asset> ImageLoader::loadAsset(const std::string &data) {
        auto id = nvgCreateImageMem(vg, NVG_IMAGE_GENERATE_MIPMAPS | NVG_IMAGE_REPEATX | NVG_IMAGE_REPEATY,
                                    (unsigned char *) data.c_str(), data.size());
        return std::make_shared<Image>(id);
    }

    Renderer::Renderer(GLFWwindow *window) : window(window) {
        vg = nvgCreateGL3(NVG_ANTIALIAS | NVG_STENCIL_STROKES | NVG_DEBUG);
    }

    Renderer::~Renderer() {
        nvgDeleteGL3(vg);
    }

    // PAINTERS

    /**
     * Paints tiles on the map. (No overlays - no cities, units, etc)
     */
    class TilePainter: public Painter {
        std::shared_ptr<Assets> assets;

        void paintTile(NVGcontext  *vg, glm::vec2 offset, const Tile &tile) {
            auto imageID = "texture/tile/" + std::string(tile.getTerrainID());
            auto image = std::dynamic_pointer_cast<Image>(assets->get(imageID));

            auto p = offset;
            auto paint = nvgImagePattern(vg, p.x, p.y, 100, 100, 0, image->id, 1);
            nvgBeginPath(vg);
            nvgRect(vg, p.x, p.y, 100, 100);
            nvgFillPaint(vg, paint);
            nvgFill(vg);

            // Tile border
            nvgStrokeColor(vg, nvgRGBA(0, 87, 183, 200));
            nvgStroke(vg);
        }

    public:
        explicit TilePainter(std::shared_ptr<Assets> assets) : assets(std::move(assets)) {}

        void paint(NVGcontext *vg, Game &game) override {
            for (int x = 0; x < game.getMapWidth(); x++) {
                for (int y = 0; y < game.getMapHeight(); y++) {
                    auto tile = game.getTile(glm::uvec2(x, y));
                    auto offset = game.getView().getMapCenter() + glm::vec2(x * 100, y * 100);

                    // Simple frustum cull.
                    if (offset.x < -100 || offset.y < -100
                            || offset.x > game.getCursor().getWindowSize().x
                            || offset.y > game.getCursor().getWindowSize().y) {
                        continue;
                    }

                    paintTile(vg, offset, tile);
                }
            }
        }
    };

    /**
 * Paints the custom cursor.
 */
    class CursorPainter : public Painter {
        std::shared_ptr<Image> icon;

    public:
        explicit CursorPainter(std::shared_ptr<Assets> assets) {
            icon = std::dynamic_pointer_cast<Image>(assets->get("icon/cursor"));
        }

        void paint(NVGcontext *vg, Game &game) override {
            const auto size = 20;
            auto pos = game.getCursor().getPos();
            nvgBeginPath(vg);
            nvgRect(vg, pos.x, pos.y, size, size);
            auto paint = nvgImagePattern(vg, pos.x, pos.y, size, size, 0, icon->id, 1);
            nvgFillPaint(vg, paint);
            nvgFill(vg);
        }
    };

    void Renderer::init(const std::shared_ptr<Assets>& assets) {
        painters.push_back(std::make_unique<TilePainter>(assets));
        painters.push_back(std::make_unique<CursorPainter>(assets));
    }
}
