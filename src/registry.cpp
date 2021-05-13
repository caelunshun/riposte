//
// Created by caelum on 5/12/21.
//

#include "registry.h"

namespace rip {
    std::shared_ptr<Asset> CivLoader::loadAsset(const std::string &data) {
        auto kind = nlohmann::json::parse(data).get<CivKind>();
        auto ptr = std::make_shared<CivKind>(std::move(kind));
        registry->addCiv(ptr);
        return ptr;
    }

    const std::vector<std::shared_ptr<CivKind>> &Registry::getCivs() const {
        return civs;
    }
}
