//
// Created by Caelum van Ispelen on 5/11/21.
//

#include <nlohmann/json.hpp>
#include <fstream>
#include <sstream>
#include <iostream>

#include "assets.h"

namespace rip {
    class IndexEntry {
    public:
        std::string path;
        std::string id;
        std::string loader;

        NLOHMANN_DEFINE_TYPE_INTRUSIVE(IndexEntry, path, loader, id);
    };

    void Assets::addLoader(std::string name, std::unique_ptr<AssetLoader> loader) {
        loaders[std::move(name)] = std::move(loader);
    }

    void Assets::loadAssetsDir(const std::string &dir, bool skipUnknownLoaders) {
        auto indexPath = dir + "/index.json";
        std::ifstream indexFile(indexPath);
        std::stringstream indexString;
        indexString << indexFile.rdbuf();
        auto index = indexString.str();

        auto entries = nlohmann::json::parse(index).get<std::vector<IndexEntry>>();
        for (const auto &entry : entries) {
            if (skipUnknownLoaders && loaders.find(entry.loader) == loaders.end()) continue;
            auto &loader = loaders.at(entry.loader);
            std::ifstream assetFile(dir + "/" + entry.path);
            std::stringstream assetString;
            assetString << assetFile.rdbuf();

            auto asset = loader->loadAsset(entry.id, assetString.str());
            std::cout << "[assets] Loaded " << entry.id << std::endl;

            assets[entry.id] = std::move(asset);

            assetFile.close();
        }

        indexFile.close();
    }

    std::shared_ptr<Asset> Assets::get(const std::string &id) const {
        return assets.at(id);
    }
}
