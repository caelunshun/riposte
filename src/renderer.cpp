//
// Created by Caelum van Ispelen on 5/11/21.
//

#include <GL/glew.h>

#define NANOVG_GL3_IMPLEMENTATION

#include <nanovg.h>
#include <nanovg_gl.h>

#include "renderer.h"
#include "ripmath.h"
#include "rng.h"

#include <glm/glm.hpp>

namespace rip {
    std::shared_ptr<Asset> ImageLoader::loadAsset(const std::string &data) {
        auto id = nvgCreateImageMem(vg, NVG_IMAGE_GENERATE_MIPMAPS | NVG_IMAGE_REPEATX | NVG_IMAGE_REPEATY,
                                    (unsigned char *) data.c_str(), data.size());
        return std::make_shared<Image>(id);
    }

    std::shared_ptr<Asset> FontLoader::loadAsset(const std::string &data) {
        // The data is now owned by NanoVG, so we have to make a copy to avoid use-after-free.
        auto *dataCopy = (unsigned char *) malloc(data.size());
        memcpy(dataCopy, data.data(), data.size());
        auto id = nvgCreateFontMem(vg, "default", dataCopy, data.size(), 0);
        return std::make_shared<Font>(id);
    }

    Renderer::Renderer(GLFWwindow *window) : window(window) {
        vg = nvgCreateGL3(NVG_ANTIALIAS | NVG_STENCIL_STROKES | NVG_DEBUG);
    }

    Renderer::~Renderer() {
        nvgDeleteGL3(vg);
    }

    // Simple 2D frustum cull.
    bool shouldPaintTile(glm::vec2 offset, glm::uvec2 pos, Game &game, bool allowFog) {
        auto &cursor = game.getCursor();
        if (offset.x < -100 || offset.y < -100
            || offset.x > cursor.getWindowSize().x
            || offset.y > cursor.getWindowSize().y) {
            return false;
        }

        auto vis = game.getThePlayer().getVisibilityMap()[pos];
        if (!game.isCheatMode()
            && (vis == Visibility::Hidden || (vis == Visibility::Fogged && !allowFog))) {
            return false;
        }

        return true;
    }

    // PAINTERS

    /**
     * Paints tiles on the map. (No overlays - no cities, units, etc)
     */
    class TilePainter : public Painter {
        std::shared_ptr<Assets> assets;

        void paintTile(NVGcontext *vg, Game &game, glm::vec2 offset, glm::uvec2 tilePos, Tile &tile) {
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

            // Improvements
            for (auto &improvement : tile.getImprovements()) {
                improvement->paint(vg, *assets, offset);
            }
        }

    public:
        explicit TilePainter(std::shared_ptr<Assets> assets) : assets(std::move(assets)) {

        }

        void paint(NVGcontext *vg, Game &game) override {
            for (int x = 0; x < game.getMapWidth(); x++) {
                for (int y = 0; y < game.getMapHeight(); y++) {
                    glm::uvec2 pos(x, y);
                    auto &tile = game.getTile(pos);
                    auto offset = glm::vec2(x * 100, y * 100) - game.getMapOrigin();

                    if (!shouldPaintTile(offset, pos, game, true)) {
                        continue;
                    }

                    paintTile(vg, game, offset, pos, tile);
                }
            }
        }
    };

    // Paints trees on forested tiles.
    class TreePainter : public Painter {
        std::shared_ptr<Image> treeIcon;

        void paintTreeTile(NVGcontext *vg, const Game &game, glm::uvec2 tilePos, glm::vec2 offset) {
            // Seed based on tile position to ensure stability.
            const auto seed = tilePos.x + tilePos.y * game.getMapWidth();
            Rng rng(seed);

            const auto numTrees = rng.u32(10, 20);
            for (int i = 0; i < numTrees; i++) {
                auto scaleX = (rng.f32() + 1) * 25;
                auto scaleY = scaleX * (640.0f / 512);
                auto pos = glm::vec2(rng.f32(), rng.f32()) * 100.0f;
                pos -= glm::vec2(scaleX, scaleY) / 2.0f;
                nvgBeginPath(vg);
                nvgRect(vg, offset.x + pos.x, offset.y + pos.y, scaleX, scaleY);
                auto paint = nvgImagePattern(vg, offset.x + pos.x, offset.y + pos.y, scaleX, scaleY, 0, treeIcon->id, 1);
                nvgFillPaint(vg, paint);
                nvgFill(vg);
            }
        }

