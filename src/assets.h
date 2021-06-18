//
// Created by Caelum van Ispelen on 5/11/21.
//

#ifndef RIPOSTE_ASSETS_H
#define RIPOSTE_ASSETS_H

#include <unordered_map>
#include <string>
#include <vector>
#include <memory>

namespace rip {
    class Asset {
    public:
        virtual ~Asset() {}
    };

    class AssetLoader {
    public:
        virtual std::shared_ptr<Asset> loadAsset(const std::string &id, const std::string &data) = 0;
        virtual ~AssetLoader() {}
    };

    class Assets {
        std::unordered_map<std::string, std::shared_ptr<Asset>> assets;
        std::unordered_map<std::string, std::unique_ptr<AssetLoader>> loaders;

    public:
        void addLoader(std::string name, std::unique_ptr<AssetLoader> loader);

        void loadAssetsDir(const std::string &dir);

        std::shared_ptr<Asset> get(const std::string &id) const;

        template<class T>
        std::vector<std::shared_ptr<T>> getAll() const {
            std::vector<std::shared_ptr<T>> results;
            for (const auto &entry : assets) {
                const auto &asset = entry.second;
                auto assetDowncasted = std::dynamic_pointer_cast<T>(asset);
                if (assetDowncasted) {
                    results.push_back(std::move(assetDowncasted));
                }
            }
            return results;
        }

        template<class T>
        std::vector<std::pair<std::string, std::shared_ptr<T>>> getAllWithIDs() const {
            std::vector<std::pair<std::string, std::shared_ptr<T>>> results;
            for (const auto &entry : assets) {
                const auto &asset = entry.second;
                auto assetDowncasted = std::dynamic_pointer_cast<T>(asset);
                if (assetDowncasted) {
                    results.push_back({entry.first, std::move(assetDowncasted)});
                }
            }
            return results;
        }
    };
}

#endif //RIPOSTE_ASSETS_H
