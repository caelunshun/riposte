//
// Created by Caelum van Ispelen on 5/11/21.
//

#include "renderer.h"
#include "tile.h"
#include "city.h"
#include "game.h"

namespace rip {
    int Tile::getMovementCost() const {
        return (forested ? 2 : 1);
    }

    bool Tile::isForested() const {
        return forested;
    }

    void Tile::setForested(bool forested) {
        this->forested = forested;
    }

    Yield Tile::getYield(const Game &game, glm::uvec2 pos) const {
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
        return tile.getImprovements().empty();
    }

    Yield Mine::getYieldContribution(const Game &game) const {
        return Yield(2, 0, 0);
    }

    void Mine::paint(NVGcontext *vg, const Assets &assets, glm::vec2 offset) {
        paintImprovementIcon(vg, assets, offset, "icon/mine");
    }

    std::string Mine::getName() const {
        return "Mine";
    }

    int Mine::getNumBuildTurns() const {
        return 5;
    }

    bool Cottage::isCompatible(const Tile &tile) const {
        return tile.getImprovements().empty();
    }

    Yield Cottage::getYieldContribution(const Game &game) const {
        return Yield(0, 1, 0);
    }

    void Cottage::paint(NVGcontext *vg, const Assets &assets, glm::vec2 offset) {
        paintImprovementIcon(vg, assets, offset, "icon/cottage");
    }

    std::string Cottage::getName() const {
        return "Cottage";
    }

    int Cottage::getNumBuildTurns() const {
        return 4;
    }
}