    public:
        explicit TreePainter(std::shared_ptr<Assets> assets) {
            treeIcon = std::dynamic_pointer_cast<Image>(assets->get("icon/tree"));
        }

        void paint(NVGcontext *vg, Game &game) override {
            for (int x = 0; x < game.getMapWidth(); x++) {
                for (int y = 0; y < game.getMapHeight(); y++) {
                    glm::uvec2 tilePos(x, y);
                    auto offset = game.getScreenOffset(tilePos);

                    if (!shouldPaintTile(offset, tilePos, game, true)) {
                        continue;
                    }

                    const auto &tile = game.getTile(tilePos);
                    if (tile.isForested()) {
                        paintTreeTile(vg, game, tilePos, offset);
                    }
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

        void paintBubble(NVGcontext *vg, const City &city, const Game &game, glm::vec2 offset) {
            // Rectangle
            const auto width = 100;
            const auto bubbleHeight = 20;
            nvgBeginPath(vg);
            nvgRoundedRect(vg, offset.x, offset.y + 10, width, bubbleHeight, 5);
            auto paint = nvgLinearGradient(vg, offset.x, offset.y + 10, offset.x, offset.y + 30,
                                           nvgRGBA(61, 61, 62, 180),
                                           nvgRGBA(40, 40, 41, 180));
            nvgFillPaint(vg, paint);
            nvgFill(vg);

            const auto task = city.getBuildTask();

            // Production progress bar
            if (task) {
                auto progress = static_cast<double>(task->getProgress()) / task->getCost();
                auto projectedProgress =
                        static_cast<double>(task->getProgress() + city.computeYield(game).hammers) / task->getCost();

                projectedProgress = std::clamp(projectedProgress, 0.0, 1.0);

                const auto offsetY = 20;
                const auto height = bubbleHeight / 2;

                nvgBeginPath(vg);
                nvgRect(vg, offset.x, offset.y + offsetY, width * progress, height);
                nvgFillColor(vg, nvgRGBA(72, 159, 223, 160));
                nvgFill(vg);

                nvgBeginPath(vg);
                nvgRect(vg, offset.x + width * progress, offset.y + offsetY, width * (projectedProgress - progress),
                        height);
                nvgFillColor(vg, nvgRGBA(141, 200, 232, 160));
                nvgFill(vg);
            }

            // Left circle
            nvgBeginPath(vg);
            const auto radius = 10;
            nvgCircle(vg, offset.x - 5 + radius, offset.y + 10 + radius, radius);
            nvgFillColor(vg, nvgRGB(182, 207, 174));
            nvgFill(vg);
            nvgStrokeColor(vg, nvgRGB(0, 0, 0));
            nvgStrokeWidth(vg, 1.5);
            nvgStroke(vg);

            // Right circle
            nvgBeginPath(vg);
            nvgCircle(vg, offset.x + width + 5 - radius, offset.y + 10 + radius, radius);
            nvgFillColor(vg, nvgRGB(244, 195, 204));
            nvgFill(vg);
            nvgStrokeColor(vg, nvgRGB(0, 0, 0));
            nvgStroke(vg);

            // Left circle text (population)
            auto population = city.getPopulation();
            nvgFontSize(vg, 12);
            nvgFillColor(vg, nvgRGB(0, 0, 0));
            nvgTextAlign(vg, NVG_ALIGN_CENTER | NVG_ALIGN_MIDDLE);
            nvgText(vg, offset.x + - 5 + radius, offset.y + 10 + radius, std::to_string(population).c_str(), nullptr);

            // Right circle text (first character of current build task)
            if (task) {
                auto text = task->getName().substr(0, 1);
                nvgText(vg, offset.x + width + 5 - radius, offset.y + 10 + radius, text.c_str(), nullptr);
            }

            // City name
            nvgFontFace(vg, "default");
            nvgTextAlign(vg, NVG_ALIGN_CENTER | NVG_ALIGN_MIDDLE);
            nvgFontSize(vg, 15);
            nvgFillColor(vg, nvgRGB(255, 255, 255));
            nvgText(vg, offset.x + 50, offset.y + 20, city.getName().c_str(), nullptr);
        }

        void paintCity(NVGcontext *vg, const City &city, const Game &game, glm::vec2 offset) {
            paintHouses(vg, offset);
            paintBubble(vg, city, game, offset);
        }

    public:
        explicit CityPainter(std::shared_ptr<Assets> assets) {
            houseIcon = std::dynamic_pointer_cast<Image>(assets->get("icon/house"));
        }

        void paint(NVGcontext *vg, Game &game) override {
            for (const auto &city : game.getCities()) {
                auto offset = glm::vec2(city.getPos()) * 100.0f - game.getMapOrigin();
                if (!shouldPaintTile(offset, city.getPos(), game, true)) {
                    continue;
                }

                paintCity(vg, city, game, offset);
            }
        }
    };

    /**
     * Paints units.
     */
    class UnitPainter : public Painter {
        std::shared_ptr<Assets> assets;

        glm::vec2 getInterpolatedUnitPos(glm::vec2 fromPos, glm::vec2 toPos, float time) {
            float pos;
            // integral of cosine velocity function
            const auto end = 1;
            const auto vel = 1000;
            if (time <= end) {
                pos = vel * -cos(time * (end * 2 * pi())) + vel;
            } else {
                pos = (vel * -cos(end * (end / 2 * pi())) + vel) + vel * (time - end);
            }

            auto dist = glm::distance(fromPos, toPos);
            pos = std::clamp(pos, 0.0f, dist);

            auto ray = glm::normalize(toPos - fromPos);
            return fromPos + (ray * pos);
        }

        void paintUnit(NVGcontext *vg, Game &game, const Unit &unit, glm::vec2 offset) {
            auto imageID = "texture/unit/" + unit.getKind().id;
            auto image = std::dynamic_pointer_cast<Image>(assets->get(imageID));

            int imageWidth, imageHeight;
            nvgImageSize(vg, image->id, &imageWidth, &imageHeight);
            float width = imageWidth * 0.1;
            float height = imageHeight * 0.1;

            auto posX = (100.0f - width) / 2 + offset.x;
            auto posY = (100.0f - height) / 2 + offset.y;

            // Unit icon
            nvgBeginPath(vg);
            nvgRect(vg, posX, posY, width, height);
            auto paint = nvgImagePattern(vg, posX, posY, width, height, 0, image->id, 1);
            nvgFillPaint(vg, paint);
            nvgFill(vg);

            // Colored rectangle to indicate nationality
            const auto &owner = game.getPlayer(unit.getOwner());
            const auto color = owner.getCiv().color;
            nvgBeginPath(vg);
            nvgRect(vg, posX + width - 10, posY + height / 2 - 15, 25, 30);
            nvgFillColor(vg, nvgRGB(color[0], color[1], color[2]));
            nvgFill(vg);

            // Text (unit name)
            nvgFontSize(vg, 14);
            nvgFillColor(vg, nvgRGB(0, 0, 0));
            nvgTextAlign(vg, NVG_ALIGN_CENTER | NVG_ALIGN_TOP);
            nvgText(vg, posX + (width / 2), posY + height, unit.getKind().name.c_str(), nullptr);
        }

    public:
        explicit UnitPainter(std::shared_ptr<Assets> assets) : assets(std::move(assets)) {}

        void paint(NVGcontext *vg, Game &game) override {
            for (auto &unit : game.getUnits()) {
                auto offset = game.getScreenOffset(unit.getPos());
                if (!shouldPaintTile(offset, unit.getPos(), game, false)) {
                    continue;
                }

                if (unit.moveTime != -1) {
                    auto newOffset = getInterpolatedUnitPos(game.getScreenOffset(unit.moveFrom), offset, unit.moveTime);
                    unit.moveTime += game.getDeltaTime();
                    if (glm::distance(offset, newOffset) <= 0.1) {
                        unit.moveTime = -1;
                    }
                    offset = newOffset;
                }

                paintUnit(vg, game, unit, offset);
            }
        }
    };

    // Paints the yield for tiles.
    class YieldPainter : public Painter {
        std::shared_ptr<Image> foodIcon;
        std::shared_ptr<Image> hammerIcon;
        std::shared_ptr<Image> coinIcon;

        void paintTile(NVGcontext *vg, Game &game, Tile &tile, glm::uvec2 tilePos, glm::vec2 offset) {
            auto yieldScale = 15;

            if (game.isTileWorked(tilePos)) {
                yieldScale = 25;
            }

            const auto yield = tile.getYield(game, tilePos, game.getThePlayerID());
            std::vector<std::pair<int, float>> icons;
            auto cursor = 0;
            const auto spacing = 6;
            const auto bigSpacing = 20;
            for (int i = 0; i < yield.food; i++) {
                icons.emplace_back(foodIcon->id, cursor);
                cursor += spacing;
            }
            if (yield.hammers != 0) cursor += bigSpacing;
            for (int i = 0; i < yield.hammers; i++) {
                icons.emplace_back(hammerIcon->id, cursor);
                cursor += spacing;
            }
            if (yield.commerce != 0) cursor += bigSpacing;
            for (int i = 0; i < yield.commerce; i++) {
                icons.emplace_back(coinIcon->id, cursor);
                cursor += spacing;
            }

            float length = 0;
            if (!icons.empty()) length = icons[icons.size() - 1].second + yieldScale;

            for (const auto entry : icons) {
                auto iconID = entry.first;
                auto offsetX = entry.second + (50 - length / 2);
                nvgBeginPath(vg);
                nvgRect(vg, offset.x + offsetX, offset.y + 50 - yieldScale / 2, yieldScale, yieldScale);
                auto paint = nvgImagePattern(vg, offset.x + offsetX, offset.y + 50 - yieldScale / 2,
                                             yieldScale, yieldScale, 0, iconID, 1);
                nvgFillPaint(vg, paint);
                nvgFill(vg);
            }
        }

    public:
        explicit YieldPainter(std::shared_ptr<Assets> assets) {
            coinIcon = std::dynamic_pointer_cast<Image>(assets->get("icon/coin"));
            foodIcon = std::dynamic_pointer_cast<Image>(assets->get("icon/bread"));
            hammerIcon = std::dynamic_pointer_cast<Image>(assets->get("icon/hammer"));
        }

        void paint(NVGcontext *vg, Game &game) override {
            for (int x = 0; x < game.getMapWidth(); x++) {
                for (int y = 0; y < game.getMapHeight(); y++) {
                    glm::uvec2 tilePos(x, y);
                    auto offset = game.getScreenOffset(tilePos);
                    if (shouldPaintTile(offset, tilePos, game, true)) {
                        paintTile(vg, game, game.getTile(tilePos), tilePos, offset);
                    }
                }
            }
        }
    };

    /**
     * Paints a fog overlay over fogged regions.
     */
    class FogPainter : public Painter {
    public:
        void paint(NVGcontext *vg, Game &game) override {
            for (int x = 0; x < game.getMapWidth(); x++) {
                for (int y = 0; y < game.getMapHeight(); y++) {
                    glm::uvec2 pos(x, y);
                    auto offset = game.getScreenOffset(pos);
                    if (!shouldPaintTile(offset, pos, game, true)) {
                        continue;
                    }

                    if (game.getThePlayer().getVisibilityMap()[pos] != Visibility::Fogged) {
                        continue;
                    }

                    nvgBeginPath(vg);
                    nvgRect(vg, offset.x, offset.y, 100, 100);
                    nvgFillColor(vg, nvgRGBA(50, 50, 50, 150));
                    nvgFill(vg);
                }
            }
        }
    };

    // Paints cultural borders.
    class CultureBorderPainter : public Painter {
        void paintTileCulture(NVGcontext *vg, Game &game, glm::uvec2 tilePos, glm::vec2 offset) {
            auto owner = game.getCultureMap().getTileOwner(tilePos);
            const auto width = 3.0f;
            nvgStrokeWidth(vg, width);
            nvgLineCap(vg, NVG_SQUARE);
            if (owner.has_value()) {
                // Draw borders only when the neighbor on that side has a different owner.
                for (const auto neighborTilePos : getSideNeighbors(tilePos)) {
                    if (!game.containsTile(neighborTilePos)) continue;
                    auto neighborOwner = game.getCultureMap().getTileOwner(neighborTilePos);
                    if (neighborOwner != owner) {
                        // Draw a border along this edge.
                        glm::vec2 edgeOffset;
                        glm::vec2 edgeLength;
                        glm::vec2 crossVector;

                        auto diff = glm::ivec2(neighborTilePos) - glm::ivec2(tilePos);
                        if (diff == glm::ivec2(1, 0)) {
                            edgeOffset = glm::vec2(100 - width / 2, width / 2);
                            edgeLength = glm::vec2(0, 100 - width);
                            crossVector = glm::vec2(-1, 0);
                        } else if (diff == glm::ivec2(-1, 0)) {
                            edgeOffset = glm::vec2(width / 2,  width / 2);
                            edgeLength = glm::vec2(0, 100 - width);
                            crossVector = glm::vec2(1, 0);
                        } else if (diff == glm::ivec2(0, 1)) {
                            edgeOffset = glm::vec2(width / 2, 100 - width / 2);
                            edgeLength = glm::vec2(100 - width, 0);
                            crossVector = glm::vec2(0, -1);
                        } else {
                            edgeOffset = glm::vec2(width / 2, width / 2);
                            edgeLength = glm::vec2(100 - width, 0);
                            crossVector = glm::vec2(0, 1);
                        }

                        nvgBeginPath(vg);
                        auto start = offset + edgeOffset;
                        auto end = start + edgeLength;
                        nvgMoveTo(vg, start.x, start.y);
                        nvgLineTo(vg, end.x, end.y);
                        const auto &color = game.getPlayer(*owner).getCiv().color;
                        nvgStrokeColor(vg, nvgRGB(color[0], color[1], color[2]));
                        nvgStroke(vg);

                        // Gradient to indicate direction of border.
                        nvgBeginPath(vg);
                        auto gradientStart = start;
                        auto gradientEnd = gradientStart + crossVector * 30.0f;
                        nvgRect(vg, gradientStart.x, gradientStart.y,
                                gradientEnd.x - gradientStart.x + edgeLength.x,
                                gradientEnd.y - gradientStart.y + edgeLength.y);
                        auto paint = nvgLinearGradient(vg, gradientStart.x, gradientStart.y, gradientEnd.x, gradientEnd.y,
                                                       nvgRGBA(color[0], color[1], color[2], 128),
                                                       nvgRGBA(color[0], color[1], color[2], 0));
                        nvgFillPaint(vg, paint);
                        nvgFill(vg);
                    }
                }
            }
        }

    public:
        void paint(NVGcontext *vg, Game &game) override {
            for (int x = 0; x < game.getMapWidth(); x++) {
                for (int y = 0; y < game.getMapHeight(); y++) {
                    glm::uvec2 tilePos(x, y);
                    auto offset = game.getScreenOffset(tilePos);
                    if (!shouldPaintTile(offset, tilePos, game, true)) {
                        continue;
                    }

                    paintTileCulture(vg, game, tilePos, offset);
                }
            }
        }
    };

    // Paints tile resources.
    class ResourcePainter : public Painter {
        std::shared_ptr<Assets> assets;

    public:
        explicit ResourcePainter(std::shared_ptr<Assets> assets) : assets(std::move(assets)) {}

        void paint(NVGcontext *vg, Game &game) override {
            for (int x = 0; x < game.getMapWidth(); x++) {
                for (int y = 0; y < game.getMapHeight(); y++) {
                    glm::uvec2 tilePos(x, y);
                    auto offset = game.getScreenOffset(tilePos);
                    if (!shouldPaintTile(offset, tilePos, game, true)) {
                        continue;
                    }

                    const auto &tile = game.getTile(tilePos);
                    if (!tile.hasResource()) continue;
                    const auto &resource = *tile.getResource();
                    if (!game.getThePlayer().getTechs().isTechUnlocked(resource->revealedBy)) {
                        continue;
                    }

                    auto resourceID = "texture/resource/" + resource->id;
                    const auto image = std::dynamic_pointer_cast<Image>(assets->get(resourceID))->id;

                    nvgBeginPath(vg);
                    nvgRect(vg, offset.x, offset.y, 100, 100);
                    auto paint = nvgImagePattern(vg, offset.x, offset.y, 100, 100, 0, image, 1);
                    nvgFillPaint(vg, paint);
                    nvgFill(vg);
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
            const auto size = 25;
            auto pos = game.getCursor().getPos();
            nvgBeginPath(vg);
            nvgRect(vg, pos.x, pos.y, size, size);
            auto paint = nvgImagePattern(vg, pos.x, pos.y, size, size, 0, icon->id, 1);
            nvgFillPaint(vg, paint);
            nvgFill(vg);
        }
    };

    void Renderer::init(const std::shared_ptr<Assets> &assets) {
        // Painting happens in the order painters are added here,
        // allowing for layering.
        gamePainters.push_back(std::make_unique<TilePainter>(assets));
        gamePainters.push_back(std::make_unique<ResourcePainter>(assets));
        gamePainters.push_back(std::make_unique<TreePainter>(assets));
        gamePainters.push_back(std::make_unique<CultureBorderPainter>());
        gamePainters.push_back(std::make_unique<CityPainter>(assets));
        gamePainters.push_back(std::make_unique<YieldPainter>(assets));
        gamePainters.push_back(std::make_unique<UnitPainter>(assets));
        gamePainters.push_back(std::make_unique<FogPainter>());
        overlayPainters.push_back(std::make_unique<CursorPainter>(assets));
    }
}
