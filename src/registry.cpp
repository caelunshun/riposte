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

    const absl::flat_hash_map<std::string, std::shared_ptr<Resource>> &Registry::getResources() const {
        return resources;
    }

    const std::shared_ptr<UnitKind> &Registry::getUnit(const std::string &id) const {
        for (const auto &unit : units) {
            if (unit->id == id) {
                return unit;
            }
        }
        throw std::string("missing unit '" + id + "'");
    }

    std::shared_ptr<Asset> BuildingLoader::loadAsset(const std::string &data) {
        auto building = nlohmann::json::parse(data).get<Building>();
        auto ptr = std::make_shared<Building>(std::move(building));
        registry->addBuilding(ptr);
        return ptr;
    }

    const std::vector<std::shared_ptr<Building>> &Registry::getBuildings() const {
        return buildings;
    }

    const std::shared_ptr<Building> &Registry::getBuilding(const std::string &name) const {
        for (const auto &building : buildings) {
            if (building->name == name) {
                return building;
            }
        }
        throw std::string("missing building '" + name + "'");
    }
}
