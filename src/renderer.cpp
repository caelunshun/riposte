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

    std::shared_ptr<Asset> FontLoader::loadAsset(const std::string &data) {
        auto id = nvgCreateFontMem(vg, "default", (unsigned char *) data.data(), data.size(), 0);
        return std::make_shared<Font>(id);
    }

    Renderer::Renderer(GLFWwindow *window) : window(window) {
        vg = nvgCreateGL3(NVG_ANTIALIAS | NVG_STENCIL_STROKES | NVG_DEBUG);
    }

    Renderer::~Renderer() {
        nvgDeleteGL3(vg);
    }

    // Simple 2D frustum cull.
    bool shouldPaintTile(glm::vec2 offset, glm::uvec2 pos, Game &game) {
        auto &cursor = game.getCursor();
        if (offset.x < -100 || offset.y < -100
        || offset.x > cursor.getWindowSize().x
        || offset.y > cursor.getWindowSize().y) {
            return false;
        }

        if (game.getThePlayer().getVisibilityMap()[pos] != Visibility::Visible) {
            return false;
        }

        return true;
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
            nvgStrokeColor(vg, nvgRGBA(0, 87, 183, 100));
            nvgStroke(vg);
        }

    public:
        explicit TilePainter(std::shared_ptr<Assets> assets) : assets(std::move(assets)) {}

        void paint(NVGcontext *vg, Game &game) override {
            for (int x = 0; x < game.getMapWidth(); x++) {
                for (int y = 0; y < game.getMapHeight(); y++) {
                    glm::uvec2 pos(x, y);
                    auto tile = game.getTile(pos);
                    auto offset = glm::vec2(x * 100, y * 100) - game.getMapOrigin();

                    if (!shouldPaintTile(offset, pos, game)) {
                        continue;
                    }

                    paintTile(vg, offset, tile);
                }
            }
        }
    };

    /**
     * Paints cities.
     */
     class CityPainter : public Painter {
         std::shared_ptr<Image> houseIcon;

         void paintHouses(NVGcontext *vg, glm::vec2 offset) {
             const auto numHouses = 3;
             const auto aspectRatio = 1.0 / 1.424;
             const glm::vec2 housePositions[numHouses] = {
                     glm::vec2(20, 25),
                     glm::vec2(50, 25),
                     glm::vec2(25, 30),
             };
             const float houseScales[numHouses] = {
                     25,
                     25,
                     55,
             };

             for (int i = 0; i < numHouses; i++) {
                 auto pos = housePositions[i] + offset;
                 auto scale = houseScales[i];

                 nvgBeginPath(vg);
                 nvgRect(vg, pos.x, pos.y, scale * aspectRatio, scale);
                 auto paint = nvgImagePattern(vg, pos.x, pos.y, scale * aspectRatio, scale, 0, houseIcon->id, 1);
                 nvgFillPaint(vg, paint);
                 nvgFill(vg);
             }
         }

         void paintBubble(NVGcontext *vg, const City &city, glm::vec2 offset) {
             // Rectangle
             nvgBeginPath(vg);
             nvgRoundedRect(vg, offset.x, offset.y + 10, 100, 20, 5);
             auto paint = nvgLinearGradient(vg, offset.x, offset.y + 10, offset.x, offset.y + 20,
                                            nvgRGBA(61, 61, 62, 180),
                                            nvgRGBA(40, 40, 41, 180));
             nvgFillPaint(vg, paint);
             nvgFill(vg);

             // Circle
             nvgBeginPath(vg);
             const auto radius = 10;
             nvgCircle(vg, offset.x - 5 + radius, offset.y + 10 + radius, radius);
             nvgFillColor(vg, nvgRGB(182, 207, 174));
             nvgFill(vg);
             nvgStrokeColor(vg, nvgRGB(0, 0, 0));
             nvgStrokeWidth(vg, 1.5);
             nvgStroke(vg);

             // City name
             nvgFontFace(vg, "default");
             nvgTextAlign(vg, NVG_ALIGN_CENTER | NVG_ALIGN_MIDDLE);
             nvgFontSize(vg, 15);
             nvgFillColor(vg, nvgRGB(255, 255, 255));
             nvgText(vg, offset.x + 50, offset.y + 20, city.getName().c_str(), nullptr);
         }

         void paintCity(NVGcontext *vg, const City &city, glm::vec2 offset) {
             paintHouses(vg, offset);
             paintBubble(vg, city, offset);
         }

     public:
         explicit CityPainter(std::shared_ptr<Assets> assets) {
             houseIcon = std::dynamic_pointer_cast<Image>(assets->get("icon/house"));
         }

         void paint(NVGcontext *vg, Game &game) override {
            for (const auto &city : game.getCities()) {
                auto offset = glm::vec2(city.getPos()) * 100.0f - game.getMapOrigin();
                if (!shouldPaintTile(offset, city.getPos(), game)) {
                    continue;
                }

                paintCity(vg, city, offset);
            }
         }
     };

     /**
      * Paints units.
      */
      class UnitPainter : public Painter {
          std::shared_ptr<Assets> assets;

          void paintUnit(NVGcontext *vg, const Unit &unit, glm::vec2 offset) {
              auto imageID = "texture/unit/" + unit.getKind().id;
              auto image = std::dynamic_pointer_cast<Image>(assets->get(imageID));

              int imageWidth, imageHeight;
              nvgImageSize(vg, image->id, &imageWidth, &imageHeight);
              float width = imageWidth * 0.1;
              float height = imageHeight * 0.1;

              auto posX = (100.0f - width) / 2 + offset.x;
              auto posY = (100.0f - height) / 2 + offset.y;

              nvgBeginPath(vg);
              nvgRect(vg, posX, posY, width, height);
              auto paint = nvgImagePattern(vg, posX, posY, width, height, 0, image->id, 1);
              nvgFillPaint(vg, paint);
              nvgFill(vg);
          }

      public:
          explicit UnitPainter(std::shared_ptr<Assets> assets) : assets(std::move(assets)) {}

          void paint(NVGcontext *vg, Game &game) override {
            for (const auto &unit : game.getUnits()) {
                auto offset = glm::vec2(unit.getPos()) * glm::vec2(100) - game.getMapOrigin();
                if (!shouldPaintTile(offset, unit.getPos(), game)) {
                    continue;
                }

                paintUnit(vg, unit, offset);
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
        painters.push_back(std::make_unique<CityPainter>(assets));
        painters.push_back(std::make_unique<UnitPainter>(assets));
        painters.push_back(std::make_unique<CursorPainter>(assets));
    }
}
