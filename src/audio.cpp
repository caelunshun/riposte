//
// Created by Caelum van Ispelen on 5/22/21.
//

#include <iostream>
#include "audio.h"

namespace rip {
    std::shared_ptr<Asset> AudioLoader::loadAsset(const std::string &id, const std::string &data) {
        auto handle = rodio_create_sound(reinterpret_cast<const uint8_t *>(data.data()), data.size());
        return std::make_shared<SoundAsset>(handle);
    }

    SoundAsset::SoundAsset(SoundHandle *handle) : handle(handle) {}

    AudioManager::AudioManager() {
        rodio = rodio_new();
    }

    InstanceHandle *AudioManager::playSound(const std::string &id) {
        auto sound = std::dynamic_pointer_cast<SoundAsset>(assets->get(id));
        return playSound(*sound);
    }

    void AudioManager::setAssets(std::shared_ptr<Assets> assets) {
        this->assets = std::move(assets);
    }

    void AudioManager::update() {
        // check for completed sounds
        if (!playingSounds.empty()) {
            for (int i = playingSounds.size() - 1; i >= 0; i--) {
                if (rodio_is_sound_done(playingSounds[i])) {
                    playingSounds.erase(playingSounds.begin() + i);
                }
            }
        }
    }

    bool AudioManager::isSoundPlaying(InstanceHandle *sound) const {
        return !rodio_is_sound_done(sound);
    }

    InstanceHandle *AudioManager::playSound(const SoundAsset &sound) {
        const auto maxSounds = 16;
        if (playingSounds.size() > maxSounds) {
            return nullptr;
        }

        auto *instance = rodio_start_sound(rodio, sound.handle);
        playingSounds.push_back(instance);
        return instance;
    }
}
