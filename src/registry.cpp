//
// Created by Caelum van Ispelen on 5/12/21.
//

#include "registry.h"

namespace rip {
    std::shared_ptr<Asset> CivLoader::loadAsset(const std::string &data) {
        auto kind = nlohmann::json::parse(data).get<CivKind>();
        auto ptr = std::make_shared<CivKind>(std::move(kind));
        registry->addCiv(ptr);
        return ptr;
    }

    std::shared_ptr<Asset> UnitLoader::loadAsset(const std::string &data) {
        auto unit = nlohmann::json::parse(data).get<UnitKind>();
        auto ptr = std::make_shared<UnitKind>(std::move(unit));
        registry->addUnit(ptr);
        return ptr;
    }

    const std::vector<std::shared_ptr<CivKind>> &Registry::getCivs() const {
        return civs;
    }

    const std::vector<std::shared_ptr<UnitKind>> &Registry::getUnits() const {
        return units;
    }

    std::shared_ptr<Asset> ResourceLoader::loadAsset(const std::string &data) {
        auto resource = nlohmann::json::parse(data).get<Resource>();
        auto ptr = std::make_shared<Resource>(std::move(resource));
        registry->addResource(ptr);
        return ptr;
    }
}
