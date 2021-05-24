//
// Created by Caelum van Ispelen on 5/11/21.
//

#include "renderer.h"
#include "tile.h"
#include "city.h"
#include "game.h"
#include "rng.h"

namespace rip {
    float Tile::getMovementCost() const {
        float cost = forested ? 2 : 1;

        if (hasImprovement<Road>()) {
            cost /= 3;
        }

        return cost;
    }

    bool Tile::isForested() const {
        return forested;
    }

    void Tile::setForested(bool forested) {
        this->forested = forested;
    }

    bool Tile::hasImprovement(const std::string &name) const {
        for (const auto &improvement : improvements) {
            if (improvement->getName() == name) {
                return true;
            }
        }
        return false;
    }

    Yield Tile::getYield(const Game &game, glm::uvec2 pos, PlayerId playerID) const {
        Yield yield(0, 0, 0);

        switch (terrain) {
            case Grassland:
                yield.commerce += 1;
                yield.food += 2;
                break;
            case Plains:
                yield.food += 1;
                yield.hammers += 1;
                break;
            case Ocean:
                yield.food += 2;
                yield.commerce += 2;
                break;
            case Desert:
                break;
        }

        if (forested) {
            yield.hammers += 1;
        }

        if (game.getCityAtLocation(pos)) {
            yield.hammers += 1;
            yield.food += 1;
            yield.commerce += 1;
        }

        for (const auto &improvement : getImprovements()) {
            yield += improvement->getYieldContribution(game);
        }

        // Resource.
        if (resource.has_value()) {
            const auto &theResource = **resource;
            const auto &player = game.getPlayer(playerID);
            if (player.getTechs().isTechUnlocked(theResource.revealedBy)) {
                yield += theResource.yieldBonus;

                if (hasImprovement(theResource.improvement)) {
                    yield += theResource.improvedBonus;
                }
            }
        }

        return yield;
    }

    const std::vector<std::unique_ptr<Improvement>> &Tile::getImprovements() const {
        return improvements;
    }

    bool Tile::addImprovement(std::unique_ptr<Improvement> improvement) {
        if (improvement->isCompatible(*this)) {
            improvements.push_back(std::move(improvement));
            return true;
        } else {
            return false;
        }
    }

    void Tile::clearImprovements() {
        improvements.clear();
    }

    static void paintImprovementIcon(NVGcontext *vg, const Assets &assets, glm::vec2 offset, const std::string &assetID) {
        const auto &image = std::dynamic_pointer_cast<Image>(assets.get(assetID));
        nvgBeginPath(vg);
        auto aspectRatio = 640.0f / 512;
        auto width = 60.0f;
        auto height = aspectRatio * width;
        offset += 50.0f;
        offset -= glm::vec2(width, height) / 2.0f;
        nvgRect(vg, offset.x, offset.y, width, height);
        auto paint = nvgImagePattern(vg, offset.x, offset.y, width, height, 0, image->id, 1);
        nvgFillPaint(vg, paint);
        nvgFill(vg);
    }

    bool Mine::isCompatible(const Tile &tile) const {
        return !tile.hasNonRoadImprovements() && tile.getTerrain() != Terrain::Desert;
    }

    Yield Mine::getYieldContribution(const Game &game) const {
        return Yield(2, 0, 0);
    }

    void Mine::paint(const Game &game, glm::uvec2 pos, NVGcontext *vg, const Assets &assets) {
        paintImprovementIcon(vg, assets, game.getScreenOffset(pos), "icon/mine");
    }

    std::string Mine::getName() const {
        return "Mine";
    }

    int Mine::getNumBuildTurns() const {
        return 5;
    }

    bool Cottage::isCompatible(const Tile &tile) const {
        return !tile.hasNonRoadImprovements() && tile.getTerrain() != Terrain::Desert;
    }

    Yield Cottage::getYieldContribution(const Game &game) const {
        return Yield(0, 1, 0);
    }

    void Cottage::paint(const Game &game, glm::uvec2 pos, NVGcontext *vg, const Assets &assets) {
        paintImprovementIcon(vg, assets, game.getScreenOffset(pos), "icon/cottage");
    }

    std::string Cottage::getName() const {
        return "Cottage";
    }

    int Cottage::getNumBuildTurns() const {
        return 4;
    }

    bool Farm::isCompatible(const Tile &tile) const {
        return !tile.hasNonRoadImprovements() && tile.getTerrain() != Terrain::Desert;
    }

    Yield Farm::getYieldContribution(const Game &game) const {
        return Yield(0, 0, 1);
    }

    int Farm::getNumBuildTurns() const {
        return 5;
    }

    std::string Farm::getName() const {
        return "Farm";
    }

    void Farm::paint(const Game &game, glm::uvec2 pos, NVGcontext *vg, const Assets &assets) {
        paintImprovementIcon(vg, assets, game.getScreenOffset(pos), "icon/farm");
    }

    bool Road::isCompatible(const Tile &tile) const {
        return tile.getTerrain() != Terrain::Ocean && !tile.hasImprovement("Road");
    }

    Yield Road::getYieldContribution(const Game &game) const {
        return Yield();
    }

    int Road::getNumBuildTurns() const {
        return 2;
    }

    std::string Road::getName() const {
        return "Road";
    }

    static uint64_t makeSeed(glm::uvec2 pos) {
        return (static_cast<uint64_t>(pos.x) << 32) | static_cast<uint64_t>(pos.y);
    }

    void Road::paint(const Game &game, glm::uvec2 pos, NVGcontext *vg, const Assets &assets) {
        const auto offset = game.getScreenOffset(pos);
        const auto center = offset + 50.0f;
        Rng rng(makeSeed(pos));

        std::array<glm::vec2, 8> entryPoints;
        int numEntryPoints = 0;

        for (const auto neighborPos : getSideNeighbors(pos)) {
            if (!game.containsTile(neighborPos)) continue;
            const auto &neighbor = game.getTile(neighborPos);
            const auto diff = glm::ivec2(neighborPos) - glm::ivec2(pos);
            const auto edgeCenter = center + glm::vec2(diff * 50);
            if (neighbor.hasImprovement<Road>() || game.getCityAtLocation(neighborPos)) {
                entryPoints[numEntryPoints++] = edgeCenter;
            }
        }

        // Draw road connections.
        for (int i = 0; i < numEntryPoints; i += 2) {
            auto first = entryPoints[i];
            glm::vec2 second;
            if (i < numEntryPoints - 1) {
                second = entryPoints[i + 1];
            } else {
                second = center;
            }

            nvgBeginPath(vg);
            nvgMoveTo(vg, first.x, first.y);
            nvgLineTo(vg, second.x, second.y);

            nvgLineCap(vg, NVG_ROUND);
            nvgStrokeWidth(vg, 5);
            nvgStrokeColor(vg, nvgRGB(80, 80, 80));
            nvgStroke(vg);
        }
    }

    bool Pasture::isCompatible(const Tile &tile) const {
        return !tile.hasNonRoadImprovements() && tile.hasImproveableResource(getName());
    }

    Yield Pasture::getYieldContribution(const Game &game) const {
        return Yield();
    }

    int Pasture::getNumBuildTurns() const {
        return 5;
    }

    std::string Pasture::getName() const {
        return "Pasture";
    }

    void Pasture::paint(const Game &game, glm::uvec2 pos, NVGcontext *vg, const Assets &assets) {
        paintImprovementIcon(vg, assets, game.getScreenOffset(pos), "icon/pasture");
    }
}
