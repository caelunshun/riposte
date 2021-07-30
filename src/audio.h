//
// Created by Caelum van Ispelen on 5/22/21.
//

#ifndef RIPOSTE_AUDIO_H
#define RIPOSTE_AUDIO_H

#include <crodio.h>
#include <optional>
#include "assets.h"

namespace rip {
    class Game;

    struct SoundAsset : public Asset {
        SoundHandle *handle;

        SoundAsset(SoundHandle *handle);
    };

    class AudioManager {
        std::shared_ptr<Assets> assets;
        std::optional<InstanceHandle*> currentMusic;

        std::vector<InstanceHandle*> playingSounds;

        InstanceHandle *playSound(const SoundAsset &sound, float volume);

    public:
        OutputDevice *rodio;

        AudioManager();

        void setAssets(std::shared_ptr<Assets> assets);

        InstanceHandle *playSound(const std::string &id, float volume);

        void update();

        bool isSoundPlaying(InstanceHandle *sound) const;
    };

    class AudioLoader : public AssetLoader {
        std::shared_ptr<AudioManager> manager;

    public:
        explicit AudioLoader(std::shared_ptr<AudioManager> manager) : manager(manager) {}

        std::shared_ptr<Asset> loadAsset(const std::string &id, const std::string &data) override;
    };
}

#endif //RIPOSTE_AUDIO_H
