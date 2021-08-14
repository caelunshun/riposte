//
// Created by Caelum van Ispelen on 5/22/21.
//

#ifndef RIPOSTE_AUDIO_H
#define RIPOSTE_AUDIO_H

#include <crodio.h>
#include <optional>
#include "assets.h"
#include "slot_map.h"

namespace rip {
    class Game;

    typedef ID SoundId;

    struct SoundAsset : public Asset {
        SoundHandle *handle;

        SoundAsset(SoundHandle *handle);
    };

    struct Sound {
        SoundId id;
        InstanceHandle *handle;
    };

    class AudioManager {
        std::shared_ptr<Assets> assets;

        slot_map<Sound> playingSounds;

        OutputDevice *rodio;

        InstanceHandle *playSound(const SoundAsset &sound, float volume);

        void deleteSound(SoundId sound);

    public:
        AudioManager();

        void setAssets(std::shared_ptr<Assets> assets);

        SoundId playSound(const std::string &id, float volume);

        void update();

        bool isSoundPlaying(SoundId sound) const;

        void stopSound(SoundId sound);
    };

    class AudioLoader : public AssetLoader {
        std::shared_ptr<AudioManager> manager;

    public:
        explicit AudioLoader(std::shared_ptr<AudioManager> manager) : manager(manager) {}

        std::shared_ptr<Asset> loadAsset(const std::string &id, const std::string &data) override;
    };
}

#endif //RIPOSTE_AUDIO_H
