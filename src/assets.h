//
// Created by Caelum van Ispelen on 5/11/21.
//

#ifndef RIPOSTE_ASSETS_H
#define RIPOSTE_ASSETS_H

#include <unordered_map>
#include <string>
#include <memory>

namespace rip {
    class Asset {
    public:
        virtual ~Asset() {}
    };

    class AssetLoader {
    public:
        virtual std::shared_ptr<Asset> loadAsset(const std::string &data) = 0;
        virtual ~AssetLoader() {}
    };

    class Assets {
        std::unordered_map<std::string, std::shared_ptr<Asset>> assets;
        std::unordered_map<std::string, std::unique_ptr<AssetLoader>> loaders;

    public:
        void addLoader(std::string name, std::unique_ptr<AssetLoader> loader);

        void loadAssetsDir(const std::string &dir);

        std::shared_ptr<Asset> get(const std::string &id) const;
    };
}

#endif //RIPOSTE_ASSETS_H
