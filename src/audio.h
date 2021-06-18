//
// Created by Caelum van Ispelen on 5/22/21.
//

#ifndef RIPOSTE_AUDIO_H
#define RIPOSTE_AUDIO_H

#include <crodio.h>
#include <optional>
#include <absl/container/flat_hash_map.h>
#include "era.h"
#include "rng.h"
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
        Era currentMusicEra = Era::Ancient;

        absl::flat_hash_map<Era, FairPicker<std::shared_ptr<SoundAsset>>> eraMusic;

        std::vector<InstanceHandle*> playingSounds;

        void updateEraMusic(const Game &game);

        InstanceHandle *playSound(const SoundAsset &sound);

    public:
        OutputDevice *rodio;

        AudioManager();

        void addSounds(std::shared_ptr<Assets> assets);

        void playSound(const std::string &id);

        void update(const Game &game);
    };

    class AudioLoader : public AssetLoader {
        std::shared_ptr<AudioManager> manager;

    public:
        explicit AudioLoader(std::shared_ptr<AudioManager> manager) : manager(manager) {}

        std::shared_ptr<Asset> loadAsset(const std::string &id, const std::string &data) override;
    };
}

#endif //RIPOSTE_AUDIO_H
