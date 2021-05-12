//
// Created by Caelum van Ispelen on 5/11/21.
//

#include <GL/glew.h>

#define NANOVG_GL3_IMPLEMENTATION
#include <nanovg.h>
#include <nanovg_gl.h>

#include "renderer.h"

#include <utility>
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

        void paintTile(NVGcontext  *vg, glm::uvec2 pos, glm::vec2 offset, const Tile &tile) {
            auto imageID = "texture/tile/" + std::string(tile.getTerrainID());
            auto image = std::dynamic_pointer_cast<Image>(assets->get(imageID));

            auto p = glm::vec2(pos * glm::uvec2(100)) + offset;
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
            for (int x = 0; x < 16; x++) {
                for (int y = 0; y < 16; y++) {
                    Terrain t;
                    if (x % 2 == 0) t = Terrain::Grassland;
                    else if (y % 2 == 0) t = Terrain::Desert;
                    else if (x % 3 == 0) t = Terrain::Ocean;
                    else t = Terrain::Plains;
                    Tile tile(t);

                    paintTile(vg, glm::uvec2(x, y), glm::vec2(0, 0), tile);
                }
            }
        }
    };

    void Renderer::init(std::shared_ptr<Assets> assets) {
        painters.push_back(std::make_unique<TilePainter>(assets));
    }
}
